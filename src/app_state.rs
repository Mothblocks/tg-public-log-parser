use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::ongoing_round_protection::{OngoingRoundProtection, OngoingRoundProtectionConfig};

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    ongoing_round_protection: OngoingRoundProtection,
}

impl AppState {
    pub async fn load() -> eyre::Result<Self> {
        let mut config: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
        config.raw_logs_path = config.raw_logs_path.canonicalize()?;

        Ok(AppState {
            ongoing_round_protection: OngoingRoundProtection::new(
                config.ongoing_round_protection.take().unwrap(),
            ),

            config,
        })
    }

    pub async fn path_is_ongoing_round(&self, path: &Path) -> eyre::Result<bool> {
        self.ongoing_round_protection
            .path_is_ongoing_round(path)
            .await
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub raw_logs_path: PathBuf,
    ongoing_round_protection: Takeable<OngoingRoundProtectionConfig>,
}

#[derive(Debug)]
struct Takeable<T> {
    value: Option<T>,
}

impl<T> Takeable<T> {
    fn take(&mut self) -> Option<T> {
        std::mem::take(&mut self.value)
    }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Takeable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            value: Some(T::deserialize(deserializer)?),
        })
    }
}
