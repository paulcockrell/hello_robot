# LDR POC

POC for using Rust and PIGPIO to read LDR sensors on a RaspberryPI >= 4

## PI Setup

You must be running 64 Bit image (Trixie) for these steps to work.

```bash
sudo nano /boot/firmware/config.txt
```

add the following:

```bash
dtparam=i2c_arm=on
dtoverlay=i2c1,pins_2_3
```

Reboot the RPI

```bash
sudo reboot
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

## Send commands to ROS

The Robot Operating System is listening on a Unix socket, so you can issue commands to it like so:

```bash
echo '{"type": "servo", "angle": 60}' | socat - UNIX-CONNECT:/tmp/robot.sock
```

Available commands:

```bash
{"type": "motor", "direction": "forward", "speed": 100}
{"type": "servo", "angle": 60}
{"type": "led", "r": 255, "g": 80, "b": 0}
{"type": "camera", "command": "snap"}
```
