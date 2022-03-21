use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

const ENV_PREFIX: &str = "POLYDRIVE_";
const ENV_SEPARATOR: &str = "_";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Config {
    /// The server configuration block
    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ServerConfig {
    /// The server address, e.g: localhost:9000
    #[serde(default)]
    pub host: String,
    /// The scheme to use for the requests, e.g: `http`.
    ///
    /// If not provided, `http` will be used by default.
    #[serde(default)]
    pub scheme: Option<String>,
}

impl Config {
    /// Load the configuration.
    ///
    /// If a file path was provided in arguments, the method will try to load the configuration from it.
    /// Otherwise, it'll take default implementation.
    ///
    /// If `override_with_env` is set to true, the loader will detect every POLYDRIVE_* environment variables and try to parse them
    /// as configuration keys. If one or more matches, the value of the variable will take precedence on any other configuration.
    pub fn load(path: Option<PathBuf>, override_with_env: Option<bool>) -> Result<Self> {
        info!("loading configuration");
        // Start with the default configuration
        // and change attributes after
        let mut config = Config::default();

        if let Some(path) = path {
            let config_file = File::open(path)?;
            config = serde_yaml::from_reader::<File, Config>(config_file)?;
        }

        if let Some(override_with_env) = override_with_env {
            if override_with_env {
                debug!("trying to detect environment variables overrides");

                let env_overrides = std::env::vars()
                    .filter(|(k, _)| k.starts_with(ENV_PREFIX))
                    .map(|(k, v)| (Self::env_to_key(&k), v))
                    .collect::<Vec<(String, String)>>();

                for (key, value) in &env_overrides {
                    match key.as_str() {
                        "server.host" => config.server.host = value.clone(),
                        "server.scheme" => config.server.scheme = Some(value.clone()),
                        _ => warn!(
                            "environment variable {} has no effect on configuration.",
                            Self::key_to_env(key)
                        ),
                    };
                }
            }
        }

        debug!("loaded configuration={:?}", &config);
        Ok(config)
    }

    /// Format the server address and return it
    pub fn get_server_address(&self) -> String {
        let scheme = if let Some(scheme) = &self.server.scheme {
            scheme
        } else {
            "http"
        };

        format!("{}://{}", scheme, self.server.host)
    }

    /// Convert an environment variable key into processable configuration key.
    ///
    /// Example:
    ///
    /// env_to_key(&String::new("POLYDRIVE_SERVER_HOST")) -> "server.host"
    pub fn env_to_key(key: &str) -> String {
        key.replace(ENV_PREFIX, "")
            .replace(ENV_SEPARATOR, ".")
            .to_lowercase()
    }

    /// Convert configuration key into an environment variable.
    ///
    /// Example:
    ///
    /// key_to_env(&String::new("server.host")) -> "POLYDRIVE_SERVER_HOST"
    pub fn key_to_env(key: &str) -> String {
        format!(
            "{}{}",
            ENV_PREFIX,
            key.to_uppercase().replace('.', ENV_SEPARATOR)
        )
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost:8090".to_string(),
            scheme: Some("http".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_env_to_key() {
        assert_eq!(
            Config::env_to_key("POLYDRIVE_SERVER_HOST"),
            "server.host".to_string()
        )
    }

    #[test]
    fn test_key_to_env() {
        assert_eq!(
            Config::key_to_env("server.host"),
            "POLYDRIVE_SERVER_HOST".to_string()
        )
    }

    #[test]
    fn test_it_load_default_configuration() {
        let configuration = Config::load(None, None).expect("failed to load configuration");
        assert_eq!(configuration, Config::default())
    }

    #[test]
    fn test_it_load_configuration_with_environment_variable_taking_precedence() {
        std::env::set_var("POLYDRIVE_SERVER_HOST", "test.com:8080");

        let configuration = Config::load(None, Some(true)).expect("failed to load configuration");

        assert_eq!(configuration.server.host, "test.com:8080");
    }

    #[test]
    fn test_it_load_configuration_from_file() {
        let tmp_dir = tempfile::tempdir().expect("failed to create temporary file");
        let tmp_path = tmp_dir.path().join("config.yml");
        let tmp_config = Config::default();

        // Write default config to file
        let mut file = File::create(&tmp_path).expect("failed to create config file");
        file.write_all(
            serde_yaml::to_string(&tmp_config)
                .expect("failed to serialize configuration")
                .as_bytes(),
        )
        .expect("failed to write config to file");

        let config =
            Config::load(Some(tmp_path), None).expect("failed to load configuration from file");
        assert_eq!(config, tmp_config)
    }
}
