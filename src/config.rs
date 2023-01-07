use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Error, Vendor};

pub struct Config {
    pub first_use: bool,
    pub gpu_priority: HashMap<Vendor, u32>,
}

impl Default for Config {
    fn default() -> Self {
        let mut gpu_priority = HashMap::new();
        gpu_priority.insert(Vendor::NVIDIA, 0);
        gpu_priority.insert(Vendor::AMD, 1);
        gpu_priority.insert(Vendor::Intel, 2);
        Self {
            gpu_priority,
            first_use: true,
        }
    }
}

impl Config {
    pub fn parse(src: &str) -> Self {
        let mut this = Self::default();
        let mut conf: Vec<(&str, &str)> = src
            .lines()
            .filter_map(|line| line.split_once("="))
            .collect();
        while let Some((key, val)) = conf.pop() {
            let key = key.to_ascii_lowercase();
            let val = val.to_ascii_lowercase();
            match key.as_str() {
                "gpu_priority" => {
                    val.split(",")
                        .filter_map(|v| v.split_once(":"))
                        .filter_map(|(k, v)| {
                            let v: u32 = match v.parse() {
                                Ok(v) => v,
                                Err(_) => return None,
                            };
                            match k.trim() {
                                "nvidia" => Some((Vendor::NVIDIA, v)),
                                "amd" => Some((Vendor::AMD, v)),
                                "intel" => Some((Vendor::Intel, v)),
                                _ => None,
                            }
                        })
                        .for_each(|(v, p)| {
                            this.gpu_priority.insert(v, p);
                        });
                }
                "first_use" => {
                    this.first_use = val.parse().unwrap_or(false);
                }
                _ => (),
            }
        }
        this
    }
    pub fn open() -> Result<Self, super::Error> {
        std::fs::create_dir_all(config_path().parent().unwrap())?;
        if std::fs::try_exists(config_path().canonicalize()?)? {
            let this = std::fs::read_to_string(config_path())
                .map(|src| Self::parse(&src))
                .unwrap_or(Self::default());
            this.save()?;
            return Ok(this);
        } else {
            let this = Self::default();
            this.save()?;
            return Ok(this);
        }
    }
    pub fn save(&self) -> Result<(), super::Error> {
        let mut output = String::new();
        output.push_str(
            format!(
                "gpu_priority={}\n",
                self.gpu_priority
                    .iter()
                    .map(|v| format!("{}:{}", v.0.to_string(), v.1))
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .as_str(),
        );
        output.push_str(format!("first_use={}\n", self.first_use.to_string()).as_str());
        std::fs::write(config_path(), output.as_bytes()).map_err(Error::Io)
    }
}

fn config_path() -> PathBuf {
    let path = std::env::var("HOME").unwrap_or(String::from("./"));
    PathBuf::from(path)
        .canonicalize()
        .unwrap()
        .join(".config/primer/config.txt")
}
