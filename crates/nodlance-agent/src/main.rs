use std::{env, time::Duration};

use nodlance_agent::{ControlPlaneClient, MetricsCollector};
use tokio::time::interval;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let control_plane_url =
        env::var("CONTROL_PLANE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let fingerprint = env::var("NODLANCE_AGENT_FINGERPRINT")
        .unwrap_or_else(|_| "dev-fingerprint-node-1".to_string());
    let interval_secs: u64 = env::var("HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);

    tracing::info!(
        control_plane = %control_plane_url,
        fingerprint = %fingerprint,
        interval_secs,
        "starting Nodlance host agent daemon"
    );

    let client = ControlPlaneClient::new(control_plane_url, fingerprint);
    let mut collector = MetricsCollector::new();
    let mut ticker = interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let metrics = collector.collect();
                tracing::debug!(
                    cpu = %metrics.cpu_percent,
                    mem_used = %metrics.memory_used_bytes,
                    "collected host metrics, sending heartbeat"
                );

                if let Err(e) = client.send_heartbeat(Some(metrics)).await {
                    tracing::warn!(error = %e, "failed to send agent heartbeat to control plane");
                } else {
                    tracing::info!("heartbeat delivered successfully");
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down Nodlance agent");
                break;
            }
        }
    }

    Ok(())
}
