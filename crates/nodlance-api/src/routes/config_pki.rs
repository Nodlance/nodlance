//! Route handlers for Centralized Configuration Management & Built-in mTLS PKI CA.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{Duration, Utc};
use nodlance_domain::{
    AuditRepository, CertType, ConfigDiff, ConfigTemplate, ConfigurationRepository, DomainError,
    IssuedCertificate, NewAuditEvent, Permission, PkiCertificate, PkiRepository,
    calculate_config_diff,
};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{auth::AuthUser, error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub cluster_id: Option<Uuid>,
    pub engine: Option<String>,
}

/// GET /api/v1/configs/templates
pub async fn list_templates(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<Vec<ConfigTemplate>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let templates = state
        .config_repo
        .list_templates(query.cluster_id, query.engine.as_deref())
        .await?;

    Ok(Json(templates))
}

/// GET /api/v1/configs/templates/{id}
pub async fn get_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ConfigTemplate>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let template = state
        .config_repo
        .get_template_by_id(id)
        .await?
        .ok_or_else(|| DomainError::NotFound(format!("template {id}")))?;

    Ok(Json(template))
}

#[derive(Debug, Deserialize)]
pub struct CreateOrUpdateTemplateRequest {
    pub cluster_id: Option<Uuid>,
    pub name: String,
    pub engine: String,
    pub file_path: String,
    pub content: String,
}

/// POST /api/v1/configs/templates
pub async fn create_or_update_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateOrUpdateTemplateRequest>,
) -> Result<(StatusCode, Json<ConfigTemplate>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let template = ConfigTemplate {
        id: Uuid::now_v7(),
        cluster_id: payload.cluster_id,
        name: payload.name.trim().to_string(),
        engine: payload.engine.trim().to_lowercase(),
        file_path: payload.file_path.trim().to_string(),
        content: payload.content,
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    template.validate()?;

    let saved = state
        .config_repo
        .create_or_update_template(&template)
        .await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "config_template.upsert".to_string(),
            resource_type: "config_template".to_string(),
            resource_id: Some(saved.id),
            details: Some(serde_json::json!({
                "name": saved.name,
                "engine": saved.engine,
                "version": saved.version,
            })),
            source_ip: None,
        })
        .await;

    Ok((StatusCode::CREATED, Json(saved)))
}

/// DELETE /api/v1/configs/templates/{id}
pub async fn delete_template(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    state.config_repo.delete_template(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "config_template.delete".to_string(),
            resource_type: "config_template".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({ "template_id": id })),
            source_ip: None,
        })
        .await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ConfigDiffRequest {
    pub engine: Option<String>,
    pub template_id: Option<Uuid>,
    pub current_content: Option<String>,
    pub target_content: Option<String>,
}

/// POST /api/v1/clusters/{id}/configs/diff
pub async fn calculate_diff(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(_cluster_id): Path<Uuid>,
    Json(payload): Json<ConfigDiffRequest>,
) -> Result<Json<Vec<ConfigDiff>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let target = if let Some(target_str) = payload.target_content {
        target_str
    } else if let Some(tid) = payload.template_id {
        let tmpl = state
            .config_repo
            .get_template_by_id(tid)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("template {tid}")))?;
        tmpl.content
    } else {
        return Err(ApiError::from(DomainError::Validation(
            "either target_content or template_id must be provided".to_string(),
        )));
    };

    let current = payload.current_content.unwrap_or_default();
    let engine = payload.engine.as_deref().unwrap_or("postgres");

    let diffs = calculate_config_diff(engine, &current, &target);
    Ok(Json(diffs))
}

// ---------------------------------------------------------------------------
// PKI Certificate Authority Endpoints
// ---------------------------------------------------------------------------

/// GET /api/v1/clusters/{id}/pki/certificates
pub async fn list_certificates(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
) -> Result<Json<Vec<PkiCertificate>>, ApiError> {
    auth.0.require_permission(Permission::HostRead)?;

    let certs = state
        .pki_repo
        .list_certificates_by_cluster(Some(cluster_id))
        .await?;

    Ok(Json(certs))
}

#[derive(Debug, Deserialize)]
pub struct GenerateCertRequest {
    pub common_name: String,
    pub cert_type: Option<String>,
    pub validity_days: Option<i64>,
}

/// POST /api/v1/clusters/{id}/pki/generate
pub async fn generate_certificate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(cluster_id): Path<Uuid>,
    Json(payload): Json<GenerateCertRequest>,
) -> Result<(StatusCode, Json<IssuedCertificate>), ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    let cn = payload.common_name.trim();
    if cn.is_empty() {
        return Err(ApiError::from(DomainError::Validation(
            "common_name cannot be empty".to_string(),
        )));
    }

    let cert_type = match payload.cert_type.as_deref() {
        Some(t) => t.parse::<CertType>()?,
        None => CertType::Server,
    };

    let validity_days = payload.validity_days.unwrap_or(365).max(1);

    // Generate KeyPair and Certificate with rcgen
    let key_pair = KeyPair::generate()
        .map_err(|e| DomainError::Internal(format!("failed to generate keypair: {e}")))?;

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, cn);
    dn.push(DnType::OrganizationName, "Nodlance Managed PKI");
    params.distinguished_name = dn;

    if cert_type == CertType::RootCa {
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    }

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| DomainError::Internal(format!("failed to issue self-signed cert: {e}")))?;

    let cert_pem = cert.pem();
    let priv_key_pem = key_pair.serialize_pem();

    let mut hasher = Sha256::new();
    hasher.update(cert_pem.as_bytes());
    let fingerprint = hex::encode(hasher.finalize());

    let now = Utc::now();
    let expires_at = now + Duration::days(validity_days);

    let cert_record = PkiCertificate {
        id: Uuid::now_v7(),
        cluster_id: Some(cluster_id),
        common_name: cn.to_string(),
        cert_type,
        certificate_pem: cert_pem,
        fingerprint_sha256: fingerprint,
        valid_from: now,
        expires_at,
        is_revoked: false,
        created_at: now,
    };

    let saved = state.pki_repo.store_certificate(&cert_record).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "pki.certificate_generated".to_string(),
            resource_type: "pki_certificate".to_string(),
            resource_id: Some(saved.id),
            details: Some(serde_json::json!({
                "cluster_id": cluster_id,
                "common_name": saved.common_name,
                "cert_type": saved.cert_type.as_str(),
                "fingerprint": saved.fingerprint_sha256,
            })),
            source_ip: None,
        })
        .await;

    Ok((
        StatusCode::CREATED,
        Json(IssuedCertificate {
            certificate: saved,
            private_key_pem: priv_key_pem,
        }),
    ))
}

/// POST /api/v1/pki/certificates/{id}/revoke
pub async fn revoke_certificate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    auth.0.require_permission(Permission::HostWrite)?;

    state.pki_repo.revoke_certificate(id).await?;

    let _ = state
        .audit_repo
        .record(&NewAuditEvent {
            actor_id: auth.0.user_id,
            action: "pki.certificate_revoked".to_string(),
            resource_type: "pki_certificate".to_string(),
            resource_id: Some(id),
            details: Some(serde_json::json!({ "certificate_id": id })),
            source_ip: None,
        })
        .await;

    Ok(StatusCode::NO_CONTENT)
}
