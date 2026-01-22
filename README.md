# binconf

[![Binconf Workflow](https://github.com/OLoKo64/binconf/actions/workflows/rust.yml/badge.svg)](https://github.com/OLoKo64/binconf/actions/workflows/rust.yml)

Save and load from a binary configuration file with ease.

The data is hashed ([XXH3](https://github.com/Cyan4973/xxHash)) during serialization and validated when deserializing, so you can be sure that the data is not corrupted.

Crate used for XXH3: [xxhash-rust](https://crates.io/crates/xxhash-rust)

---

### Deprecated Features

The `serde_yaml` library is currently [unmaintained](https://github.com/dtolnay/serde-yaml/releases/tag/0.9.34). Although it still functions, I recommend using `serde_json` or `serde_toml` as alternatives while a new replacement is being developed.


---

You can also save the configuration using `toml`, `json`, `yaml` and `ron`. You need to enable the respective feature for this. **(hash validation is not supported for `toml`, `json`, `yaml` or `ron`)**

### Optional Features

- `bincode-conf`: Enables saving and loading configurations in binary. (Enabled by default)
- `toml-conf`: Enables saving and loading configurations using `toml`.
- `json-conf`: Enables saving and loading configurations using `json`.
- `yaml-conf`: Enables saving and loading configurations using `yaml`.
- `ron-conf`: Enables saving and loading configurations using `ron`.
- `full`: Enables all configuration types. This gives you the ability to save and load using `toml`, `json`, `yaml`, `ron` as well as binary.

### Disabling Default Features

If you want to only use one of the features, you can disable the default features and enable the feature you want to use.

Only using `toml`, for example:

```
[dependencies.binconf]
features = ["toml-conf"]
default-features = false
```

---

## Usage

```rust
use binconf::ConfigLocation::{Cache, Config, LocalData, Cwd};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
struct TestConfig {
    strings: String,
    vecs: Vec<u8>,
}

fn main() {
    let config = TestConfig {
        strings: String::from("binconf"),
        vecs: vec![1, 2, 3, 4, 5],
    };

    // Save the data at the user's config directory
    binconf::store_bin("binconf-app", Some("config.bin"), Config, &config).unwrap();

    // Load the data from the user's config directory
    let stored =
        binconf::load_bin::<TestConfig>("binconf-app", Some("config.bin"), Config, false).unwrap();

    assert_eq!(stored.strings, config.strings);
    assert_eq!(stored.vecs, config.vecs);
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
