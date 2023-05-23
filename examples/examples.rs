use binconf::ConfigLocation::{Cache, Config, Cwd, LocalData};
use serde::{Deserialize, Serialize};

// The struct needs to have all of its fields as owned types
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

    // Save the data at the user's config directory in binary format
    binconf::store_bin("binconf-app", Some("config.bin"), Config, &config).unwrap();

    // Save the data at the user's config directory in TOML format, requires the `toml-conf` feature
    // binconf::store_toml("binconf-app", Some("config.toml"), Config, &config).unwrap();

    // Save the data at the user's config directory in RON format, requires the `ron-conf` feature
    // binconf::store_ron("binconf-app", Some("config.ron"), Config, &config).unwrap();

    // Save the data at the user's config directory in JSON format, requires the `json-conf` feature
    // binconf::store_json("binconf-app", Some("config.json"), Config, &config).unwrap();

    // Save the data at the user's config directory in YAML format, requires the `yaml-conf` feature
    // binconf::store_yaml("binconf-app", Some("config.yaml"), Config, &config).unwrap();

    // Load the data from the user's config directory, from a binary file
    let stored =
        binconf::load_bin::<TestConfig>("binconf-app", Some("config.bin"), Config, false).unwrap();

    // Load the data from the user's config directory, from a TOML file, requires the `toml-conf` feature
    // let stored = binconf::load_toml::<TestConfig>("binconf-app", Some("config.toml"), Config, false).unwrap();

    // Load the data from the user's config directory, from a RON file, requires the `ron-conf` feature
    // let stored = binconf::load_ron::<TestConfig>("binconf-app", Some("config.ron"), Config, false).unwrap();

    // Load the data from the user's config directory, from a JSON file, requires the `json-conf` feature
    // let stored = binconf::load_json::<TestConfig>("binconf-app", Some("config.json"), Config, false).unwrap();

    // Load the data from the user's config directory, from a YAML file, requires the `yaml-conf` feature
    // let stored = binconf::load_yaml::<TestConfig>("binconf-app", Some("config.yaml"), Config, false).unwrap();

    assert_eq!(stored.strings, config.strings);
    assert_eq!(stored.vecs, config.vecs);
}
