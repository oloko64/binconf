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

        let full_data = prepare_serialized_data(&default_config)?;
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

    // If the file is empty, or smaller than 16 bytes, we can't have a `md5` hash
    if data.len() < 16 {
        if reset_conf_on_err {
            return save_default_conf();
        }
        return Err(ConfigError::HashMismatch);
    }

    let (binary_hash_from_file, binary_hash_from_data) = get_hash_from_file_and_data(&data);

    if binary_hash_from_file != binary_hash_from_data {
        if reset_conf_on_err {
            return save_default_conf();
        }
        return Err(ConfigError::HashMismatch);
    }

    // The first 16 bytes are the `md5` hash, the rest is the serialized data
    let binary_data_without_hash = &data[16..];
    let config: T = match bincode::deserialize_from(binary_data_without_hash) {
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

    let full_data = prepare_serialized_data(data)?;

    file.write_all(&full_data[..]).map_err(ConfigError::Io)?;

    Ok(())
}

/// Returns the `md5` hash of the file and the `md5` hash of the data.
///
/// The first element of the tuple is the `md5` hash of the file, the second element is the `md5` hash of the data.
///
/// If the data is corrupted, the `md5` hash of the file and the `md5` hash of the data will not match.
fn get_hash_from_file_and_data(data: &[u8]) -> (&[u8], Vec<u8>) {
    // The first 128 bits (16 bytes) of the data will be the md5 hash of the data.
    let binary_hash_from_file = &data[..16];

    // The rest of the data will be the serialized data.
    let binary_data_without_hash = &data[16..];

    let mut hasher = Md5::new();
    hasher.update(binary_data_without_hash);

    let binary_hash_from_data: &[u8] = &hasher.finalize()[..];

    // The `md5` hash should be 128 bits (16 bytes) long. If it's not, something went wrong.
    // This prevents a vec allocation with incorrect size.
    assert!(binary_hash_from_data.len() == 16);

    (binary_hash_from_file, binary_hash_from_data.to_vec())
}

/// Prepares the data to be stored in a file.
///
/// It will calculate the `md5` hash of the data and prepend it to the data.
///
/// Returns the binary data with the hash prepended.
///
/// The first `128 bits (16 bytes)` of the data will be the `md5` hash of the data, the rest of the data will be the serialized data.
fn prepare_serialized_data<T>(data: T) -> Result<Vec<u8>, ConfigError>
where
    T: serde::Serialize,
{
    let mut hasher = Md5::new();
    // Create a buffer with 16 bytes zeroed out, and append the serialized data to it.
    let mut full_data = [
        vec![0; 16],
        bincode::serialize(&data).map_err(ConfigError::Bincode)?,
    ]
    .concat();
    // Calculate the `md5` hash of the serialized data.
    hasher.update(&full_data[16..]);

    let hash: &[u8] = &hasher.finalize()[..];

    // Prepend the `md5` hash to the binary data. If the hash length is not 16 bytes, this will panic. This should never happen as the `md5` hash is always 16 bytes.
    // This function will panic if the two slices have different lengths.
    full_data[..16].clone_from_slice(hash);

    Ok(full_data)
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
