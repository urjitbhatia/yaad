use config::{Config, ConfigError, File};
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub mode: String,
    pub count: Option<u16>,
    pub addr: Option<std::net::SocketAddr>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Add in the current environment file
        // Default to 'demo' env
        let env = env::var("RUN_MODE").unwrap_or("demo".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(true))?;
        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}
