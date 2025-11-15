# LDR POC

POC for using Rust and PIGPIO to read LDR sensors on a RaspberryPI 4B

## PI Setup

You must be running 64 Bit image (Trixie) for these steps to work.

### PIGPIO

Instal PIGPIO from source, it no longer ships with the RPI image

```bash
sudo apt update
sudo apt install gcc make git
git clone https://github.com/joan2937/pigpio.git
cd pigpio
make
sudo make install
```

## Mac setup

Install Rust. WARNING do not install `Rust` via any other process than via `rustup` as the `cross` command will only work with `rustup` `Rust` installs.

Get rid of Rust if installed with Homebrew

```bash
brew uninstall rust
```

Install Rust via Rustup

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install target (suitable for RPI 4 +)

```bash
rustup target add aarch64-unknown-linux-gnu
```

Install cross

```bash
cargo install cross --force --features docker-image
```

Setup env vars to deal with cross compilation deps. Add the following to your `.zshrc` or similar

```bash
export PIGPIO_SYS_USE_PKG_CONFIG=1
export PIGPIO_SYS_GENERATE_BINDINGS=0
```

## Build project

You must have the sysroot inplace (see above) before building your project.

```bash
cross build --target aarch64-unknown-linux-gnu --release

```

## Copy program to the Raspberry PI

From your MacOS

```bash
scp target/aarch64-unknown-linux-gnu/release/<your-program> \
    operator@raspberrypi.local:~
```

_(Change the RaspberryPI username and host and path if needed.)_
