<!--
SPDX-FileCopyrightText: © 2023 Jinwoo Park (pmnxis@gmail.com)

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Develop Environment

_※ It might available with Windows, but in this document only consider Debian-Linux._

### apply USB (stlink-v2/3) permission rule to ignore root access
```sh
# refer https://calinradoni.github.io/pages/200616-non-root-access-usb.html
sudo tee /etc/udev/rules.d/70-st-link.rules > /dev/null <<'EOF'
# ST-LINK V2
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv2_%n"

# ST-LINK V2.1
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv2-1_%n"
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3752", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv2-1_%n"

# ST-LINK V3
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374d", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv3loader_%n"
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374e", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv3_%n"
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374f", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv3_%n"
SUBSYSTEMS=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3753", GROUP="plugdev", MODE="660", TAG+="uaccess", SYMLINK+="stlinkv3_%n"
EOF
```

### apply rules and reboot linux
```sh
sudo usermod -a -G plugdev $USER
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo reboot
```

### Necessary apt package for rust embedded
```sh
# necessary for basic software development environment
sudo apt install curl git build-essential -y
# build.rs uses libgit2-sys to get commit hash
sudo apt install pkg-config libssl-dev -y
# for knurling-rs/probe-run
sudo apt install libusb-1.0-0-dev libudev-dev -y

# Install rustc/rustup/cargo for rust development environment
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install probe-run for debug/flash firmware binary on target board.
cargo install probe-run

# Install cargo-binutils for analyze mem/flash usage.
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

### Build
```sh
cargo build
```

```sh
pmnxis@lmabdaDeb  ~/Develop/billmock-app-rs   master  cargo build
info: syncing channel updates for 'nightly-2023-08-24-x86_64-unknown-linux-gnu'
info: latest update on 2023-08-24, rust version 1.74.0-nightly (249595b75 2023-08-23)
info: downloading component 'cargo'
info: downloading component 'clippy'
info: downloading component 'llvm-tools'
info: downloading component 'rust-docs'
info: downloading component 'rust-src'
info: downloading component 'rust-std' for 'thumbv6m-none-eabi'
info: downloading component 'rust-std'
info: downloading component 'rustc'
info: downloading component 'rustfmt'
info: installing component 'cargo'
info: installing component 'clippy'
info: installing component 'llvm-tools'
info: installing component 'rust-docs'
info: installing component 'rust-src'
info: installing component 'rust-std' for 'thumbv6m-none-eabi'
info: installing component 'rust-std'
info: installing component 'rustc'
info: installing component 'rustfmt'
    Updating crates.io index
    Updating git repository `https://github.com/embassy-rs/embassy.git`
```

### Cargo Run
```sh
pmnxis@lmabdaDeb  ~/Develop/billmock-app-rs   master  cargo run
    Finished dev [optimized + debuginfo] target(s) in 0.06s
     Running `probe-run --chip STM32G030C8Tx --log-format '{t} [{L}][ {f}:{l} ] {s}' target/thumbv6m-none-eabi/debug/billmock-app-rs`
(HOST) INFO  flashing program (45 pages / 45.00 KiB)
(HOST) INFO  success!
────────────────────────────────────────────────────────────────────────────────
0.000000 [DEBUG][ fmt.rs:130 ] rcc: Clocks { sys: Hertz(16000000), apb1: Hertz(16000000), apb1_tim: Hertz(16000000), ahb1: Hertz(16000000) }
+-----------------------------------------------------------+
Firmware Ver : billmock-app-rs 0.1.2
Git Hash     : bf976acd38633f7204e9423def8b5b062e0a0ad3
Git Datetime : 2023-09-17 20:55:10 +0900 | bf976ac
+-----------------------------------------------------------+
0.004272 [TRACE][ fmt.rs:117 ] USART: presc=1, div=0x0000008b (mantissa = 8, fraction = 11)
0.004638 [TRACE][ fmt.rs:117 ] Using 16 bit oversampling, desired baudrate: 115200, actual baudrate: 115107
1.006286 [WARN ][ serial_device.rs:156 ] The module use a example library. It may not work in real fields.
OUT[     LED2-Indicator] : Low
OUT[     LED1-Indicator] : Low
```
