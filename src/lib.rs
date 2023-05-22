#[cfg(feature = "binary-conf")]
mod binary_conf;
#[cfg(feature = "toml-conf")]
mod toml_conf;

#[cfg(feature = "json-conf")]
mod json_conf;

#[cfg(feature = "yaml-conf")]
mod yaml_conf;

#[cfg(feature = "binary-conf")]
pub use binary_conf::{load_bin, store_bin};

#[cfg(feature = "toml-conf")]
pub use toml_conf::{load_toml, store_toml};

#[cfg(feature = "json-conf")]
pub use json_conf::{load_json, store_json};

#[cfg(feature = "yaml-conf")]
pub use yaml_conf::{load_yaml, store_yaml};

use std::path::PathBuf;

/// Prepares the path to the config file.
///
/// It will decide where to store the config file based on the `location` parameter.
///
/// If the path to the config file does not exist, it will create the path.
///
/// Returns the path to the config file with the given extension.
///
/// **The function does not guarantee that the file exists. Just that the path to the file exists.**
fn config_location(
    app_name: &str,
    config_name: Option<&str>,
    extension: &str,
    location: &ConfigLocation,
) -> Result<PathBuf, ConfigError> {
    let conf_dir = match location {
        ConfigLocation::Config => dirs::config_dir().ok_or(ConfigError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found"),
        ))?,
        ConfigLocation::Cache => dirs::cache_dir().ok_or(ConfigError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cache directory not found",
        )))?,
        ConfigLocation::LocalData => {
            dirs::data_local_dir().ok_or(ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Local data directory not found",
            )))?
        }
        ConfigLocation::Cwd => std::env::current_dir().map_err(ConfigError::Io)?,
    };

    let conf_dir = conf_dir.join(app_name);

    if !conf_dir.try_exists().map_err(ConfigError::Io)? {
        std::fs::create_dir_all(&conf_dir).map_err(ConfigError::Io)?;
    }

    let conf_file = conf_dir.join(config_name.unwrap_or(&format!("{app_name}.{extension}")));

    Ok(conf_file)
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfigLocation {
    Config,
    Cache,
    LocalData,
    Cwd,
}

impl AsRef<ConfigLocation> for ConfigLocation {
    fn as_ref(&self) -> &ConfigLocation {
        self
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),

    #[cfg(feature = "toml-conf")]
    TomlSer(toml::ser::Error),

    #[cfg(feature = "toml-conf")]
    TomlDe(toml::de::Error),

    #[cfg(feature = "json-conf")]
    Json(serde_json::Error),

    #[cfg(feature = "yaml-conf")]
    Yaml(serde_yaml::Error),

    #[cfg(feature = "binary-conf")]
    Bincode(bincode::Error),

    HashMismatch,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{err}"),

            #[cfg(feature = "binary-conf")]
            ConfigError::Bincode(err) => write!(f, "{err}"),

            #[cfg(feature = "toml-conf")]
            ConfigError::TomlSer(err) => write!(f, "{err}"),

            #[cfg(feature = "toml-conf")]
            ConfigError::TomlDe(err) => write!(f, "{err}"),

            #[cfg(feature = "json-conf")]
            ConfigError::Json(err) => write!(f, "{err}"),

            #[cfg(feature = "yaml-conf")]
            ConfigError::Yaml(err) => write!(f, "{err}"),

            ConfigError::HashMismatch => write!(f, "Hash mismatch"),
        }
    }
}
