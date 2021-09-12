use std::path::Path;
use std::path::PathBuf;

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
}

impl Config {
    fn new_default() -> Self {
        Self {
            port: 9001,
            servers: vec![],
        }
    }

    pub fn open_or_create() -> Self {
        let path = std::env::var("MPE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Path::new("./config.yml").to_owned());

        trace!(?path, "Opening config file");

        if !path.is_file() {
            warn!(?path, "Config file not found. Creating a default one.");
            let file = std::fs::File::create(&path).unwrap();
            serde_yaml::to_writer(file, &Self::new_default()).unwrap();
        }

        let config_file = std::fs::File::open(&path).unwrap();

        serde_yaml::from_reader(config_file).unwrap()
    }
}
