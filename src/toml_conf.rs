#[cfg(feature = "toml_conf")]
use crate::{ConfigError, ConfigLocation};

#[cfg(feature = "toml_conf")]
pub fn load_toml<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    reset_conf_on_err: bool,
) -> Result<T, ConfigError>
where
    T: Default + serde::Serialize + serde::de::DeserializeOwned,
{
    use std::{fs::read_to_string, io::Write};

    let config_file_path = crate::config_location(
        app_name.as_ref(),
        config_name.into(),
        "toml",
        location.as_ref(),
    )?;

    let save_default_conf = || {
        let default_config = T::default();
        let mut file = std::io::BufWriter::new(
            std::fs::File::create(&config_file_path).map_err(ConfigError::Io)?,
        );
        let toml_str = toml::to_string_pretty(&default_config).map_err(ConfigError::TomlSer)?;
        file.write_all(toml_str.as_bytes())
            .map_err(ConfigError::Io)?;
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

pub fn store_toml<'a, T>(
    app_name: impl AsRef<str>,
    config_name: impl Into<Option<&'a str>>,
    location: impl AsRef<ConfigLocation>,
    data: T,
) -> Result<(), ConfigError>
where
    T: serde::Serialize,
{
    use std::io::Write;

    let config_file_path = crate::config_location(
        app_name.as_ref(),
        config_name.into(),
        "toml",
        location.as_ref(),
    )?;

    let mut file =
        std::io::BufWriter::new(std::fs::File::create(config_file_path).map_err(ConfigError::Io)?);

    let toml_str = toml::to_string_pretty(&data).map_err(ConfigError::TomlSer)?;

    file.write_all(toml_str.as_bytes())
        .map_err(ConfigError::Io)?;

    Ok(())
}
