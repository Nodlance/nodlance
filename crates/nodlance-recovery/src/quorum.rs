//! Coordination and majority quorum calculation.

use nodlance_domain::DomainError;

/// Calculates strict majority quorum threshold for an N-node cluster.
/// Strict majority requires strictly more than half of the configured members: `(N / 2) + 1`.
#[must_use]
pub fn calculate_majority_threshold(total_members: usize) -> usize {
    if total_members == 0 {
        return 0;
    }
    (total_members / 2) + 1
}

/// Evaluates whether the currently reachable nodes satisfy the cluster's majority quorum.
pub fn evaluate_quorum(total_members: usize, reachable_members: usize) -> Result<(), DomainError> {
    if total_members == 0 {
        return Err(DomainError::Validation(
            "cannot evaluate quorum on an empty cluster".to_string(),
        ));
    }

    let required = calculate_majority_threshold(total_members);
    if reachable_members < required {
        return Err(DomainError::Validation(format!(
            "cluster quorum lost: reachable nodes ({reachable_members}) is below required majority ({required} of {total_members})"
        )));
    }

    Ok(())
}

/// Detects whether a network partition would cause split-brain.
/// If neither partition has majority, automatic promotion must be prevented.
#[must_use]
pub fn has_split_brain_risk(
    partition_a_size: usize,
    partition_b_size: usize,
    total_nodes: usize,
) -> bool {
    let majority = calculate_majority_threshold(total_nodes);
    let a_has_quorum = partition_a_size >= majority;
    let b_has_quorum = partition_b_size >= majority;

    // If both have quorum (impossible under strict majority unless misconfigured) or neither has quorum
    !a_has_quorum && !b_has_quorum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_threshold_calculation() {
        assert_eq!(calculate_majority_threshold(1), 1);
        assert_eq!(calculate_majority_threshold(2), 2);
        assert_eq!(calculate_majority_threshold(3), 2);
        assert_eq!(calculate_majority_threshold(4), 3);
        assert_eq!(calculate_majority_threshold(5), 3);
        assert_eq!(calculate_majority_threshold(7), 4);
    }

    #[test]
    fn test_quorum_evaluation_passes_with_majority() {
        assert!(evaluate_quorum(3, 2).is_ok());
        assert!(evaluate_quorum(3, 3).is_ok());
        assert!(evaluate_quorum(5, 3).is_ok());
    }

    #[test]
    fn test_quorum_evaluation_fails_without_majority() {
        assert!(evaluate_quorum(3, 1).is_err());
        assert!(evaluate_quorum(2, 1).is_err());
        assert!(evaluate_quorum(5, 2).is_err());
    }

    #[test]
    fn test_split_brain_risk_detection() {
        // 3 nodes split 1 vs 1 (1 dead) -> neither has majority (2 needed) -> risk is true
        assert!(has_split_brain_risk(1, 1, 3));

        // 3 nodes split 2 vs 1 -> partition A has quorum (2 >= 2) -> no split-brain risk
        assert!(!has_split_brain_risk(2, 1, 3));
    }
}
