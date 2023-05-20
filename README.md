# binconf

Save and load binary configuration files easily in your application.

The data is hashed during serialization and validated when deserializing, so you can be sure that the data is not corrupted.

The config file is saved in your system config directory. On Linux for example it is saved in your home directory under `~/.config/<app_name>/<config_name>`.

## Usage

```rust
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Hash, Clone)]
struct TestConfig {
    strings: String,
    vecs: Vec<u8>,
}

fn main() {
    let config = TestConfig {
        strings: String::from("testing"),
        vecs: vec![1, 2, 3, 4, 5],
    };

    // Save the data
    binconf::store("binconf-app", Some("config.bin"), config.clone()).unwrap();

    // Load the data
    let stored = binconf::read::<TestConfig>("binconf-app", Some("config.bin"), false).unwrap();

    assert_eq!(stored.strings, config.strings);
    assert_eq!(stored.vecs, config.vecs);
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
