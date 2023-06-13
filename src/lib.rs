#[cfg(feature = "binary-conf")]
mod binary_conf;
#[cfg(feature = "toml-conf")]
mod toml_conf;

#[cfg(feature = "json-conf")]
mod json_conf;

#[cfg(feature = "yaml-conf")]
mod yaml_conf;

#[cfg(feature = "ron-conf")]
mod ron_conf;

#[cfg(feature = "binary-conf")]
pub use binary_conf::{load_bin, load_bin_skip_check, store_bin};

#[cfg(feature = "toml-conf")]
pub use toml_conf::{load_toml, store_toml};

#[cfg(feature = "json-conf")]
pub use json_conf::{load_json, store_json};

#[cfg(feature = "yaml-conf")]
pub use yaml_conf::{load_yaml, store_yaml};

#[cfg(feature = "ron-conf")]
pub use ron_conf::{load_ron, store_ron};

#[cfg(any(
    feature = "toml-conf",
    feature = "json-conf",
    feature = "yaml-conf",
    feature = "ron-conf"
))]
use std::io::Write;

use std::path::PathBuf;

/// Get the configuration file path used by `load` and `store` functions.
///
/// Useful to show the user where the configuration file is located or will be located. It does not check if the file exists.
///
/// # Errors
///
/// Possible errors:
/// - If the path to the config location does not exist.
/// - The user does not have permission to access it.
///
/// # Example
///
/// ```
/// use binconf::{get_configuration_path, ConfigLocation, ConfigType};
///
/// let config_path = get_configuration_path("my-app", None, ConfigType::Bin, ConfigLocation::Config).unwrap();
///
/// println!("The configuration file is located at: {}", config_path.display());
/// ```
///

pub fn get_configuration_path<'a>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    config_extension: impl AsRef<ConfigType>,
    location: impl AsRef<ConfigLocation>,
) -> Result<PathBuf, ConfigError> {
    config_location(
        app_name.as_ref(),
        config_name.into(),
        config_extension.as_ref().as_str(),
        location.as_ref(),
    )
}

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

pub enum ConfigType {
    #[cfg(feature = "toml-conf")]
    Toml,

    #[cfg(feature = "json-conf")]
    Json,

    #[cfg(feature = "yaml-conf")]
    Yaml,

    #[cfg(feature = "ron-conf")]
    Ron,

    #[cfg(feature = "binary-conf")]
    Bin,
}

impl ConfigType {
    pub fn as_str(&self) -> &str {
        match self {
            #[cfg(feature = "toml-conf")]
            ConfigType::Toml => "toml",

            #[cfg(feature = "json-conf")]
            ConfigType::Json => "json",

            #[cfg(feature = "yaml-conf")]
            ConfigType::Yaml => "yml",

            #[cfg(feature = "ron-conf")]
            ConfigType::Ron => "ron",

            #[cfg(feature = "binary-conf")]
            ConfigType::Bin => "bin",
        }
    }
}

impl AsRef<ConfigType> for ConfigType {
    fn as_ref(&self) -> &ConfigType {
        self
    }
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

/// Saves the config as a string to the given path.
#[cfg(any(
    feature = "toml-conf",
    feature = "json-conf",
    feature = "yaml-conf",
    feature = "ron-conf"
))]
#[inline]
fn save_config_str(config_file_path: &PathBuf, config_as_str: &str) -> Result<(), ConfigError> {
    let mut file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);
    file.write_all(config_as_str.as_bytes())
        .map_err(ConfigError::Io)?;

    Ok(())
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

    #[cfg(feature = "ron-conf")]
    RonSer(ron::Error),

    #[cfg(feature = "ron-conf")]
    RonDe(ron::error::SpannedError),

    #[cfg(feature = "binary-conf")]
    Bincode(bincode::Error),

    #[cfg(feature = "binary-conf")]
    HashMismatch,

    #[cfg(feature = "binary-conf")]
    CorruptedHashSector,
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

            #[cfg(feature = "ron-conf")]
            ConfigError::RonSer(err) => write!(f, "{err}"),

            #[cfg(feature = "ron-conf")]
            ConfigError::RonDe(err) => write!(f, "{err}"),

            #[cfg(feature = "binary-conf")]
            ConfigError::HashMismatch => write!(f, "Hash mismatch"),

            #[cfg(feature = "binary-conf")]
            ConfigError::CorruptedHashSector => write!(f, "Corrupted hash sector"),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "full")]
mod tests {
    use super::*;
    use dirs;

    #[test]
    fn test_get_configuration_path_config() {
        let toml_config =
            get_configuration_path("test", None, ConfigType::Toml, ConfigLocation::Config).unwrap();
        let json_config =
            get_configuration_path("test", None, ConfigType::Json, ConfigLocation::Config).unwrap();
        let yaml_config =
            get_configuration_path("test", None, ConfigType::Yaml, ConfigLocation::Config).unwrap();
        let ron_config =
            get_configuration_path("test", None, ConfigType::Ron, ConfigLocation::Config).unwrap();
        let bin_config =
            get_configuration_path("test", None, ConfigType::Bin, ConfigLocation::Config).unwrap();

        let config_location = dirs::config_dir().unwrap();

        assert_eq!(toml_config, config_location.join("test/test.toml"));
        assert_eq!(json_config, config_location.join("test/test.json"));
        assert_eq!(yaml_config, config_location.join("test/test.yml"));
        assert_eq!(ron_config, config_location.join("test/test.ron"));
        assert_eq!(bin_config, config_location.join("test/test.bin"));
    }

    #[test]
    fn test_get_configuration_path_cache() {
        let toml_config =
            get_configuration_path("test", None, ConfigType::Toml, ConfigLocation::Cache).unwrap();
        let json_config =
            get_configuration_path("test", None, ConfigType::Json, ConfigLocation::Cache).unwrap();
        let yaml_config =
            get_configuration_path("test", None, ConfigType::Yaml, ConfigLocation::Cache).unwrap();
        let ron_config =
            get_configuration_path("test", None, ConfigType::Ron, ConfigLocation::Cache).unwrap();
        let bin_config =
            get_configuration_path("test", None, ConfigType::Bin, ConfigLocation::Cache).unwrap();

        let cache_location = dirs::cache_dir().unwrap();

        assert_eq!(toml_config, cache_location.join("test/test.toml"));
        assert_eq!(json_config, cache_location.join("test/test.json"));
        assert_eq!(yaml_config, cache_location.join("test/test.yml"));
        assert_eq!(ron_config, cache_location.join("test/test.ron"));
        assert_eq!(bin_config, cache_location.join("test/test.bin"));
    }

    #[test]
    fn test_get_configuration_path_local_data() {
        let toml_config =
            get_configuration_path("test", None, ConfigType::Toml, ConfigLocation::LocalData)
                .unwrap();
        let json_config =
            get_configuration_path("test", None, ConfigType::Json, ConfigLocation::LocalData)
                .unwrap();
        let yaml_config =
            get_configuration_path("test", None, ConfigType::Yaml, ConfigLocation::LocalData)
                .unwrap();
        let ron_config =
            get_configuration_path("test", None, ConfigType::Ron, ConfigLocation::LocalData)
                .unwrap();
        let bin_config =
            get_configuration_path("test", None, ConfigType::Bin, ConfigLocation::LocalData)
                .unwrap();

        let local_data_location = dirs::data_local_dir().unwrap();

        assert_eq!(toml_config, local_data_location.join("test/test.toml"));
        assert_eq!(json_config, local_data_location.join("test/test.json"));
        assert_eq!(yaml_config, local_data_location.join("test/test.yml"));
        assert_eq!(ron_config, local_data_location.join("test/test.ron"));
        assert_eq!(bin_config, local_data_location.join("test/test.bin"));
    }

    #[test]
    fn test_get_configuration_path_cwd() {
        let toml_config =
            get_configuration_path("test", None, ConfigType::Toml, ConfigLocation::Cwd).unwrap();
        let json_config =
            get_configuration_path("test", None, ConfigType::Json, ConfigLocation::Cwd).unwrap();
        let yaml_config =
            get_configuration_path("test", None, ConfigType::Yaml, ConfigLocation::Cwd).unwrap();
        let ron_config =
            get_configuration_path("test", None, ConfigType::Ron, ConfigLocation::Cwd).unwrap();
        let bin_config =
            get_configuration_path("test", None, ConfigType::Bin, ConfigLocation::Cwd).unwrap();

        let cwd_location = std::env::current_dir().unwrap();

        assert_eq!(toml_config, cwd_location.join("test/test.toml"));
        assert_eq!(json_config, cwd_location.join("test/test.json"));
        assert_eq!(yaml_config, cwd_location.join("test/test.yml"));
        assert_eq!(ron_config, cwd_location.join("test/test.ron"));
        assert_eq!(bin_config, cwd_location.join("test/test.bin"));
    }

    #[test]
    fn test_get_configuration_path_with_custom_app_name_path_config() {
        let toml_config = get_configuration_path(
            "test",
            Some("custom.toml"),
            ConfigType::Toml,
            ConfigLocation::Config,
        )
        .unwrap();
        let json_config = get_configuration_path(
            "test",
            Some("custom.json"),
            ConfigType::Json,
            ConfigLocation::Config,
        )
        .unwrap();
        let yaml_config = get_configuration_path(
            "test",
            Some("custom.yml"),
            ConfigType::Yaml,
            ConfigLocation::Config,
        )
        .unwrap();
        let ron_config = get_configuration_path(
            "test",
            Some("custom.ron"),
            ConfigType::Ron,
            ConfigLocation::Config,
        )
        .unwrap();
        let bin_config = get_configuration_path(
            "test",
            Some("custom.bin"),
            ConfigType::Bin,
            ConfigLocation::Config,
        )
        .unwrap();

        let config_location = dirs::config_dir().unwrap();

        assert_eq!(toml_config, config_location.join("test/custom.toml"));
        assert_eq!(json_config, config_location.join("test/custom.json"));
        assert_eq!(yaml_config, config_location.join("test/custom.yml"));
        assert_eq!(ron_config, config_location.join("test/custom.ron"));
        assert_eq!(bin_config, config_location.join("test/custom.bin"));
    }

    #[test]
    fn test_get_configuration_path_with_custom_app_name_path_cache() {
        let toml_config = get_configuration_path(
            "test",
            Some("custom.toml"),
            ConfigType::Toml,
            ConfigLocation::Cache,
        )
        .unwrap();
        let json_config = get_configuration_path(
            "test",
            Some("custom.json"),
            ConfigType::Json,
            ConfigLocation::Cache,
        )
        .unwrap();
        let yaml_config = get_configuration_path(
            "test",
            Some("custom.yml"),
            ConfigType::Yaml,
            ConfigLocation::Cache,
        )
        .unwrap();
        let ron_config = get_configuration_path(
            "test",
            Some("custom.ron"),
            ConfigType::Ron,
            ConfigLocation::Cache,
        )
        .unwrap();
        let bin_config = get_configuration_path(
            "test",
            Some("custom.bin"),
            ConfigType::Bin,
            ConfigLocation::Cache,
        )
        .unwrap();

        let cache_location = dirs::cache_dir().unwrap();

        assert_eq!(toml_config, cache_location.join("test/custom.toml"));
        assert_eq!(json_config, cache_location.join("test/custom.json"));
        assert_eq!(yaml_config, cache_location.join("test/custom.yml"));
        assert_eq!(ron_config, cache_location.join("test/custom.ron"));
        assert_eq!(bin_config, cache_location.join("test/custom.bin"));
    }

    #[test]
    fn test_get_configuration_path_with_custom_app_name_path_local_data() {
        let toml_config = get_configuration_path(
            "test",
            Some("custom.toml"),
            ConfigType::Toml,
            ConfigLocation::LocalData,
        )
        .unwrap();
        let json_config = get_configuration_path(
            "test",
            Some("custom.json"),
            ConfigType::Json,
            ConfigLocation::LocalData,
        )
        .unwrap();
        let yaml_config = get_configuration_path(
            "test",
            Some("custom.yml"),
            ConfigType::Yaml,
            ConfigLocation::LocalData,
        )
        .unwrap();
        let ron_config = get_configuration_path(
            "test",
            Some("custom.ron"),
            ConfigType::Ron,
            ConfigLocation::LocalData,
        )
        .unwrap();
        let bin_config = get_configuration_path(
            "test",
            Some("custom.bin"),
            ConfigType::Bin,
            ConfigLocation::LocalData,
        )
        .unwrap();

        let local_data_location = dirs::data_local_dir().unwrap();

        assert_eq!(toml_config, local_data_location.join("test/custom.toml"));
        assert_eq!(json_config, local_data_location.join("test/custom.json"));
        assert_eq!(yaml_config, local_data_location.join("test/custom.yml"));
        assert_eq!(ron_config, local_data_location.join("test/custom.ron"));
        assert_eq!(bin_config, local_data_location.join("test/custom.bin"));
    }

    #[test]
    fn test_get_configuration_path_with_custom_app_name_path_cwd() {
        let toml_config = get_configuration_path(
            "test",
            Some("custom.toml"),
            ConfigType::Toml,
            ConfigLocation::Cwd,
        )
        .unwrap();
        let json_config = get_configuration_path(
            "test",
            Some("custom.json"),
            ConfigType::Json,
            ConfigLocation::Cwd,
        )
        .unwrap();
        let yaml_config = get_configuration_path(
            "test",
            Some("custom.yml"),
            ConfigType::Yaml,
            ConfigLocation::Cwd,
        )
        .unwrap();
        let ron_config = get_configuration_path(
            "test",
            Some("custom.ron"),
            ConfigType::Ron,
            ConfigLocation::Cwd,
        )
        .unwrap();
        let bin_config = get_configuration_path(
            "test",
            Some("custom.bin"),
            ConfigType::Bin,
            ConfigLocation::Cwd,
        )
        .unwrap();

        let cwd_location = std::env::current_dir().unwrap();

        assert_eq!(toml_config, cwd_location.join("test/custom.toml"));
        assert_eq!(json_config, cwd_location.join("test/custom.json"));
        assert_eq!(yaml_config, cwd_location.join("test/custom.yml"));
        assert_eq!(ron_config, cwd_location.join("test/custom.ron"));
        assert_eq!(bin_config, cwd_location.join("test/custom.bin"));
    }
}
