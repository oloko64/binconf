use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::BufWriter,
};

/// Reads a config file from the config directory of the current user.
///
/// It will read a config file from the config directory of the current user, deserialize it and return it.
///
/// If the flag `reset_conf_on_err` is set to `true`, the config file will be reset to the default config if
/// the deserialization fails, if set to `false` an error will be returned.
///
/// # Example
///
/// ```
/// use binconf::read;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize, PartialEq, Hash, Debug)]
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
    T: Default + Serialize + DeserializeOwned + Hash,
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

    let save_default_conf = || {
        let default_config = Config::new(T::default());
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

    let mut hasher = DefaultHasher::new();
    config.data.hash(&mut hasher);
    let hash = hasher.finish();

    if config.hash != hash {
        if reset_conf_on_err {
            let default_config = save_default_conf()?;
            return Ok(default_config.data);
        }
        return Err(ConfigError::HashMismatch);
    }

    Ok(config.data)
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
/// #[derive(Default, Serialize, Deserialize, PartialEq, Debug, Hash)]
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
    T: Serialize + Hash,
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

    let config_data = Config::new(data);

    let file = BufWriter::new(std::fs::File::create(conf_file).map_err(ConfigError::Io)?);
    bincode::serialize_into(file, &config_data).map_err(ConfigError::Bincode)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config<T> {
    hash: u64,
    data: T,
}

impl<T: Hash + Serialize> Config<T> {
    fn new(data: T) -> Config<T> {
        let hash = {
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            hasher.finish()
        };
        Config { hash, data }
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

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug, Hash, Clone)]
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

    #[test]
    fn returns_error_on_invalid_config() {
        let data = TestConfig {
            test: String::from("test"),
            test_vec: vec![1, 2, 3, 4, 5],
        };

        store("test-binconf-returns_error_on_invalid_config", None, &data).unwrap();
        let config = read::<String>("test-binconf-returns_error_on_invalid_config", None, false);

        assert!(config.is_err());
    }
}
