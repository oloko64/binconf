mod binary;
mod toml;

#[cfg(feature = "binary")]
pub use binary::{load, store};

#[cfg(feature = "toml")]
pub use toml::{load_toml, store_toml};

#[derive(Debug, PartialEq, Clone)]
pub enum ConfigLocation {
    Config,
    Cache,
    LocalData,
}

impl AsRef<ConfigLocation> for ConfigLocation {
    fn as_ref(&self) -> &ConfigLocation {
        self
    }
}
