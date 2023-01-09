# Primer

PRIME GPU offload helper for Wayland systems, written in Rust.

## Table of Contents
* [Installation](README.md#installation)
* [Usage](README.md#usage)
* [Supported Drivers](README.md#supported-drivers)
* [Tested Hardware](README.md#tested-hardware)
## Installation
A dialog library is recommended. See if your distribution has `zenity` or `dialog`.
### Arch Linux (AUR)
```sh
# for yay
yay -S primer-git

# for paru
paru -S primer-git
```

### From Source
```
git clone https://github.com/zurrty/primer.git
cd primer
cargo build --profile release
```
Build Dependencies:
* systemd-libs (sorry artix users)
* rust (obviously)

Note that Rust needs to be installed to build primer. See if your distribution has a `rustup` package.

## Usage
To use, simply put `primer` before any command you want to run.

If you want all of your Steam games to use your dedicated graphics (when available), you can launch Steam like so: 
```
primer steam
```
The config is stored at `~/.config/primer/config.txt`. 

## Supported Drivers

* `nvidia-open`
* `radeon`
* `i915`

## Tested Hardware
### NVIDIA
| GPU | Driver | Enclosure | Working |
| --- | --- | --- | --- |
| 3070 | nvidia-open-dkms-520.56.06-1 | Core X | yes
