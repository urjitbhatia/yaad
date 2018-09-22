use config::{Config, ConfigError, File};
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mode: String,
    pub count: Option<u8>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Add in the current environment file
        // Default to 'consumer' env
        let env = env::var("RUN_MODE").unwrap_or("consumer".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
