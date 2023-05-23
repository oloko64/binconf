use md5::{Digest, Md5};
use std::io::{Read, Write};

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
/// use binconf::ConfigLocation::{Cache, Config, LocalData, Cwd};
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
        let default_config = T::default();
        let mut file = std::io::BufWriter::new(
            std::fs::File::create(&config_file_path).map_err(ConfigError::Io)?,
        );

        let mut hasher = Md5::new();
        // create a buffer with a space of 128bits(16 bytes) for storing the hash in the front of the file
        let mut full_data = [
            vec![0; 16],
            bincode::serialize(&default_config).map_err(ConfigError::Bincode)?,
        ]
        .concat();

        // hash the data without the zeroed hash
        hasher.update(&full_data[16..]);
        let hash: &[u8] = &hasher.finalize()[..];

        // mutate the array to add the hash to the front of the buffer
        full_data[..16].clone_from_slice(hash);

        file.write_all(&full_data).map_err(ConfigError::Io)?;

        Ok(default_config)
    };

    if !config_file_path.try_exists().map_err(ConfigError::Io)? {
        return save_default_conf();
    }

    let file = std::fs::File::open(&config_file_path).map_err(ConfigError::Io)?;
    let mut reader = std::io::BufReader::new(file);

    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(ConfigError::Io)?;
    let hash_from_file = &data[..16];
    let data = &data[16..];

    let mut hasher = Md5::new();
    hasher.update(data);

    let hash_from_data: &[u8] = &hasher.finalize()[..];

    if hash_from_file != hash_from_data {
        if reset_conf_on_err {
            return save_default_conf();
        }
        return Err(ConfigError::HashMismatch);
    }

    let config: T = match bincode::deserialize_from(data) {
        Ok(config) => config,
        Err(err) => {
            if reset_conf_on_err {
                save_default_conf()?
            } else {
                return Err(ConfigError::Bincode(err));
            }
        }
    };

    Ok(config)
}

/// Stores a config file in the config, cache or local data directory of the current user.
///
/// It will store a config file, serializing it with the `bincode` crate.
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

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);

    let mut hasher = Md5::new();
    let mut full_data = [
        vec![0; 16],
        bincode::serialize(&data).map_err(ConfigError::Bincode)?,
    ]
    .concat();
    hasher.update(&full_data[16..]);

    let hash: &[u8] = &hasher.finalize()[..];

    full_data[..16].clone_from_slice(hash);

    file.write_all(&full_data[..]).map_err(ConfigError::Io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};
    use ConfigLocation::{Cache, Config, Cwd, LocalData};

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestConfig {
        test: String,
        test_vec: Vec<u8>,
    }

    #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    struct TestConfig2 {
        strings: String,
        vecs: Vec<u8>,
        num_1: i32,
        num_2: i32,
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
            test_vec: vec![1, 2],
        };

        store_bin(
            "test-binconf-returns_error_on_invalid_config-bin",
            None,
            Config,
            &data,
        )
        .unwrap();
        let config = load_bin::<TestConfig2>(
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
