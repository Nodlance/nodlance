//! HTTP client for communicating with the Nodlance control plane.

use std::time::Duration;

use nodlance_domain::{HeartbeatPayload, HostMetrics};
use reqwest::Client;

pub struct ControlPlaneClient {
    base_url: String,
    client: Client,
    fingerprint: String,
}

impl ControlPlaneClient {
    pub fn new(base_url: String, fingerprint: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            base_url,
            client,
            fingerprint,
        }
    }

    pub async fn send_heartbeat(
        &self,
        metrics: Option<HostMetrics>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v1/agent/heartbeat",
            self.base_url.trim_end_matches('/')
        );
        let payload = HeartbeatPayload {
            fingerprint: self.fingerprint.clone(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            metrics,
        };

        let res = self.client.post(&url).json(&payload).send().await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(format!("control plane responded with error {status}: {body}").into());
        }

        Ok(())
    }
}
