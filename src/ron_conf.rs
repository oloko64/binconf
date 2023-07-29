use crate::{ConfigError, ConfigLocation, ConfigType};
use std::{fs::read_to_string, io::Write};

/// Loads a config file from the config, cache, cwd, or local data directory of the current user. In `ron` format.
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
/// let config = binconf::load_ron::<TestConfig>("test-binconf-read-ron", None, Config, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
pub fn load_ron<'a, T>(
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
        ConfigType::Ron.as_str(),
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = T::default();
        let ser_config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .indentor("\t".to_owned());
        let ron_str = ron::ser::to_string_pretty(&default_config, ser_config)?;
        crate::save_config_str(&config_file_path, &ron_str)?;
        Ok(default_config)
    };

    if !config_file_path.try_exists()? {
        return save_default_conf();
    }

    let ron_str = read_to_string(&config_file_path)?;
    let config = match ron::from_str::<T>(&ron_str) {
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

/// Stores a config file in the config, cache, cwd, or local data directory of the current user. In `ron` format.
///
/// It will store a config file, serializing it with the `serde_ron` crate.
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
///  test: String::from("test-ron"),
///  test_vec: vec![1, 2, 3, 4, 5],
/// };
///
/// binconf::store_ron("test-binconf-store-ron", None, Config, &test_config).unwrap();
///
/// let config = binconf::load_ron::<TestConfig>("test-binconf-store-ron", None, Config, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
pub fn store_ron<'a, T>(
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
        ConfigType::Ron.as_str(),
        location.as_ref(),
    )?;

    let mut file = std::io::BufWriter::new(std::fs::File::create(config_file_path)?);

    let ser_config = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .indentor("\t".to_owned());
    let ron_str = ron::ser::to_string_pretty(&data, ser_config)?;

    file.write_all(ron_str.as_bytes())?;

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
    fn read_default_config_ron() {
        let config = load_ron::<TestConfig>(
            "test-binconf-read_default_config-string-ron",
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

        let config: TestConfig = load_ron(
            "test-binconf-read_default_config-struct-ron",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_ron(
            "test-binconf-read_default_config-struct-ron",
            None,
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_ron(
            "test-binconf-read_default_config-struct-ron",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name_ron() {
        let config = load_ron::<TestConfig>(
            "test-binconf-config_with_name-string-ron",
            Some("test-config.ron"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = load_ron(
            "test-binconf-config_with_name-struct-ron",
            Some("test-config.ron"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store_ron(
            "test-binconf-config_with_name-struct-ron",
            Some("test-config.ron"),
            Config,
            &test_config,
        )
        .unwrap();
        let config: TestConfig = load_ron(
            "test-binconf-config_with_name-struct-ron",
            Some("test-config.ron"),
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn returns_error_on_invalid_config_ron() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_ron(
            "test-binconf-returns_error_on_invalid_config-ron",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_ron::<String>(
            "test-binconf-returns_error_on_invalid_config-ron",
            None,
            Config,
            false,
        );

        assert!(config.is_err());
    }

    #[test]
    fn save_config_user_config_ron() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_ron(
            "test-binconf-save_config_user_config-ron",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_ron(
            "test-binconf-save_config_user_config-ron",
            None,
            Config,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cache_ron() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_ron(
            "test-binconf-save_config_user_cache-ron",
            None,
            Cache,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_ron(
            "test-binconf-save_config_user_cache-ron",
            None,
            Cache,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_local_data_ron() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_ron(
            "test-binconf-save_config_user_local_data-ron",
            None,
            LocalData,
            &data,
        )
        .unwrap();
        let config: TestConfig = load_ron(
            "test-binconf-save_config_user_local_data-ron",
            None,
            LocalData,
            false,
        )
        .unwrap();
        assert_eq!(config, data);
    }

    #[test]
    fn save_config_user_cwd_ron() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store_ron("test-binconf-save_config_user_cwd-ron", None, Cwd, &data).unwrap();
        let config: TestConfig =
            load_ron("test-binconf-save_config_user_cwd-ron", None, Cwd, false).unwrap();
        assert_eq!(config, data);
    }
}
