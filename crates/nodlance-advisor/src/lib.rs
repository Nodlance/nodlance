//! Scriptable DBA heuristics and operational rule evaluation engine using embedded Rhai.

use std::sync::Arc;

use chrono::Utc;
use nodlance_domain::{AdvisorCategory, AdvisorFinding, AdvisorRule, AdvisorSeverity, DomainError};
use rhai::{Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Telemetry and configuration snapshot passed to Rhai rules for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTelemetryContext {
    pub cluster_id: Uuid,
    pub cluster_name: String,
    pub engine: String, // 'postgres', 'mysql', 'galera'
    pub buffer_pool_hit_ratio: f64,
    pub active_connections: i64,
    pub max_connections: i64,
    pub unindexed_foreign_keys_count: i64,
    pub missing_primary_keys_count: i64,
    pub dead_tuples_ratio: f64,
    pub oldest_txid_age: i64,
    pub tls_enabled: bool,
    pub super_privilege_non_localhost: bool,
    pub galera_flow_control_paused_pct: f64,
    pub replication_lag_ms: i64,
}

impl Default for ClusterTelemetryContext {
    fn default() -> Self {
        Self {
            cluster_id: Uuid::nil(),
            cluster_name: "Default Cluster".to_string(),
            engine: "postgres".to_string(),
            buffer_pool_hit_ratio: 0.99,
            active_connections: 45,
            max_connections: 500,
            unindexed_foreign_keys_count: 0,
            missing_primary_keys_count: 0,
            dead_tuples_ratio: 0.02,
            oldest_txid_age: 10_000,
            tls_enabled: true,
            super_privilege_non_localhost: false,
            galera_flow_control_paused_pct: 0.0,
            replication_lag_ms: 2,
        }
    }
}

/// DBA Heuristics Evaluation Engine powered by sandboxed Rhai scripts.
#[derive(Clone)]
pub struct AdvisorEvaluator {
    engine: Arc<Engine>,
}

impl AdvisorEvaluator {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        // Sandbox limits to prevent runaway scripts / denial of service
        engine.set_max_operations(10_000);
        engine.set_max_call_levels(16);
        engine.set_max_string_size(4_096);

        Self {
            engine: Arc::new(engine),
        }
    }

    /// Evaluates a single rule script against the telemetry context.
    /// Returns an `AdvisorFinding` if the rule triggers an anomaly.
    pub fn evaluate_rule(
        &self,
        rule: &AdvisorRule,
        ctx: &ClusterTelemetryContext,
    ) -> Result<Option<AdvisorFinding>, DomainError> {
        if !rule.is_enabled {
            return Ok(None);
        }

        // Check if rule applies to this engine
        if rule.engine != "any" && rule.engine != ctx.engine {
            return Ok(None);
        }

        let mut scope = Scope::new();
        scope.push("cluster_id", ctx.cluster_id.to_string());
        scope.push("cluster_name", ctx.cluster_name.clone());
        scope.push("engine", ctx.engine.clone());
        scope.push("buffer_pool_hit_ratio", ctx.buffer_pool_hit_ratio);
        scope.push("active_connections", ctx.active_connections);
        scope.push("max_connections", ctx.max_connections);
        scope.push(
            "unindexed_foreign_keys_count",
            ctx.unindexed_foreign_keys_count,
        );
        scope.push("missing_primary_keys_count", ctx.missing_primary_keys_count);
        scope.push("dead_tuples_ratio", ctx.dead_tuples_ratio);
        scope.push("oldest_txid_age", ctx.oldest_txid_age);
        scope.push("tls_enabled", ctx.tls_enabled);
        scope.push(
            "super_privilege_non_localhost",
            ctx.super_privilege_non_localhost,
        );
        scope.push(
            "galera_flow_control_paused_pct",
            ctx.galera_flow_control_paused_pct,
        );
        scope.push("replication_lag_ms", ctx.replication_lag_ms);

        // Evaluate Rhai script. The script should return a boolean: true if anomaly detected.
        let result: Result<Dynamic, _> = self
            .engine
            .eval_with_scope(&mut scope, &rule.script_content);

        match result {
            Ok(val) => {
                let triggered = val.as_bool().unwrap_or(false);
                if triggered {
                    Ok(Some(AdvisorFinding {
                        id: Uuid::now_v7(),
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        cluster_id: ctx.cluster_id,
                        node_id: None,
                        severity: rule.severity,
                        summary: format!("Advisor triggered: {}", rule.name),
                        recommendation: rule.description.clone(),
                        detected_at: Utc::now(),
                        resolved_at: None,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                tracing::warn!(rule_id = %rule.id, error = %e, "advisor script execution failed");
                Err(DomainError::Validation(format!(
                    "script error in rule '{}': {e}",
                    rule.id
                )))
            }
        }
    }

    /// Evaluates a collection of rules against the cluster telemetry.
    pub fn evaluate_all(
        &self,
        rules: &[AdvisorRule],
        ctx: &ClusterTelemetryContext,
    ) -> Vec<AdvisorFinding> {
        let mut findings = Vec::new();
        for rule in rules {
            if let Ok(Some(finding)) = self.evaluate_rule(rule, ctx) {
                findings.push(finding);
            }
        }
        findings
    }

    /// Returns standard default out-of-the-box ClusterControl parity DBA rules.
    pub fn default_rules() -> Vec<AdvisorRule> {
        vec![
            AdvisorRule {
                id: "rule_innodb_buffer_pool".to_string(),
                name: "Buffer Pool Cache Hit Ratio Low".to_string(),
                engine: "mysql".to_string(),
                category: AdvisorCategory::Performance,
                severity: AdvisorSeverity::Warning,
                description: "InnoDB buffer pool hit ratio is below 95%. Consider increasing innodb_buffer_pool_size to reduce disk read IOPS.".to_string(),
                script_content: "buffer_pool_hit_ratio < 0.95".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_connection_ceiling".to_string(),
                name: "High Connection Saturation (>80%)".to_string(),
                engine: "any".to_string(),
                category: AdvisorCategory::Capacity,
                severity: AdvisorSeverity::Critical,
                description: "Active connections exceed 80% of max_connections limit. Risk of connection starvation. Deploy PgBouncer or ProxySQL connection pooling.".to_string(),
                script_content: "(active_connections * 100) / max_connections > 80".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_autovacuum_bloat".to_string(),
                name: "Table Bloat & Autovacuum Lag".to_string(),
                engine: "postgres".to_string(),
                category: AdvisorCategory::Capacity,
                severity: AdvisorSeverity::Warning,
                description: "Dead tuples ratio exceeds 15%. Autovacuum workers may be lagging. Check autovacuum_vacuum_scale_factor and autovacuum_max_workers.".to_string(),
                script_content: "dead_tuples_ratio > 0.15".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_missing_primary_keys".to_string(),
                name: "Tables Without Primary Keys".to_string(),
                engine: "any".to_string(),
                category: AdvisorCategory::Availability,
                severity: AdvisorSeverity::Critical,
                description: "Discovered tables lacking primary keys. This breaks row-based replication consistency and Galera multi-primary replication certification.".to_string(),
                script_content: "missing_primary_keys_count > 0".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_non_tls_listener".to_string(),
                name: "Unencrypted Plaintext Database Traffic".to_string(),
                engine: "any".to_string(),
                category: AdvisorCategory::Security,
                severity: AdvisorSeverity::Critical,
                description: "Cluster is operating without enforced TLS/SSL encryption. Database passwords and sensitive data are vulnerable to network snooping. Enable mTLS.".to_string(),
                script_content: "!tls_enabled".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_galera_flow_control".to_string(),
                name: "Galera Cluster Flow Control Paused Stalling".to_string(),
                engine: "galera".to_string(),
                category: AdvisorCategory::Availability,
                severity: AdvisorSeverity::Critical,
                description: "wsrep_flow_control_paused exceeds 5%. One or more Galera nodes cannot keep up with write rate, causing replication pauses across the entire cluster.".to_string(),
                script_content: "galera_flow_control_paused_pct > 5.0".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_replication_lag".to_string(),
                name: "Replica Lag Exceeds Operational SLA".to_string(),
                engine: "any".to_string(),
                category: AdvisorCategory::Availability,
                severity: AdvisorSeverity::Warning,
                description: "Replication lag to follower replicas exceeds 1000ms. Reads routed to replicas may return stale transactions. Investigate disk IOPS on replica nodes.".to_string(),
                script_content: "replication_lag_ms > 1000".to_string(),
                is_enabled: true,
            },
            AdvisorRule {
                id: "rule_super_privilege_exposure".to_string(),
                name: "Untrusted Superuser Privilege Exposure".to_string(),
                engine: "any".to_string(),
                category: AdvisorCategory::Security,
                severity: AdvisorSeverity::Critical,
                description: "Non-localhost database accounts have been granted superuser/SUPER privileges. Limit administrative access to dedicated management bastions.".to_string(),
                script_content: "super_privilege_non_localhost".to_string(),
                is_enabled: true,
            },
        ]
    }
}

impl Default for AdvisorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advisor_rule_trigger_and_clear() {
        let evaluator = AdvisorEvaluator::new();
        let rules = AdvisorEvaluator::default_rules();

        // 1. Healthy cluster context: no findings should trigger
        let healthy = ClusterTelemetryContext {
            cluster_id: Uuid::now_v7(),
            cluster_name: "healthy-pg".to_string(),
            engine: "postgres".to_string(),
            buffer_pool_hit_ratio: 0.99,
            active_connections: 50,
            max_connections: 500,
            unindexed_foreign_keys_count: 0,
            missing_primary_keys_count: 0,
            dead_tuples_ratio: 0.05,
            oldest_txid_age: 10_000,
            tls_enabled: true,
            super_privilege_non_localhost: false,
            galera_flow_control_paused_pct: 0.0,
            replication_lag_ms: 1,
        };

        let findings = evaluator.evaluate_all(&rules, &healthy);
        assert!(
            findings.is_empty(),
            "Healthy cluster should have 0 findings"
        );

        // 2. Unhealthy cluster with connection saturation and no TLS
        let unhealthy = ClusterTelemetryContext {
            active_connections: 450,
            max_connections: 500, // 90% saturated
            tls_enabled: false,
            dead_tuples_ratio: 0.22, // 22% bloat
            missing_primary_keys_count: 2,
            ..healthy
        };

        let bad_findings = evaluator.evaluate_all(&rules, &unhealthy);
        assert_eq!(bad_findings.len(), 4);
        let rule_ids: Vec<_> = bad_findings.iter().map(|f| f.rule_id.as_str()).collect();
        assert!(rule_ids.contains(&"rule_connection_ceiling"));
        assert!(rule_ids.contains(&"rule_non_tls_listener"));
        assert!(rule_ids.contains(&"rule_autovacuum_bloat"));
        assert!(rule_ids.contains(&"rule_missing_primary_keys"));
    }

    #[test]
    fn test_galera_flow_control_advisor() {
        let evaluator = AdvisorEvaluator::new();
        let rules = AdvisorEvaluator::default_rules();

        let galera_ctx = ClusterTelemetryContext {
            cluster_id: Uuid::now_v7(),
            cluster_name: "galera-cluster".to_string(),
            engine: "galera".to_string(),
            galera_flow_control_paused_pct: 12.5, // > 5.0
            ..Default::default()
        };

        let findings = evaluator.evaluate_all(&rules, &galera_ctx);
        assert!(
            findings
                .iter()
                .any(|f| f.rule_id == "rule_galera_flow_control")
        );
    }
}
