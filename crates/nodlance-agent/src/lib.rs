//! Node agent core functionality.

pub mod client;
pub mod collector;

pub use client::ControlPlaneClient;
pub use collector::MetricsCollector;
