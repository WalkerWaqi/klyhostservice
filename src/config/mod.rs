use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::prelude::*;

lazy_static! {
    pub static ref CONFIG: Config = Config::new("config/kly.yaml");
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: Mqtt,
    udp: Udp,
    prometheus: Prometheus,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Mqtt {
    pub url: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Fwd {
    start: u16,
    size: u16,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Udp {
    fwd: Fwd,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Prometheus {
    url: String,
}

impl Config {
    pub fn new(file: &str) -> Config {
        let mut f = File::open(file).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
    
        let res: Result<Config, _> = serde_yaml::from_str(&s);
        res.unwrap()
    }
}