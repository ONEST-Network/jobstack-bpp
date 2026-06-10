use tokio::sync::watch;

use bpp_onest_lite::{
    config::AppConfig, http::http_server::start_http_server, utils::logging::setup_logging,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = setup_logging("app/logs", "bap-adapter");
    let config = AppConfig::new()?;

    let (_shutdown_tx, shutdown_rx) = watch::channel(());

    let http_server = start_http_server(config, shutdown_rx).await?;

    tokio::select! {
        res = http_server => res??,
    }

    Ok(())
}
