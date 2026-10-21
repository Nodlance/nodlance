//! Authentication and authorization primitives for Nodlance.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use nodlance_domain::{DomainError, Permission};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token expired")]
    TokenExpired,

    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("password hash error: {0}")]
    HashError(String),

    #[error("permission denied: requires {0}")]
    Forbidden(String),
}

impl From<AuthError> for DomainError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials
            | AuthError::TokenExpired
            | AuthError::InvalidToken(_) => DomainError::Unauthorized,
            AuthError::Forbidden(p) => DomainError::Forbidden(p),
            AuthError::HashError(msg) => DomainError::Internal(msg),
        }
    }
}

/// Hashes a plaintext password using Argon2id.
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::HashError(e.to_string()))
}

/// Verifies a plaintext password against an Argon2 hash.
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AuthError> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| AuthError::HashError(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (User ID)
    pub sub: Uuid,
    /// Username
    pub username: String,
    /// List of permission names in colon notation (e.g., "host:read")
    pub permissions: Vec<String>,
    /// Expiration timestamp (seconds since UNIX epoch)
    pub exp: usize,
    /// Issued at timestamp (seconds since UNIX epoch)
    pub iat: usize,
}

/// Generates a signed JWT for an authenticated user.
pub fn generate_token(
    user_id: Uuid,
    username: &str,
    permissions: &[Permission],
    secret: &str,
    ttl_secs: u64,
) -> Result<String, AuthError> {
    let now = Utc::now().timestamp() as usize;
    let exp = now + (ttl_secs as usize);

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        permissions: permissions.iter().map(|p| p.as_str().to_string()).collect(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::InvalidToken(e.to_string()))
}

/// Validates a JWT and extracts claims.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::InvalidToken(e.to_string()),
    })?;

    Ok(token_data.claims)
}

/// Request authorization context extracted from a valid token.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub username: String,
    pub permissions: Vec<Permission>,
}

impl AuthContext {
    pub fn from_claims(claims: Claims) -> Self {
        let permissions = claims
            .permissions
            .into_iter()
            .filter_map(|p| p.parse::<Permission>().ok())
            .collect();

        Self {
            user_id: claims.sub,
            username: claims.username,
            permissions,
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn require_permission(&self, permission: Permission) -> Result<(), AuthError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthError::Forbidden(permission.as_str().to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hashing_and_verification() {
        let password = "super-secret-password";
        let hash = hash_password(password).expect("hash should succeed");
        assert!(verify_password(password, &hash).expect("verification should succeed"));
        assert!(!verify_password("wrong-password", &hash).expect("verification should succeed"));
    }

    #[test]
    fn token_generation_and_validation() {
        let user_id = Uuid::now_v7();
        let secret = "test-secret-key-that-is-long-enough-32-bytes";
        let permissions = vec![Permission::HostRead, Permission::HostWrite];

        let token = generate_token(user_id, "admin", &permissions, secret, 3600)
            .expect("token should be generated");

        let claims = validate_token(&token, secret).expect("token should be valid");
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, "admin");

        let ctx = AuthContext::from_claims(claims);
        assert!(ctx.has_permission(Permission::HostRead));
        assert!(ctx.has_permission(Permission::HostWrite));
        assert!(!ctx.has_permission(Permission::UserWrite));

        assert!(ctx.require_permission(Permission::HostRead).is_ok());
        assert!(ctx.require_permission(Permission::UserWrite).is_err());
    }
}
