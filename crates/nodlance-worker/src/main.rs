use std::{env, time::Duration};

use nodlance_db::{PgBackupRepository, PgClusterRepository, PgJobRepository};
use nodlance_worker::WorkerRunner;
use tokio::time::sleep;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nodlance:localdev@localhost:5432/nodlance".to_owned());
    let worker_id = env::var("WORKER_ID")
        .unwrap_or_else(|_| format!("nodlance-worker-{}", uuid::Uuid::now_v7()));

    tracing::info!(%worker_id, "starting Nodlance background worker daemon");

    let pool = nodlance_db::create_pool(&database_url).await?;
    let job_repo = PgJobRepository::new(pool.clone());
    let backup_repo = PgBackupRepository::new(pool.clone());
    let cluster_repo = PgClusterRepository::new(pool.clone());

    let runner = WorkerRunner::new(worker_id, job_repo, backup_repo, cluster_repo);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down Nodlance worker daemon");
                break;
            }
            res = runner.process_next_job() => {
                match res {
                    Ok(true) => {
                        // Immediately poll next job if work was found
                        continue;
                    }
                    Ok(false) => {
                        // Backoff sleep if no job was waiting
                        sleep(Duration::from_secs(3)).await;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "error processing job queue");
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    Ok(())
}
