use md5::{Digest, Md5};

use crate::{ConfigError, ConfigLocation};

const BIN_EXTENSION: &str = "bin";

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
/// let config = binconf::load_bin::<TestConfig>("test-binconf-read-bin", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.

pub fn load_bin<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    reset_conf_on_err: bool,
) -> Result<T, ConfigError>
where
    T: Default + serde::Serialize + serde::de::DeserializeOwned,
{
    let config_file_path = crate::config_location(
        app_name.as_ref(),
        config_name.into(),
        BIN_EXTENSION,
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = Config::new(T::default()).map_err(ConfigError::Bincode)?;
        let file = std::io::BufWriter::new(
            std::fs::File::create(&config_file_path).map_err(ConfigError::Io)?,
        );
        bincode::serialize_into(file, &default_config).map_err(ConfigError::Bincode)?;
        Ok(default_config)
    };

    if !config_file_path.try_exists().map_err(ConfigError::Io)? {
        return save_default_conf().map(|config| config.data);
    }

    let file = std::fs::File::open(&config_file_path).map_err(ConfigError::Io)?;
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
///  test: String::from("test-bin"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store_bin("test-binconf-store-bin", None, Config, &test_config).unwrap();
///
/// let config = binconf::load_bin::<TestConfig>("test-binconf-store-bin", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while serializing the config.

pub fn store_bin<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    data: T,
) -> Result<(), ConfigError>
where
    T: serde::Serialize,
{
    let config_file_path = crate::config_location(
        app_name.as_ref(),
        config_name.into(),
        BIN_EXTENSION,
        location.as_ref(),
    )?;

    let config_data = Config::new(data).map_err(ConfigError::Bincode)?;

    let file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);
    bincode::serialize_into(file, &config_data).map_err(ConfigError::Bincode)?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Config<T> {
    hash: String,
    data: T,
}

impl<T: serde::Serialize> Config<T> {
    fn new(data: T) -> Result<Config<T>, bincode::Error> {
        let mut hasher = Md5::new();
        hasher.update(bincode::serialize(&data)?);
        let hash = format!("{:x}", hasher.finalize());

        Ok(Config { hash, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;
    use ConfigLocation::{Cache, Config, Cwd, LocalData};

    #[derive(Default, serde::Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestConfig {
        test: String,
        test_vec: Vec<u8>,
    }

    #[test]
    fn read_default_config_bin() {
        let config = load_bin::<String>(
            "test-binconf-read_default_config-string-bin",
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

        let config: TestConfig = load_bin(
            "test-binconf-read_default_config-struct-bin",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_bin(
            "test-binconf-read_default_config-struct-bin",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_bin(
            "test-binconf-read_default_config-struct-bin",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name_bin() {
        let config = load_bin::<String>(
            "test-binconf-config_with_name-string-bin",
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

        let config: TestConfig = load_bin(
            "test-binconf-config_with_name-struct-bin",
            Some("test-config.bin"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_bin(
            "test-binconf-config_with_name-struct-bin",
            Some("test-config.bin"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_bin(
            "test-binconf-config_with_name-struct-bin",
            Some("test-config.bin"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config_bin() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_bin(
            "test-binconf-returns_error_on_invalid_config-bin",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_bin::<String>(
            "test-binconf-returns_error_on_invalid_config-bin",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config_bin() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_bin(
            "test-binconf-save_config_user_config-bin",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_bin(
            "test-binconf-save_config_user_config-bin",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache_bin() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_bin(
            "test-binconf-save_config_user_cache-bin",
            None,
            Cache,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_bin(
            "test-binconf-save_config_user_cache-bin",
            None,
            Cache,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data_bin() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_bin(
            "test-binconf-save_config_user_local_data-bin",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_bin(
            "test-binconf-save_config_user_local_data-bin",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cwd_bin() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_bin("test-binconf-save_config_user_cwd-bin", None, Cwd, &data).unwrap();
        let config: TestConfig =
            load_bin("test-binconf-save_config_user_cwd-bin", None, Cwd, false).unwrap();
        assert_eq!(config, data);
    }
}
