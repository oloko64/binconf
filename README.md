# binconf

[![Binconf Workflow](https://github.com/OLoKo64/binconf/actions/workflows/rust.yml/badge.svg)](https://github.com/OLoKo64/binconf/actions/workflows/rust.yml)

Save and load from a binary configuration file with ease.

The data is hashed ([md-5](https://crates.io/crates/md-5)) during serialization and validated when deserializing, so you can be sure that the data is not corrupted.

The config file is saved in your system config directory. On Linux for example it is saved in your home directory under `~/.config/<app_name>/<config_name>`.

---

You can also save the config using `toml`. You need to enable the `toml` feature for this.

### Optional Features

- `full`: Enables all features. This gives you the ability to save and load using `toml` as well as `bincode`.
- `toml`: Enables saving and loading using `toml` instead of `bincode`.

### Disabling Default Features

If you want to only use `toml` you can disable the default features and enable the `toml` feature.

```
[dependencies.binconf]
features = ["toml"]
default-features = false
```

---

## Usage

```rust
use binconf::ConfigLocation::{Cache, Config, LocalData};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
struct TestConfig {
    strings: String,
    vecs: Vec<u8>,
}

fn main() {
    let config = TestConfig {
        strings: String::from("testing"),
        vecs: vec![1, 2, 3, 4, 5],
    };

    // Save the data at the user's config directory
    binconf::store("binconf-app", Some("config.bin"), Config, &config).unwrap();

    // Load the data from the user's config directory
    let stored =
        binconf::load::<TestConfig>("binconf-app", Some("config.bin"), Config, false).unwrap();

    assert_eq!(stored.strings, config.strings);
    assert_eq!(stored.vecs, config.vecs);
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
