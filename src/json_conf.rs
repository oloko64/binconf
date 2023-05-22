use crate::{ConfigError, ConfigLocation};
use std::{fs::read_to_string, io::Write};

const JSON_EXTENSION: &str = "json";

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
/// use binconf::ConfigLocation::{Cache, Config, LocalData, Cwd};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///    test: String,
///    test_vec: Vec<u8>,
/// }
///
/// let config = binconf::load_json::<TestConfig>("test-binconf-read-json", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.
pub fn load_json<'a, T>(
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
        JSON_EXTENSION,
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = T::default();
        let mut file = std::io::BufWriter::new(
            std::fs::File::create(&config_file_path).map_err(ConfigError::Io)?,
        );
        let json_str = serde_json::to_string_pretty(&default_config).map_err(ConfigError::Json)?;
        file.write_all(json_str.as_bytes())
            .map_err(ConfigError::Io)?;
        Ok(default_config)
    };

    if !config_file_path.try_exists().map_err(ConfigError::Io)? {
        return save_default_conf();
    }

    let json_str = read_to_string(&config_file_path).map_err(ConfigError::Io)?;
    let config = match serde_json::from_str::<T>(&json_str).map_err(ConfigError::Json) {
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

/// Stores a config file in the config, cache or local data directory of the current user.
///
/// It will store a config file, serializing it with the `serde_json` crate.
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
///  test: String::from("test-json"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store_json("test-binconf-store-json", None, Config, &test_config).unwrap();
///
/// let config = binconf::load_json::<TestConfig>("test-binconf-store-json", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while serializing the config.
pub fn store_json<'a, T>(
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
        JSON_EXTENSION,
        location.as_ref(),
    )?;

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);

    let json_str = serde_json::to_string_pretty(&data).map_err(ConfigError::Json)?;

    file.write_all(json_str.as_bytes())
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
    fn read_default_config_json() {
        let config = load_json::<TestConfig>(
            "test-binconf-read_default_config-string-json",
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

        let config: TestConfig = load_json(
            "test-binconf-read_default_config-struct-json",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_json(
            "test-binconf-read_default_config-struct-json",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_json(
            "test-binconf-read_default_config-struct-json",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name_json() {
        let config = load_json::<TestConfig>(
            "test-binconf-config_with_name-string-json",
            Some("test-config.json"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load_json(
            "test-binconf-config_with_name-struct-json",
            Some("test-config.json"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_json(
            "test-binconf-config_with_name-struct-json",
            Some("test-config.json"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_json(
            "test-binconf-config_with_name-struct-json",
            Some("test-config.json"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config_json() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_json(
            "test-binconf-returns_error_on_invalid_config-json",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_json::<String>(
            "test-binconf-returns_error_on_invalid_config-json",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config_json() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_json(
            "test-binconf-save_config_user_config-json",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_json(
            "test-binconf-save_config_user_config-json",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache_json() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_json(
            "test-binconf-save_config_user_cache-json",
            None,
            Cache,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_json(
            "test-binconf-save_config_user_cache-json",
            None,
            Cache,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data_json() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_json(
            "test-binconf-save_config_user_local_data-json",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_json(
            "test-binconf-save_config_user_local_data-json",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cwd_json() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_json("test-binconf-save_config_user_cwd-json", None, Cwd, &data).unwrap();
        let config: TestConfig =
            load_json("test-binconf-save_config_user_cwd-json", None, Cwd, false).unwrap();
        assert_eq!(config, data);
    }
}
