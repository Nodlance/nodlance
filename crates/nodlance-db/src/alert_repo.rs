//! PostgreSQL implementation of AlertRepository.

use chrono::{DateTime, Utc};
use nodlance_domain::{
    AlertIncident, AlertRepository, AlertRule, AlertSeverity, AlertStatus, DomainError,
    MaintenanceWindow, NewMaintenanceWindow,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

#[derive(Clone)]
pub struct PgAlertRepository {
    pool: PgPool,
}

impl PgAlertRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct RuleRow {
    id: Uuid,
    name: String,
    condition: String,
    severity: String,
    is_enabled: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<RuleRow> for AlertRule {
    type Error = DomainError;

    fn try_from(row: RuleRow) -> Result<Self, Self::Error> {
        let severity = match row.severity.as_str() {
            "info" => AlertSeverity::Info,
            "warning" => AlertSeverity::Warning,
            "critical" => AlertSeverity::Critical,
            other => {
                return Err(DomainError::Internal(format!(
                    "unknown alert severity: {other}"
                )));
            }
        };

        Ok(Self {
            id: row.id,
            name: row.name,
            condition: row.condition,
            severity,
            is_enabled: row.is_enabled,
            created_at: row.created_at,
        })
    }
}

#[derive(FromRow)]
struct IncidentRow {
    id: Uuid,
    rule_id: Option<Uuid>,
    title: String,
    description: String,
    severity: String,
    status: String,
    resource_type: String,
    resource_id: Option<Uuid>,
    fired_at: DateTime<Utc>,
    acknowledged_at: Option<DateTime<Utc>>,
    resolved_at: Option<DateTime<Utc>>,
}

impl TryFrom<IncidentRow> for AlertIncident {
    type Error = DomainError;

    fn try_from(row: IncidentRow) -> Result<Self, Self::Error> {
        let severity = match row.severity.as_str() {
            "info" => AlertSeverity::Info,
            "warning" => AlertSeverity::Warning,
            "critical" => AlertSeverity::Critical,
            other => {
                return Err(DomainError::Internal(format!(
                    "unknown alert severity: {other}"
                )));
            }
        };

        let status = match row.status.as_str() {
            "firing" => AlertStatus::Firing,
            "acknowledged" => AlertStatus::Acknowledged,
            "resolved" => AlertStatus::Resolved,
            other => {
                return Err(DomainError::Internal(format!(
                    "unknown alert status: {other}"
                )));
            }
        };

        Ok(Self {
            id: row.id,
            rule_id: row.rule_id,
            title: row.title,
            description: row.description,
            severity,
            status,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            fired_at: row.fired_at,
            acknowledged_at: row.acknowledged_at,
            resolved_at: row.resolved_at,
        })
    }
}

#[derive(FromRow)]
struct MaintenanceRow {
    id: Uuid,
    title: String,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    resource_type: Option<String>,
    resource_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl From<MaintenanceRow> for MaintenanceWindow {
    fn from(row: MaintenanceRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            created_at: row.created_at,
        }
    }
}

impl AlertRepository for PgAlertRepository {
    async fn list_rules(&self) -> Result<Vec<AlertRule>, DomainError> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, condition, severity, is_enabled, created_at
            FROM alert_rules
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list alert rules"))?;

        rows.into_iter().map(AlertRule::try_from).collect()
    }

    async fn record_incident(
        &self,
        incident: &AlertIncident,
    ) -> Result<AlertIncident, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, IncidentRow>(
            r#"
            INSERT INTO alert_incidents (id, rule_id, title, description, severity, status, resource_type, resource_id, fired_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
            RETURNING id, rule_id, title, description, severity, status, resource_type, resource_id, fired_at, acknowledged_at, resolved_at
            "#,
        )
        .bind(id)
        .bind(incident.rule_id)
        .bind(&incident.title)
        .bind(&incident.description)
        .bind(incident.severity.as_str())
        .bind(incident.status.as_str())
        .bind(&incident.resource_type)
        .bind(incident.resource_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "record alert incident"))?;

        row.try_into()
    }

    async fn list_active_incidents(&self) -> Result<Vec<AlertIncident>, DomainError> {
        let rows = sqlx::query_as::<_, IncidentRow>(
            r#"
            SELECT id, rule_id, title, description, severity, status, resource_type, resource_id, fired_at, acknowledged_at, resolved_at
            FROM alert_incidents
            WHERE status IN ('firing', 'acknowledged')
            ORDER BY fired_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list active alert incidents"))?;

        rows.into_iter().map(AlertIncident::try_from).collect()
    }

    async fn acknowledge_incident(&self, incident_id: Uuid) -> Result<AlertIncident, DomainError> {
        let row = sqlx::query_as::<_, IncidentRow>(
            r#"
            UPDATE alert_incidents
            SET status = 'acknowledged', acknowledged_at = now()
            WHERE id = $1
            RETURNING id, rule_id, title, description, severity, status, resource_type, resource_id, fired_at, acknowledged_at, resolved_at
            "#,
        )
        .bind(incident_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "acknowledge incident"))?
        .ok_or_else(|| DomainError::NotFound(format!("incident not found: {incident_id}")))?;

        row.try_into()
    }

    async fn create_maintenance_window(
        &self,
        window: &NewMaintenanceWindow,
    ) -> Result<MaintenanceWindow, DomainError> {
        let id = Uuid::now_v7();

        let row = sqlx::query_as::<_, MaintenanceRow>(
            r#"
            INSERT INTO maintenance_windows (id, title, starts_at, ends_at, resource_type, resource_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            RETURNING id, title, starts_at, ends_at, resource_type, resource_id, created_at
            "#,
        )
        .bind(id)
        .bind(&window.title)
        .bind(window.starts_at)
        .bind(window.ends_at)
        .bind(window.resource_type.as_deref())
        .bind(window.resource_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create maintenance window"))?;

        Ok(row.into())
    }

    async fn is_in_maintenance(
        &self,
        resource_type: &str,
        resource_id: Option<Uuid>,
    ) -> Result<bool, DomainError> {
        #[derive(FromRow)]
        struct CountRow {
            count: Option<i64>,
        }

        let row = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT count(*) as count
            FROM maintenance_windows
            WHERE now() >= starts_at AND now() <= ends_at
              AND (
                (resource_type IS NULL AND resource_id IS NULL)
                OR (resource_type = $1 AND (resource_id IS NULL OR resource_id = $2))
              )
            "#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "check maintenance status"))?;

        Ok(row.count.unwrap_or(0) > 0)
    }
}
