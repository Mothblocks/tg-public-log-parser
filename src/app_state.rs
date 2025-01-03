use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
}

impl AppState {
    pub fn load() -> eyre::Result<Self> {
        let mut config: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
        config.raw_logs_path = config.raw_logs_path.canonicalize()?;

        Ok(AppState { config })
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub raw_logs_path: PathBuf,
}
