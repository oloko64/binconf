use serde::{de::DeserializeOwned, Serialize};
use std::io::BufWriter;

/// Reads a config file from the config directory of the current user.
///
/// It will read a config file from the config directory of the current user, deserialize it and return it.
///
/// If the flag `reset_conf_on_err` is set to `true`, the config file will be reset to the default config if
/// the deserialization fails, if set to `false` an error will be returned.
///
/// **As the data is stored in a binary format the deserialization can return incorrect data if the type of the returned data is wrong.**
///
/// **Even with the `reset_conf_on_err` set to false it is not guaranteed to always fail if the type is wrong.**
///
/// # Example
///
/// ```
/// use binconf::read;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
/// struct TestConfig {
///    test: String,
///    test_vec: Vec<u8>,
/// }
///
/// let config: TestConfig = read("test-binconf-read", None, false).unwrap();
/// assert_eq!(config, TestConfig::default());
/// ```
///
/// # Errors
///
/// This function will return an error if the config directory could not be found or created, or if something went wrong while deserializing the config.
///
/// If the flag `reset_conf_on_err` is set to `false` and the deserialization fails, an error will be returned. If it is set to `true` the config file will be reset to the default config.
pub fn read<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    reset_conf_on_err: bool,
) -> Result<T, ConfigError>
where
    T: Default + Serialize + DeserializeOwned,
{
    let conf_dir = dirs::config_dir().ok_or(ConfigError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Config directory not found",
    )))?;

    let conf_dir = conf_dir.join(app_name.as_ref());

    if !conf_dir.try_exists().map_err(ConfigError::Io)? {
        std::fs::create_dir_all(&conf_dir).map_err(ConfigError::Io)?;
    }

    let conf_file = conf_dir.join(config_name.into().unwrap_or(app_name.as_ref()));

    if !conf_file.try_exists().map_err(ConfigError::Io)? {
        let default_config = T::default();
        let file = BufWriter::new(std::fs::File::create(&conf_file).map_err(ConfigError::Io)?);
        bincode::serialize_into(file, &default_config).map_err(ConfigError::Bincode)?;
    }

    let file = std::fs::File::open(&conf_file).map_err(ConfigError::Io)?;
    let reader = std::io::BufReader::new(file);
    let config = match bincode::deserialize_from(reader) {
        Ok(config) => config,
        Err(err) => {
            if reset_conf_on_err {
                let default_config = T::default();
                let file =
                    BufWriter::new(std::fs::File::create(&conf_file).map_err(ConfigError::Io)?);
                bincode::serialize_into(file, &default_config).map_err(ConfigError::Bincode)?;
                default_config
            } else {
                return Err(ConfigError::Bincode(err));
            }
        }
    };

    Ok(config)
}

/// Stores a config file in the config directory of the current user.
///
/// It will store a config file in the config directory of the current user. Serializing it with the `bincode` crate.
///
/// # Example
///
/// ```
/// use binconf::{store, read};
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
/// store("test-binconf-store", None, &test_config).unwrap();
///
/// let config = read::<TestConfig>("test-binconf-store", None, false).unwrap();
/// assert_eq!(config, test_config);
/// ```
///
/// # Errors
///
/// This function will return an error if the config directory could not be found or created, or if something went wrong while serializing the config.
pub fn store<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    data: T,
) -> Result<(), ConfigError>
where
    T: Serialize,
{
    let conf_dir = dirs::config_dir().ok_or(ConfigError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Config directory not found",
    )))?;

    let conf_dir = conf_dir.join(app_name.as_ref());

    if !conf_dir.try_exists().map_err(ConfigError::Io)? {
        std::fs::create_dir_all(&conf_dir).map_err(ConfigError::Io)?;
    }

    let conf_file = conf_dir.join(config_name.into().unwrap_or(app_name.as_ref()));

    let file = BufWriter::new(std::fs::File::create(conf_file).map_err(ConfigError::Io)?);
    bincode::serialize_into(file, &data).map_err(ConfigError::Bincode)?;

    Ok(())
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Bincode(bincode::Error),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "{err}"),
            ConfigError::Bincode(err) => write!(f, "{err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
    struct TestConfig {
        test: String,
        test_vec: Vec<u8>,
    }

    // #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
    // struct TestConfig2 {
    //     test: u64,
    //     test_vec: Vec<u8>,
    // }

    // #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
    // struct TestConfig3 {
    //     vals: Vec<u8>,
    //     name: String,
    // }

    #[test]
    fn read_default_config() {
        let config =
            read::<String>("test-binconf-read_default_config-string", None, false).unwrap();
        assert_eq!(config, String::from(""));

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig =
            read("test-binconf-read_default_config-struct", None, false).unwrap();
        assert_eq!(config, TestConfig::default());

        store(
            "test-binconf-read_default_config-struct",
            None,
            &test_config,
        )
        .unwrap();
        let config: TestConfig =
            read("test-binconf-read_default_config-struct", None, false).unwrap();
        assert_eq!(config, test_config);
    }

    #[test]
    fn config_with_name() {
        let config = read::<String>(
            "test-binconf-config_with_name-string",
            Some("test-config.bin"),
            false,
        )
        .unwrap();
        assert_eq!(config, String::from(""));

        let test_config = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        let config: TestConfig = read(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            false,
        )
        .unwrap();
        assert_eq!(config, TestConfig::default());

        store(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            &test_config,
        )
        .unwrap();
        let config: TestConfig = read(
            "test-binconf-config_with_name-struct",
            Some("test-config.bin"),
            false,
        )
        .unwrap();
        assert_eq!(config, test_config);
    }

    // #[test]
    // fn returns_error_on_invalid_config() {
    //     let data = TestConfig2 {
    //         test: 1,
    //         test_vec: vec![1, 2, 3, 4, 5],
    //     };

    //     store("test-binconf-returns_error_on_invalid_config", None, &data).unwrap();
    //     let config =
    //         read::<TestConfig3>("test-binconf-panics_on_invalid_config-string", None, false);

    //     assert!(config.unwrap() == TestConfig3::default());
    // }
}
