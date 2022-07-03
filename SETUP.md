**TODO: NEED TO ACTUALLY IMPLEMENT LINKER CHANGES BY INSTALLING XCODE ON MY MAC**

# First Time Setup

If any of the steps have already been done _or_ do not apply to your operating system, feel free to skip them. This exists mostly as an assertion of what this project assumes the environment it runs in has.

1. Install [rustup](https://rustup.rs/) to get the full Rust toolchain.
2. On MacOS, install [homebrew](https://brew.sh), which is a helpful package manager for MacOS.
3. On MacOS, install the XCode app from the App Store (needed to install `zld` in the next step).
4. Install the appropriate linker to speed up development:
```shell
# On Ubuntu
sudo apt-get install lld clang
# On Arch
sudo pacman -S lld clang
# On MacOS
brew install michaeleisel/zld/zld
```
5. Install the following Cargo CLI tools for easier development:
```shell
# Manually specifying a version just to be safe
cargo install cargo-watch --version 8.1.1
cargo install bunyan --version 0.1.7
```
