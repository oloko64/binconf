use crate::{ConfigError, ConfigLocation};
use std::{fs::read_to_string, io::Write};

const TOML_EXTENSION: &str = "toml";

/// Loads a config file from the config, cache, cwd, or local data directory of the current user. In `toml` format.
///
/// It will load a config file, deserialize it and return it.
///
/// If the flag `reset_conf_on_err` is set to `true`, the config file will be reset to the default config if
/// the deserialization fails, if set to `false` an error will be returned.
///
/// # Example
///
/// ```
/// use binconf::ConfigLocation::{Cache, Config, LocalData, Cwd};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///    test: String,
///    test_vec: Vec<u8>,
/// }
///
/// let config = binconf::load_toml::<TestConfig>("test-binconf-read-toml", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.
pub fn load_toml<'a, T>(
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
        TOML_EXTENSION,
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = T::default();
        let toml_str = toml::to_string_pretty(&default_config).map_err(ConfigError::TomlSer)?;
        crate::save_config_str(&config_file_path, &toml_str)?;
        Ok(default_config)
    };

    if !config_file_path.try_exists().map_err(ConfigError::Io)? {
        return save_default_conf();
    }

    let toml_str = read_to_string(&config_file_path).map_err(ConfigError::Io)?;
    let config = match toml::from_str::<T>(&toml_str).map_err(ConfigError::TomlDe) {
        Ok(config) => config,
        Err(err) => {
            if reset_conf_on_err {
                return save_default_conf();
            }
            return Err(err);
        }
    };

    Ok(config)
}

/// Stores a config file in the config, cache, cwd, or local data directory of the current user. In `toml` format.
///
/// It will store a config file, serializing it with the `toml` crate.
///
/// # Example
///
/// ```
/// use binconf::ConfigLocation::{Cache, Config, LocalData, Cwd};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///   test: String,
///   test_vec: Vec<u8>,
/// }
///
/// let test_config = TestConfig {
///  test: String::from("test-toml"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store_toml("test-binconf-store-toml", None, Config, &test_config).unwrap();
///
/// let config = binconf::load_toml::<TestConfig>("test-binconf-store-toml", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while serializing the config.
pub fn store_toml<'a, T>(
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
        TOML_EXTENSION,
        location.as_ref(),
    )?;

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);

    let toml_str = toml::to_string_pretty(&data).map_err(ConfigError::TomlSer)?;

    file.write_all(toml_str.as_bytes())
        .map_err(ConfigError::Io)?;

    Ok(())
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
    fn read_default_config_toml() {
        let config = load_toml::<TestConfig>(
            "test-binconf-read_default_config-string-toml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load_toml(
            "test-binconf-read_default_config-struct-toml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_toml(
            "test-binconf-read_default_config-struct-toml",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_toml(
            "test-binconf-read_default_config-struct-toml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name_toml() {
        let config = load_toml::<TestConfig>(
            "test-binconf-config_with_name-string-toml",
            Some("test-config.toml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load_toml(
            "test-binconf-config_with_name-struct-toml",
            Some("test-config.toml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_toml(
            "test-binconf-config_with_name-struct-toml",
            Some("test-config.toml"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_toml(
            "test-binconf-config_with_name-struct-toml",
            Some("test-config.toml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config_toml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_toml(
            "test-binconf-returns_error_on_invalid_config-toml",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_toml::<String>(
            "test-binconf-returns_error_on_invalid_config-toml",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config_toml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_toml(
            "test-binconf-save_config_user_config-toml",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_toml(
            "test-binconf-save_config_user_config-toml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache_toml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_toml(
            "test-binconf-save_config_user_cache-toml",
            None,
            Cache,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_toml(
            "test-binconf-save_config_user_cache-toml",
            None,
            Cache,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data_toml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_toml(
            "test-binconf-save_config_user_local_data-toml",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_toml(
            "test-binconf-save_config_user_local_data-toml",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cwd_toml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_toml("test-binconf-save_config_user_cwd-toml", None, Cwd, &data).unwrap();
        let config: TestConfig =
            load_toml("test-binconf-save_config_user_cwd-toml", None, Cwd, false).unwrap();
        assert_eq!(config, data);
    }
}
