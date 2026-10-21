//! Cluster term/epoch monotonicity and stale-command prevention.

use nodlance_domain::DomainError;

/// Validates that an incoming control command matches or advances the current cluster term/epoch.
/// Rejects stale commands originating from partitioned or delayed controllers.
pub fn validate_epoch_for_command(
    incoming_epoch: u64,
    current_cluster_epoch: u64,
) -> Result<(), DomainError> {
    if incoming_epoch < current_cluster_epoch {
        return Err(DomainError::Validation(format!(
            "stale command rejected: incoming epoch ({incoming_epoch}) is older than active cluster epoch ({current_cluster_epoch})"
        )));
    }

    // Commands must not skip ahead by multiple uncommitted generations
    if incoming_epoch > current_cluster_epoch + 1 {
        return Err(DomainError::Validation(format!(
            "future epoch gap: incoming epoch ({incoming_epoch}) exceeds permissible transition from ({current_cluster_epoch})"
        )));
    }

    Ok(())
}

/// Computes the next monotonic epoch counter.
#[must_use]
pub fn next_cluster_epoch(current_epoch: u64) -> u64 {
    current_epoch.saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_current_and_next_epoch_accepted() {
        assert!(validate_epoch_for_command(1, 1).is_ok());
        assert!(validate_epoch_for_command(2, 1).is_ok());
        assert!(validate_epoch_for_command(5, 4).is_ok());
    }

    #[test]
    fn test_stale_epoch_rejected() {
        let err = validate_epoch_for_command(1, 2).expect_err("should reject stale epoch");
        assert!(matches!(err, DomainError::Validation(_)));
        assert!(err.to_string().contains("stale command rejected"));
    }

    #[test]
    fn test_future_gap_epoch_rejected() {
        let err = validate_epoch_for_command(5, 2).expect_err("should reject future epoch gap");
        assert!(matches!(err, DomainError::Validation(_)));
        assert!(err.to_string().contains("future epoch gap"));
    }

    #[test]
    fn test_next_cluster_epoch() {
        assert_eq!(next_cluster_epoch(0), 1);
        assert_eq!(next_cluster_epoch(42), 43);
    }
}
