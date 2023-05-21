mod binary_conf;
mod toml_conf;

use std::path::PathBuf;

#[cfg(feature = "binary_conf")]
pub use binary_conf::{load_bin, store_bin};

#[cfg(feature = "toml_conf")]
pub use toml_conf::{load_toml, store_toml};

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
}

impl AsRef<ConfigLocation> for ConfigLocation {
    fn as_ref(&self) -> &ConfigLocation {
        self
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),

    #[cfg(feature = "toml_conf")]
    TomlSer(toml::ser::Error),

    #[cfg(feature = "toml_conf")]
    TomlDe(toml::de::Error),

    #[cfg(feature = "binary_conf")]
    Bincode(bincode::Error),
    HashMismatch,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{err}"),

            #[cfg(feature = "binary_conf")]
            ConfigError::Bincode(err) => write!(f, "{err}"),

            #[cfg(feature = "toml_conf")]
            ConfigError::TomlSer(err) => write!(f, "{err}"),

            #[cfg(feature = "toml_conf")]
            ConfigError::TomlDe(err) => write!(f, "{err}"),

            ConfigError::HashMismatch => write!(f, "Hash mismatch"),
        }
    }
}
