use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

trait Configurable: Default + Serialize + DeserializeOwned {
    fn get_config<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        if let Some(path) = config_path.as_ref().parent() {
            fs::create_dir_all(path)?;
        }

        if !config_path.as_ref().exists() {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(config_path)?;

            let config = Self::default();
            serde_json::to_writer_pretty(&file, &config)?;
            return Ok(config);
        }

        let config_file = OpenOptions::new().read(true).open(config_path)?;
        serde_json::from_reader(&config_file).map_err(|error| anyhow::anyhow!(error))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    pub bot_name: String,
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: "".to_string(),
            bot_name: "Schedule IPT".to_string(),
            database_url: "".to_string(),
            max_connections: 5,
        }
    }
}

impl Configurable for Config {}

impl Config {
    pub fn new() -> Result<Self> {
        Self::get_config("config.json")
    }
}
