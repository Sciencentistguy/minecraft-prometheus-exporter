use std::path::PathBuf;

use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub servers: Vec<Server>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub stats_root: PathBuf,
    pub server_name: String,
    pub server_ip: String,
    pub rcon_port: u16,
    pub rcon_password: String,
}

impl Config {
    fn new_default() -> Self {
        Self {
            port: 9001,
            servers: vec![],
        }
    }

    pub fn open_or_create() -> Result<Self> {
        let path = &crate::OPTIONS.config_file;

        trace!(?path, "Opening config file");

        if !path.is_file() {
            warn!(?path, "Config file not found. Creating a default one.");
            let file = std::fs::File::create(&path)?;
            serde_yaml::to_writer(file, &Self::new_default())?;
        }

        let config_file = std::fs::File::open(&path)?;

        let config: Self = serde_yaml::from_reader(config_file)?;
        if config.servers.is_empty() {
            warn!(
                "The config file does not define any servers.\
                This program will do nothing if no servers are defined"
            );
        }
        Ok(config)
    }
}
