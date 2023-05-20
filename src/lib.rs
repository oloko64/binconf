use serde::{de::DeserializeOwned, Serialize};
use std::io::BufWriter;

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

    let file = BufWriter::new(std::fs::File::create(&conf_file).map_err(ConfigError::Io)?);
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

    struct TestConfig {
        test: String,
        
    }

    #[test]
    fn read_default_config() {
        let config: String = read("test-binconf", None, false).unwrap();
        assert_eq!(config, String::from(""));
    }
}
