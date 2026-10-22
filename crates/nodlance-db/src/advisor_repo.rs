//! PostgreSQL implementation of AdvisorRepository for DBA heuristics, findings, and query digests.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use nodlance_domain::{
    AdvisorCategory, AdvisorFinding, AdvisorRepository, AdvisorRule, AdvisorSeverity, DomainError,
    QueryDigest,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgAdvisorRepository {
    pool: PgPool,
}

impl PgAdvisorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AdvisorRuleRow {
    id: String,
    name: String,
    engine: String,
    category: String,
    severity: String,
    description: String,
    script_content: String,
    is_enabled: bool,
}

impl TryFrom<AdvisorRuleRow> for AdvisorRule {
    type Error = DomainError;

    fn try_from(row: AdvisorRuleRow) -> Result<Self, Self::Error> {
        let category = AdvisorCategory::from_str(&row.category)
            .map_err(|e| DomainError::Internal(format!("corrupt advisor category in db: {e}")))?;
        let severity = AdvisorSeverity::from_str(&row.severity)
            .map_err(|e| DomainError::Internal(format!("corrupt advisor severity in db: {e}")))?;

        Ok(Self {
            id: row.id,
            name: row.name,
            engine: row.engine,
            category,
            severity,
            description: row.description,
            script_content: row.script_content,
            is_enabled: row.is_enabled,
        })
    }
}

#[derive(FromRow)]
struct AdvisorFindingRow {
    id: Uuid,
    rule_id: String,
    rule_name: String,
    cluster_id: Uuid,
    node_id: Option<Uuid>,
    severity: String,
    summary: String,
    recommendation: String,
    detected_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

impl TryFrom<AdvisorFindingRow> for AdvisorFinding {
    type Error = DomainError;

    fn try_from(row: AdvisorFindingRow) -> Result<Self, Self::Error> {
        let severity = AdvisorSeverity::from_str(&row.severity)
            .map_err(|e| DomainError::Internal(format!("corrupt severity in finding row: {e}")))?;

        Ok(Self {
            id: row.id,
            rule_id: row.rule_id,
            rule_name: row.rule_name,
            cluster_id: row.cluster_id,
            node_id: row.node_id,
            severity,
            summary: row.summary,
            recommendation: row.recommendation,
            detected_at: row.detected_at,
            resolved_at: row.resolved_at,
        })
    }
}

#[derive(FromRow)]
struct QueryDigestRow {
    query_hash: String,
    cluster_id: Uuid,
    node_id: Option<Uuid>,
    database_name: String,
    digest_text: String,
    execution_count: i64,
    total_latency_ms: f64,
    mean_latency_ms: f64,
    p95_latency_ms: f64,
    max_latency_ms: f64,
    rows_examined: i64,
    rows_sent: i64,
    last_seen: DateTime<Utc>,
}

impl From<QueryDigestRow> for QueryDigest {
    fn from(row: QueryDigestRow) -> Self {
        Self {
            query_hash: row.query_hash,
            cluster_id: row.cluster_id,
            node_id: row.node_id,
            database_name: row.database_name,
            digest_text: row.digest_text,
            execution_count: row.execution_count.max(0) as u64,
            total_latency_ms: row.total_latency_ms,
            mean_latency_ms: row.mean_latency_ms,
            p95_latency_ms: row.p95_latency_ms,
            max_latency_ms: row.max_latency_ms,
            rows_examined: row.rows_examined.max(0) as u64,
            rows_sent: row.rows_sent.max(0) as u64,
            last_seen: row.last_seen,
        }
    }
}

impl AdvisorRepository for PgAdvisorRepository {
    async fn list_rules(
        &self,
        engine: Option<&str>,
        category: Option<AdvisorCategory>,
    ) -> Result<Vec<AdvisorRule>, DomainError> {
        let category_str = category.map(|c| c.as_str());

        let rows = sqlx::query_as::<_, AdvisorRuleRow>(
            r#"
            SELECT id, name, engine, category, severity, description, script_content, is_enabled
            FROM advisor_rules
            WHERE ($1::text IS NULL OR engine = $1 OR engine = 'any')
              AND ($2::text IS NULL OR category = $2)
            ORDER BY id ASC
            "#,
        )
        .bind(engine)
        .bind(category_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list advisor rules"))?;

        rows.into_iter().map(AdvisorRule::try_from).collect()
    }

    async fn get_rule_by_id(&self, id: &str) -> Result<Option<AdvisorRule>, DomainError> {
        let opt = sqlx::query_as::<_, AdvisorRuleRow>(
            r#"
            SELECT id, name, engine, category, severity, description, script_content, is_enabled
            FROM advisor_rules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get advisor rule by id"))?;

        match opt {
            Some(row) => Ok(Some(AdvisorRule::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn create_or_update_rule(&self, rule: &AdvisorRule) -> Result<AdvisorRule, DomainError> {
        rule.validate()?;

        let category_str = rule.category.as_str();
        let severity_str = rule.severity.as_str();

        let row = sqlx::query_as::<_, AdvisorRuleRow>(
            r#"
            INSERT INTO advisor_rules (
                id, name, engine, category, severity, description, script_content, is_enabled
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                engine = EXCLUDED.engine,
                category = EXCLUDED.category,
                severity = EXCLUDED.severity,
                description = EXCLUDED.description,
                script_content = EXCLUDED.script_content,
                is_enabled = EXCLUDED.is_enabled
            RETURNING id, name, engine, category, severity, description, script_content, is_enabled
            "#,
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.engine)
        .bind(category_str)
        .bind(severity_str)
        .bind(&rule.description)
        .bind(&rule.script_content)
        .bind(rule.is_enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "upsert advisor rule"))?;

        AdvisorRule::try_from(row)
    }

    async fn record_findings(
        &self,
        findings: &[AdvisorFinding],
    ) -> Result<Vec<AdvisorFinding>, DomainError> {
        if findings.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved = Vec::with_capacity(findings.len());

        for finding in findings {
            let severity_str = finding.severity.as_str();
            let row = sqlx::query_as::<_, AdvisorFindingRow>(
                r#"
                INSERT INTO advisor_findings (
                    id, rule_id, rule_name, cluster_id, node_id, severity, summary, recommendation, detected_at, resolved_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING id, rule_id, rule_name, cluster_id, node_id, severity, summary, recommendation, detected_at, resolved_at
                "#,
            )
            .bind(finding.id)
            .bind(&finding.rule_id)
            .bind(&finding.rule_name)
            .bind(finding.cluster_id)
            .bind(finding.node_id)
            .bind(severity_str)
            .bind(&finding.summary)
            .bind(&finding.recommendation)
            .bind(finding.detected_at)
            .bind(finding.resolved_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "record advisor finding"))?;

            saved.push(AdvisorFinding::try_from(row)?);
        }

        Ok(saved)
    }

    async fn list_findings_by_cluster(
        &self,
        cluster_id: Uuid,
        only_unresolved: bool,
    ) -> Result<Vec<AdvisorFinding>, DomainError> {
        let rows = sqlx::query_as::<_, AdvisorFindingRow>(
            r#"
            SELECT id, rule_id, rule_name, cluster_id, node_id, severity, summary, recommendation, detected_at, resolved_at
            FROM advisor_findings
            WHERE cluster_id = $1
              AND ($2 = FALSE OR resolved_at IS NULL)
            ORDER BY detected_at DESC
            "#,
        )
        .bind(cluster_id)
        .bind(only_unresolved)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list findings by cluster"))?;

        rows.into_iter().map(AdvisorFinding::try_from).collect()
    }

    async fn resolve_finding(&self, id: Uuid) -> Result<AdvisorFinding, DomainError> {
        let row = sqlx::query_as::<_, AdvisorFindingRow>(
            r#"
            UPDATE advisor_findings
            SET resolved_at = NOW()
            WHERE id = $1
            RETURNING id, rule_id, rule_name, cluster_id, node_id, severity, summary, recommendation, detected_at, resolved_at
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "resolve advisor finding"))?
        .ok_or_else(|| DomainError::NotFound(format!("advisor finding '{id}' not found")))?;

        AdvisorFinding::try_from(row)
    }

    async fn record_query_digests(&self, digests: &[QueryDigest]) -> Result<(), DomainError> {
        for q in digests {
            let exec_count = q.execution_count as i64;
            let rows_examined = q.rows_examined as i64;
            let rows_sent = q.rows_sent as i64;

            sqlx::query(
                r#"
                INSERT INTO query_digests (
                    query_hash, cluster_id, node_id, database_name, digest_text,
                    execution_count, total_latency_ms, mean_latency_ms, p95_latency_ms,
                    max_latency_ms, rows_examined, rows_sent, last_seen
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (cluster_id, query_hash) DO UPDATE SET
                    execution_count = query_digests.execution_count + EXCLUDED.execution_count,
                    total_latency_ms = query_digests.total_latency_ms + EXCLUDED.total_latency_ms,
                    mean_latency_ms = (query_digests.total_latency_ms + EXCLUDED.total_latency_ms) / (query_digests.execution_count + EXCLUDED.execution_count),
                    p95_latency_ms = GREATEST(query_digests.p95_latency_ms, EXCLUDED.p95_latency_ms),
                    max_latency_ms = GREATEST(query_digests.max_latency_ms, EXCLUDED.max_latency_ms),
                    rows_examined = query_digests.rows_examined + EXCLUDED.rows_examined,
                    rows_sent = query_digests.rows_sent + EXCLUDED.rows_sent,
                    last_seen = EXCLUDED.last_seen
                "#,
            )
            .bind(&q.query_hash)
            .bind(q.cluster_id)
            .bind(q.node_id)
            .bind(&q.database_name)
            .bind(&q.digest_text)
            .bind(exec_count)
            .bind(q.total_latency_ms)
            .bind(q.mean_latency_ms)
            .bind(q.p95_latency_ms)
            .bind(q.max_latency_ms)
            .bind(rows_examined)
            .bind(rows_sent)
            .bind(q.last_seen)
            .execute(&self.pool)
            .await
            .map_err(|e| map_sqlx_error(e, "record query digest"))?;
        }

        Ok(())
    }

    async fn list_slow_queries_by_cluster(
        &self,
        cluster_id: Uuid,
        limit: u32,
    ) -> Result<Vec<QueryDigest>, DomainError> {
        let limit = limit.min(100) as i64;

        let rows = sqlx::query_as::<_, QueryDigestRow>(
            r#"
            SELECT query_hash, cluster_id, node_id, database_name, digest_text,
                   execution_count, total_latency_ms, mean_latency_ms, p95_latency_ms,
                   max_latency_ms, rows_examined, rows_sent, last_seen
            FROM query_digests
            WHERE cluster_id = $1
            ORDER BY mean_latency_ms DESC
            LIMIT $2
            "#,
        )
        .bind(cluster_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list slow queries by cluster"))?;

        Ok(rows.into_iter().map(QueryDigest::from).collect())
    }
}
