//! Recovery, High Availability, fencing, and fault injection qualification crate for Nodlance.

pub mod candidate;
pub mod epoch;
pub mod fault_injection;
pub mod fencing;
pub mod quorum;

pub use candidate::{score_candidate, select_best_candidate};
pub use epoch::{next_cluster_epoch, validate_epoch_for_command};
pub use fault_injection::FaultInjectionRunner;
pub use fencing::FencingEngine;
pub use quorum::{calculate_majority_threshold, evaluate_quorum, has_split_brain_risk};
