use crate::{ConfigError, ConfigLocation, ConfigType};
use std::{fs::read_to_string, io::Write};

/// Loads a config file from the config, cache, cwd, or local data directory of the current user. In `yaml` format.
///
/// It will load a config file, deserialize it and return it.
///
/// If the flag `reset_conf_on_err` is set to `true`, the config file will be reset to the default config if
/// the deserialization fails, if set to `false` an error will be returned.
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.
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
/// let config = binconf::load_yaml::<TestConfig>("test-binconf-read-yaml", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
pub fn load_yaml<'a, T>(
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
        ConfigType::Yaml.as_str(),
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = T::default();
        let yaml_str = serde_yaml::to_string(&default_config)?;
        crate::save_config_str(&config_file_path, &yaml_str)?;
        Ok(default_config)
    };

    if !config_file_path.try_exists()? {
        return save_default_conf();
    }

    let yaml_str = read_to_string(&config_file_path)?;
    let config = match serde_yaml::from_str::<T>(&yaml_str) {
        Ok(config) => config,
        Err(err) => {
            if reset_conf_on_err {
                return save_default_conf();
            }
            return Err(err.into());
        }
    };

    Ok(config)
}

/// Stores a config file in the config, cache, cwd, or local data directory of the current user. In `yaml` format.
///
/// It will store a config file, serializing it with the `serde_yaml` crate.
///
/// # Errors
///
/// This function will return an error if the config, cache or local data directory could not be found or created, or if something went wrong while serializing the config.
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
///  test: String::from("test-yaml"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store_yaml("test-binconf-store-yaml", None, Config, &test_config).unwrap();
///
/// let config = binconf::load_yaml::<TestConfig>("test-binconf-store-yaml", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
pub fn store_yaml<'a, T>(
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
        ConfigType::Yaml.as_str(),
        location.as_ref(),
    )?;

    let mut file = std::io::BufWriter::new(std::fs::File::create(config_file_path)?);

    let yaml_str = serde_yaml::to_string(&data)?;

    file.write_all(yaml_str.as_bytes())?;

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
    fn read_default_config_yaml() {
        let config = load_yaml::<TestConfig>(
            "test-binconf-read_default_config-string-yaml",
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

        let config: TestConfig = load_yaml(
            "test-binconf-read_default_config-struct-yaml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_yaml(
            "test-binconf-read_default_config-struct-yaml",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_yaml(
            "test-binconf-read_default_config-struct-yaml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name_yaml() {
        let config = load_yaml::<TestConfig>(
            "test-binconf-config_with_name-string-yaml",
            Some("test-config.yml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load_yaml(
            "test-binconf-config_with_name-struct-yaml",
            Some("test-config.yml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_yaml(
            "test-binconf-config_with_name-struct-yaml",
            Some("test-config.yml"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_yaml(
            "test-binconf-config_with_name-struct-yaml",
            Some("test-config.yml"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config_yaml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_yaml(
            "test-binconf-returns_error_on_invalid_config-yaml",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_yaml::<String>(
            "test-binconf-returns_error_on_invalid_config-yaml",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config_yaml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_yaml(
            "test-binconf-save_config_user_config-yaml",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_yaml(
            "test-binconf-save_config_user_config-yaml",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache_yaml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_yaml(
            "test-binconf-save_config_user_cache-yaml",
            None,
            Cache,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_yaml(
            "test-binconf-save_config_user_cache-yaml",
            None,
            Cache,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data_yaml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_yaml(
            "test-binconf-save_config_user_local_data-yaml",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_yaml(
            "test-binconf-save_config_user_local_data-yaml",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cwd_yaml() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_yaml("test-binconf-save_config_user_cwd-yaml", None, Cwd, &data).unwrap();
        let config: TestConfig =
            load_yaml("test-binconf-save_config_user_cwd-yaml", None, Cwd, false).unwrap();
        assert_eq!(config, data);
    }
}
