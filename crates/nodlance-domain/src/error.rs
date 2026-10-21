//! Domain error types shared across the Nodlance control plane.

use thiserror::Error;

/// Errors returned by domain operations and repository ports.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("internal: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::DomainError;

    #[test]
    fn not_found_displays_context() {
        let err = DomainError::NotFound("host abc".into());
        assert_eq!(err.to_string(), "not found: host abc");
    }

    #[test]
    fn unauthorized_has_no_extra_context() {
        let err = DomainError::Unauthorized;
        assert_eq!(err.to_string(), "unauthorized");
    }
}
