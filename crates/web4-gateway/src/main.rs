use std::path::Path;

use web4_gateway::{app::run, config::GatewayConfig};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| GatewayConfig::default_path().to_string());
    let config = match GatewayConfig::from_file(Path::new(&config_path)) {
        Ok(config) => config,
        Err(err) => {
            tracing::error!(
                error = %err.0.message,
                code = %err.0.code,
                config_path = %config_path,
                "invalid gateway configuration"
            );
            std::process::exit(2);
        }
    };
    if let Err(err) = run(config).await {
        tracing::error!(error = %err.0.message, code = %err.0.code, "gateway terminated with error");
        std::process::exit(1);
    }
}
