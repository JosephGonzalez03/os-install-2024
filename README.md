# os-install-2024

This is my post-arch linux installation CLI. This will install the fonts and software I use as
well as apply my dotfiles.

### How to Use

1. Run the archintall helper. Make sure to select hyprland as the DE.
2. Install firefox and git.
3. Install [Rust](https://www.rust-lang.org/tools/install).
3. Download this project from GitHub and build it:
```
  cargo build --release
```
4. Make the binary executable:
```
  chmod +x target/release/os-install-2024
```
5. Run it with:
```
  os-install-2024 install
```
6. Enter the root and GitHub SSH key passwords when prompted.
