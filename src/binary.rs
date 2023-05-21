use md5::{Digest, Md5};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::BufWriter;

use crate::ConfigLocation;

/// Reads a config file from the config, cache or local data directory of the current user.
///
/// It will load a config file, deserialize it and return it.
///
/// If the flag `reset_conf_on_err` is set to `true`, the config file will be reset to the default config if
/// the deserialization fails, if set to `false` an error will be returned.
///
/// # Example
///
/// ```
/// use binconf::ConfigLocation::{Cache, Config, LocalData};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///    test: String,
///    test_vec: Vec<u8>,
/// }
///
/// let config = binconf::load::<TestConfig>("test-binconf-read", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.
pub fn load<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    reset_conf_on_err: bool,
) -> Result<T, ConfigError>
where
    T: Default + Serialize + DeserializeOwned,
{
    let conf_dir = match location.as_ref() {
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

    let conf_dir = conf_dir.join(app_name.as_ref());

    if !conf_dir.try_exists().map_err(ConfigError::Io)? {
        std::fs::create_dir_all(&conf_dir).map_err(ConfigError::Io)?;
    }

    let conf_file = conf_dir.join(config_name.into().unwrap_or(app_name.as_ref()));

    let save_default_conf = || {
        let default_config = Config::new(T::default()).map_err(ConfigError::Bincode)?;
        let file = BufWriter::new(std::fs::File::create(&conf_file).map_err(ConfigError::Io)?);
        bincode::serialize_into(file, &default_config).map_err(ConfigError::Bincode)?;
        Ok(default_config)
    };

    if !conf_file.try_exists().map_err(ConfigError::Io)? {
        return save_default_conf().map(|config| config.data);
    }

    let file = std::fs::File::open(&conf_file).map_err(ConfigError::Io)?;
    let reader = std::io::BufReader::new(file);
    let config: Config<T> = match bincode::deserialize_from(reader) {
        Ok(config) => config,
        Err(err) => {
            if reset_conf_on_err {
                save_default_conf()?
            } else {
                return Err(ConfigError::Bincode(err));
            }
        }
    };

    let mut hasher = Md5::new();
    hasher.update(bincode::serialize(&config.data).map_err(ConfigError::Bincode)?);
    let hash = format!("{:x}", hasher.finalize());

    if config.hash != hash {
        if reset_conf_on_err {
            let default_config = save_default_conf()?;
            return Ok(default_config.data);
        }
        return Err(ConfigError::HashMismatch);
    }

    Ok(config.data)
}

/// Stores a config file in the config, cache or local data directory of the current user.
///
/// It will store a config file, serializing it with the `bincode` crate.
///
/// # Example
///
/// ```
/// use binconf::ConfigLocation::{Cache, Config, LocalData};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///   test: String,
///   test_vec: Vec<u8>,
/// }
///
/// let test_config = TestConfig {
///  test: String::from("test"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store("test-binconf-store", None, Config, &test_config).unwrap();
///
/// let config = binconf::load::<TestConfig>("test-binconf-store", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while serializing the config.
pub fn store<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    data: T,
) -> Result<(), ConfigError>
where
    T: Serialize,
{
    let conf_dir = match location.as_ref() {
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

    let conf_dir = conf_dir.join(app_name.as_ref());

    if !conf_dir.try_exists().map_err(ConfigError::Io)? {
        std::fs::create_dir_all(&conf_dir).map_err(ConfigError::Io)?;
    }

    let conf_file = conf_dir.join(config_name.into().unwrap_or(app_name.as_ref()));

    let config_data = Config::new(data).map_err(ConfigError::Bincode)?;

    let file = BufWriter::new(std::fs::File::create(conf_file).map_err(ConfigError::Io)?);
    bincode::serialize_into(file, &config_data).map_err(ConfigError::Bincode)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config<T> {
    hash: String,
    data: T,
}

impl<T: Serialize> Config<T> {
    fn new(data: T) -> Result<Config<T>, bincode::Error> {
        let mut hasher = Md5::new();
        hasher.update(bincode::serialize(&data)?);
        let hash = format!("{:x}", hasher.finalize());

        Ok(Config { hash, data })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Bincode(bincode::Error),
    HashMismatch,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{err}"),
            ConfigError::Bincode(err) => write!(f, "{err}"),
            ConfigError::HashMismatch => write!(f, "Hash mismatch"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;
    use ConfigLocation::{Cache, Config, LocalData};

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestConfig {
        test: String,
        test_vec: Vec<u8>,
    }

    #[test]
    fn read_default_config() {
        let config = load::<String>(
            "test-binconf-read_default_config-string",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, String::from(""));

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load(
            "test-binconf-read_default_config-struct",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store(
            "test-binconf-read_default_config-struct",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load(
            "test-binconf-read_default_config-struct",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name() {
        let config = load::<String>(
            "test-binconf-config_with_name-string",
            Some("test-config.bin"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, String::from(""));

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store(
            "test-binconf-returns_error_on_invalid_config",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load::<String>(
            "test-binconf-returns_error_on_invalid_config",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store("test-binconf-save_config_user_config", None, Config, &data).unwrap();
        let config: TestConfig =
            load("test-binconf-save_config_user_config", None, Config, false).unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store("test-binconf-save_config_user_cache", None, Cache, &data).unwrap();
        let config: TestConfig =
            load("test-binconf-save_config_user_cache", None, Cache, false).unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store(
            "test-binconf-save_config_user_local_data",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load(
            "test-binconf-save_config_user_local_data",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }
}
