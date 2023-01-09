use std::path::PathBuf;
use tini::Ini;

use crate::{Error, Vendor};

pub struct Config {
    pub first_use: bool,
    pub gpu_priority: Vec<Vendor>,
    pub kill_on_unplug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            first_use: true,
            gpu_priority: vec![Vendor::NVIDIA, Vendor::AMD, Vendor::Intel],
            kill_on_unplug: true,
        }
    }
}

impl Config {
    pub fn open() -> Result<Self, super::Error> {
        let path = config_path();
        std::fs::create_dir_all(&path.parent().unwrap())?;
        if !std::fs::try_exists(&path)? {
            std::fs::File::create(path)?;
        }
        let ini = Ini::from_file(&config_path())?;
        Ok(Self {
            first_use: ini.get("general", "first_use").unwrap_or(true),
            gpu_priority: ini
                .get_vec::<String>("general", "gpu_priority")
                .unwrap_or(vec!["nvidia".into(), "amd".into(), "intel".into()])
                .into_iter()
                .filter_map(|vendor| match vendor.trim() {
                    "nvidia" => Some(Vendor::NVIDIA),
                    "amd" => Some(Vendor::AMD),
                    "intel" => Some(Vendor::Intel),
                    _ => None,
                })
                .collect(),
            kill_on_unplug: ini.get("general", "kill_on_unplug").unwrap_or(true),
        })
    }
    pub fn save(&self) -> Result<(), super::Error> {
        Ini::new()
            .section("general")
            .item("first_use", false)
            .item_vec(
                "gpu_priority",
                &self
                    .gpu_priority
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
            )
            .item("kill_on_unplug", self.kill_on_unplug)
            .to_file(config_path().as_path())
            .map_err(|e| Error::Io(e))
    }
}

fn config_path() -> PathBuf {
    let path = std::env::var("HOME").unwrap_or(String::from("./"));
    PathBuf::from(path)
        .canonicalize()
        .unwrap()
        .join(".config/primer/config.ini")
}
