//! PostgreSQL implementations of ConfigurationRepository and PkiRepository.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use nodlance_domain::{
    CertType, ConfigTemplate, ConfigurationRepository, DomainError, PkiCertificate, PkiRepository,
};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::map_sqlx_error;

// ============================================================================
// Configuration Template Repository
// ============================================================================

#[derive(Clone)]
pub struct PgConfigurationRepository {
    pool: PgPool,
}

impl PgConfigurationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ConfigTemplateRow {
    id: Uuid,
    cluster_id: Option<Uuid>,
    name: String,
    engine: String,
    file_path: String,
    content: String,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ConfigTemplateRow> for ConfigTemplate {
    fn from(row: ConfigTemplateRow) -> Self {
        Self {
            id: row.id,
            cluster_id: row.cluster_id,
            name: row.name,
            engine: row.engine,
            file_path: row.file_path,
            content: row.content,
            version: row.version.max(1) as u32,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl ConfigurationRepository for PgConfigurationRepository {
    async fn create_or_update_template(
        &self,
        template: &ConfigTemplate,
    ) -> Result<ConfigTemplate, DomainError> {
        template.validate()?;

        let row = sqlx::query_as::<_, ConfigTemplateRow>(
            r#"
            INSERT INTO config_templates (
                id, cluster_id, name, engine, file_path, content, version, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                cluster_id = EXCLUDED.cluster_id,
                engine = EXCLUDED.engine,
                file_path = EXCLUDED.file_path,
                content = EXCLUDED.content,
                version = config_templates.version + 1,
                updated_at = NOW()
            RETURNING id, cluster_id, name, engine, file_path, content, version, created_at, updated_at
            "#,
        )
        .bind(template.id)
        .bind(template.cluster_id)
        .bind(&template.name)
        .bind(&template.engine)
        .bind(&template.file_path)
        .bind(&template.content)
        .bind(template.version as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "create or update config template"))?;

        Ok(ConfigTemplate::from(row))
    }

    async fn get_template_by_id(&self, id: Uuid) -> Result<Option<ConfigTemplate>, DomainError> {
        let opt = sqlx::query_as::<_, ConfigTemplateRow>(
            r#"
            SELECT id, cluster_id, name, engine, file_path, content, version, created_at, updated_at
            FROM config_templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get config template by id"))?;

        Ok(opt.map(ConfigTemplate::from))
    }

    async fn list_templates(
        &self,
        cluster_id: Option<Uuid>,
        engine: Option<&str>,
    ) -> Result<Vec<ConfigTemplate>, DomainError> {
        let rows = sqlx::query_as::<_, ConfigTemplateRow>(
            r#"
            SELECT id, cluster_id, name, engine, file_path, content, version, created_at, updated_at
            FROM config_templates
            WHERE ($1::uuid IS NULL OR cluster_id = $1 OR cluster_id IS NULL)
              AND ($2::text IS NULL OR engine = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(cluster_id)
        .bind(engine)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list config templates"))?;

        Ok(rows.into_iter().map(ConfigTemplate::from).collect())
    }

    async fn delete_template(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            DELETE FROM config_templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "delete config template"))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "config template '{id}' not found"
            )));
        }

        Ok(())
    }
}

// ============================================================================
// PKI Certificate Repository
// ============================================================================

#[derive(Clone)]
pub struct PgPkiRepository {
    pool: PgPool,
}

impl PgPkiRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PkiCertificateRow {
    id: Uuid,
    cluster_id: Option<Uuid>,
    common_name: String,
    cert_type: String,
    certificate_pem: String,
    fingerprint_sha256: String,
    valid_from: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    is_revoked: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<PkiCertificateRow> for PkiCertificate {
    type Error = DomainError;

    fn try_from(row: PkiCertificateRow) -> Result<Self, Self::Error> {
        let cert_type = CertType::from_str(&row.cert_type)
            .map_err(|e| DomainError::Internal(format!("corrupt cert_type in db: {e}")))?;

        Ok(Self {
            id: row.id,
            cluster_id: row.cluster_id,
            common_name: row.common_name,
            cert_type,
            certificate_pem: row.certificate_pem,
            fingerprint_sha256: row.fingerprint_sha256,
            valid_from: row.valid_from,
            expires_at: row.expires_at,
            is_revoked: row.is_revoked,
            created_at: row.created_at,
        })
    }
}

impl PkiRepository for PgPkiRepository {
    async fn store_certificate(
        &self,
        cert: &PkiCertificate,
    ) -> Result<PkiCertificate, DomainError> {
        let cert_type_str = cert.cert_type.as_str();

        let row = sqlx::query_as::<_, PkiCertificateRow>(
            r#"
            INSERT INTO pki_certificates (
                id, cluster_id, common_name, cert_type, certificate_pem, fingerprint_sha256,
                valid_from, expires_at, is_revoked, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            RETURNING id, cluster_id, common_name, cert_type, certificate_pem, fingerprint_sha256,
                      valid_from, expires_at, is_revoked, created_at
            "#,
        )
        .bind(cert.id)
        .bind(cert.cluster_id)
        .bind(&cert.common_name)
        .bind(cert_type_str)
        .bind(&cert.certificate_pem)
        .bind(&cert.fingerprint_sha256)
        .bind(cert.valid_from)
        .bind(cert.expires_at)
        .bind(cert.is_revoked)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "store pki certificate"))?;

        PkiCertificate::try_from(row)
    }

    async fn get_certificate_by_id(&self, id: Uuid) -> Result<Option<PkiCertificate>, DomainError> {
        let opt = sqlx::query_as::<_, PkiCertificateRow>(
            r#"
            SELECT id, cluster_id, common_name, cert_type, certificate_pem, fingerprint_sha256,
                   valid_from, expires_at, is_revoked, created_at
            FROM pki_certificates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "get pki certificate by id"))?;

        match opt {
            Some(row) => Ok(Some(PkiCertificate::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn list_certificates_by_cluster(
        &self,
        cluster_id: Option<Uuid>,
    ) -> Result<Vec<PkiCertificate>, DomainError> {
        let rows = sqlx::query_as::<_, PkiCertificateRow>(
            r#"
            SELECT id, cluster_id, common_name, cert_type, certificate_pem, fingerprint_sha256,
                   valid_from, expires_at, is_revoked, created_at
            FROM pki_certificates
            WHERE ($1::uuid IS NULL OR cluster_id = $1)
            ORDER BY expires_at ASC
            "#,
        )
        .bind(cluster_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "list pki certificates"))?;

        rows.into_iter().map(PkiCertificate::try_from).collect()
    }

    async fn revoke_certificate(&self, id: Uuid) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE pki_certificates
            SET is_revoked = TRUE
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| map_sqlx_error(e, "revoke certificate"))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound(format!(
                "certificate '{id}' not found"
            )));
        }

        Ok(())
    }
}
