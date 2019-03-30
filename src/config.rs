use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub token: String,
    pub admin_user_id: i64,
    pub commands: std::collections::HashMap<String, Command>
}

impl Config {
    pub fn from_config() -> Result<Config, Box<Error>> {
        let file = std::fs::File::open(String::from("./config.json")).unwrap();
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub script: String,
}