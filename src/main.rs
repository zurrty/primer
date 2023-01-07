#![feature(fs_try_exists)]
pub mod config;

use colored::*;
use dialog::DialogBox;
use std::process::Command;
use udev::{Device, Enumerator};

#[macro_use]
extern crate derive_error;

#[derive(Error, Debug)]
pub enum Error {
    Io(std::io::Error),
    DeviceNotFound,
    InvalidProperty,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Vendor {
    NVIDIA,
    AMD,
    Intel,
}

impl ToString for Vendor {
    fn to_string(&self) -> String {
        match self {
            Vendor::NVIDIA => "NVIDIA",
            Vendor::AMD => "AMD",
            Vendor::Intel => "Intel",
        }
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct GPU {
    vendor: Vendor,
    name: String,
    integrated: bool,
    dev: Device,
}

impl GPU {
    pub fn name_fancy(&self) -> ColoredString {
        match self.vendor {
            Vendor::NVIDIA => self.name.green(),
            Vendor::AMD => self.name.red(),
            Vendor::Intel => self.name.blue(),
        }
    }
    pub fn print_info(&self) {
        let name = format!("-- {} --", self.name_fancy()).bold();
        println!("{}", name);
        self.dev.properties().for_each(|prop| {
            println!(
                "{}: {}",
                prop.name().to_str().unwrap_or("").bold(),
                prop.value().to_str().unwrap_or("")
            )
        })
    }
    pub fn pci_slot(&self) -> Option<String> {
        match self
            .dev
            .property_value("PCI_SLOT_NAME")
            .map(|slot| slot.to_str())
            .flatten()
        {
            Some(slot) => Some(
                slot.chars()
                    .map(|c| match c {
                        ':' | '.' => '_',
                        _ => c,
                    })
                    .collect(),
            ),
            None => None,
        }
    }
    pub fn prepare_run(&self, mut command: Vec<String>) -> Result<Command, Error> {
        println!(
            "{}",
            format!("-- Using GPU: {} --", self.name_fancy()).bold()
        );
        let pci = match self.pci_slot() {
            Some(pci) => pci,
            None => return Err(Error::InvalidProperty),
        };
        let mut cmd = std::process::Command::new(command.remove(0).as_str());
        cmd.args(command);
        match self.vendor {
            Vendor::NVIDIA => {
                cmd.env("DRI_PRIME", format!("pci-{pci}"));
                cmd.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
                cmd.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
            }
            Vendor::AMD => {
                cmd.env("DRI_PRIME", format!("pci-{pci}"));
            }
            Vendor::Intel => (), // arc cards not supported yet
        };
        Ok(cmd)
    }
}

fn find_gpus() -> Result<Vec<GPU>, Error> {
    let mut enumerator = Enumerator::new()?;
    let devices: Vec<GPU> = enumerator
        .scan_devices()?
        .filter(|dev| dev.driver().is_some())
        .filter_map(|dev| {
            let driver = dev.driver().map(|drv| drv.to_str()).flatten().unwrap_or("");
            let vendor = match driver {
                "nvidia" => Some(Vendor::NVIDIA),
                "i915" => Some(Vendor::Intel),
                "radv" => Some(Vendor::AMD),
                _ => None,
            }?;
            let name = dev
                .property_value("ID_MODEL_FROM_DATABASE")
                .map_or("", |name| name.to_str().unwrap_or(""))
                .to_string();
            let integrated = name.to_lowercase().contains("integrated"); // theres a better way probably but it works for now
            Some(GPU {
                vendor,
                name,
                integrated,
                dev,
            })
        })
        .collect();
    if devices.len() > 0 {
        Ok(devices)
    } else {
        Err(Error::DeviceNotFound)
    }
}

pub fn prime_run(args: Vec<String>) -> Result<(), Error> {
    println!("{:?}", args);
    let mut config = config::Config::open()?;
    if config.first_use {
        dialog::Message::new("It seems that it's your first time using primer, welcome!\nYou can edit the config at \"~/.config/primer/config.txt\"")
        .title("Primer")
        .show()
        .expect("Failed to show welcome dialog!");
        config.first_use = false;
        config.save()?;
    }
    let mut gpus = match find_gpus() {
        Ok(gpus) => gpus,
        Err(e) => {
            dialog::Message::new("Primer failed: no graphics device was found. Please make sure you have the right drivers installed for your system.")
                .title("Primer Error")
                .show()
                .unwrap();
            return Err(e);
        }
    };
    gpus.sort_by(|a, b| {
        config
            .gpu_priority
            .get(&a.vendor)
            .cmp(&config.gpu_priority.get(&b.vendor))
    });
    let gpu = match gpus.first() {
        Some(gpu) => gpu,
        None => return Err(Error::DeviceNotFound),
    };
    if gpu.integrated {
        dialog::Message::new("No discrete GPU detected, using integrated graphics")
            .title("Primer")
            .show()
            .unwrap();
    }
    gpu.prepare_run(args)?.spawn()?.wait()?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        println!("No command provided. Exiting...");
        return Ok(())
    }
    args.remove(0);
    prime_run(args).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::prime_run;

    #[test]
    fn test_glxinfo() {
        prime_run(vec!["glxinfo".to_string(), "-B".to_string()]).unwrap();
    }
}
