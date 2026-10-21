use std::{env, net::SocketAddr};

use nodlance_api::{AppState, build_app};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse::<u16>()?;
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nodlance:localdev@localhost:5432/nodlance".to_owned());
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-me-in-production-min-32-chars-key".to_owned());
    let admin_password =
        env::var("NODLANCE_ADMIN_PASSWORD").unwrap_or_else(|_| "adminpassword".to_owned());

    tracing::info!("connecting to PostgreSQL database");
    let pool = nodlance_db::create_pool(&database_url).await?;

    tracing::info!("running database migrations");
    nodlance_db::run_migrations(&pool).await?;

    tracing::info!("ensuring seed admin user is present");
    let admin_hash = nodlance_auth::hash_password(&admin_password)?;
    nodlance_db::seed_admin_if_needed(&pool, &admin_hash).await?;

    let state = AppState::new(pool, jwt_secret);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address).await?;

    tracing::info!(%address, "starting Nodlance API control plane");
    axum::serve(listener, build_app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
