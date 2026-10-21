//! Error conversions between SQLx and DomainError.

use nodlance_domain::DomainError;

pub fn map_sqlx_error(err: sqlx::Error, context: &str) -> DomainError {
    match err {
        sqlx::Error::RowNotFound => DomainError::NotFound(context.to_string()),
        sqlx::Error::Database(db_err) => {
            if let Some(code) = db_err.code() {
                // Postgres error codes
                if code == "23505" {
                    // unique_violation
                    return DomainError::Conflict(format!("{}: {}", context, db_err.message()));
                }
                if code == "23503" {
                    // foreign_key_violation
                    return DomainError::Validation(format!("{}: {}", context, db_err.message()));
                }
            }
            DomainError::Internal(format!("{}: {}", context, db_err.message()))
        }
        other => DomainError::Internal(format!("{}: {}", context, other)),
    }
}
