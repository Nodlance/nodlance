//! Candidate replica evaluation and durability scoring.

use nodlance_domain::{DomainError, PromotionCandidate};

/// Calculates suitability score for a candidate replica.
/// Returns a score from 0 to 100, where 100 represents zero lag and ideal durability.
#[must_use]
pub fn score_candidate(candidate: &PromotionCandidate, max_allowed_lag_bytes: u64) -> u32 {
    if !candidate.is_durable || candidate.health != "healthy" {
        return 0;
    }

    if candidate.replication_lag_bytes > max_allowed_lag_bytes {
        return 0;
    }

    let mut score = 100u32;

    // Deduct points for replication lag (up to 40 points)
    if max_allowed_lag_bytes > 0 {
        let lag_penalty =
            ((candidate.replication_lag_bytes as f64 / max_allowed_lag_bytes as f64) * 40.0) as u32;
        score = score.saturating_sub(lag_penalty);
    }

    // Deduct points for replay latency (up to 30 points, where > 5000ms is max penalty)
    let latency_penalty = ((candidate.replay_latency_ms.min(5000) as f64 / 5000.0) * 30.0) as u32;
    score = score.saturating_sub(latency_penalty);

    score
}

/// Evaluates candidates and selects the most suitable replica for promotion.
pub fn select_best_candidate(
    candidates: &[PromotionCandidate],
    max_allowed_lag_bytes: u64,
) -> Result<PromotionCandidate, DomainError> {
    if candidates.is_empty() {
        return Err(DomainError::Validation(
            "no promotion candidates provided for evaluation".to_string(),
        ));
    }

    let mut scored: Vec<(u32, &PromotionCandidate)> = candidates
        .iter()
        .map(|c| (score_candidate(c, max_allowed_lag_bytes), c))
        .filter(|(score, _)| *score > 0)
        .collect();

    // Sort descending by score
    scored.sort_by_key(|a| std::cmp::Reverse(a.0));

    scored
        .first()
        .map(|(score, c)| {
            let mut best = (*c).clone();
            best.suitability_score = *score;
            best
        })
        .ok_or_else(|| {
            DomainError::Validation(
                "no candidate meets the durability and replication lag safety thresholds"
                    .to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_candidate_scoring_prefers_lower_lag() {
        let candidate_a = PromotionCandidate {
            host_id: Uuid::now_v7(),
            role: "replica".to_string(),
            replication_lag_bytes: 1024,
            replay_latency_ms: 50,
            is_durable: true,
            health: "healthy".to_string(),
            suitability_score: 0,
        };

        let candidate_b = PromotionCandidate {
            host_id: Uuid::now_v7(),
            role: "replica".to_string(),
            replication_lag_bytes: 1048576, // 1MB lag
            replay_latency_ms: 100,
            is_durable: true,
            health: "healthy".to_string(),
            suitability_score: 0,
        };

        let score_a = score_candidate(&candidate_a, 10485760);
        let score_b = score_candidate(&candidate_b, 10485760);

        assert!(score_a > score_b);
        assert!(score_a >= 90);
    }

    #[test]
    fn test_non_durable_or_unhealthy_candidate_scores_zero() {
        let unhealthy = PromotionCandidate {
            host_id: Uuid::now_v7(),
            role: "replica".to_string(),
            replication_lag_bytes: 0,
            replay_latency_ms: 0,
            is_durable: true,
            health: "degraded".to_string(),
            suitability_score: 0,
        };

        assert_eq!(score_candidate(&unhealthy, 1024), 0);
    }

    #[test]
    fn test_select_best_candidate() {
        let host1 = Uuid::now_v7();
        let host2 = Uuid::now_v7();

        let candidates = vec![
            PromotionCandidate {
                host_id: host1,
                role: "replica".to_string(),
                replication_lag_bytes: 5000,
                replay_latency_ms: 20,
                is_durable: true,
                health: "healthy".to_string(),
                suitability_score: 0,
            },
            PromotionCandidate {
                host_id: host2,
                role: "replica".to_string(),
                replication_lag_bytes: 200,
                replay_latency_ms: 5,
                is_durable: true,
                health: "healthy".to_string(),
                suitability_score: 0,
            },
        ];

        let best = select_best_candidate(&candidates, 10000).expect("best candidate");
        assert_eq!(best.host_id, host2);
        assert!(best.suitability_score > 90);
    }
}
