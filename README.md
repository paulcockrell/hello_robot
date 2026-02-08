# hello_robot

A simple Raspberry Pi-based robot control system written in Rust, with a lightweight web interface for manual control.

The kit can be found [here](https://www.aliexpress.com/p/tesla-landing/index.html?UTABTest=aliabtest128998_32160&src=bing&albch=shopping&acnt=135105396&albcp=555690814&albag=1304022061862648&slnk=&trgt=pla-2333301113533049&plac=&crea=81501442626688&netw=s&device=c&mtctp=e&utm_source=Bing&utm_medium=shopping&utm_campaign=PA_Bing_%20GB_PMAX_9_MAXV__AESupply_26.1.23_hylt_1004266343&utm_content=AssetGroup_102125_153753&utm_term=Adeept%204WD%20Smart%20Robot%20Kit&msclkid=1ec982150c9413f981fdeb9da447c8bd&aff_fcid=11a13a589b4648c1abf47553a9a1844a-1770563799480-09519-UneMJZVf&aff_fsk=UneMJZVf&aff_platform=aaf&sk=UneMJZVf&aff_trace_key=11a13a589b4648c1abf47553a9a1844a-1770563799480-09519-UneMJZVf&terminal_id=4274b293fbe04386a62969acb4466e0d&scenario=c_ppc_item_bridge&productId=1005009982011552&_immersiveMode=true&withMainCard=true&OLP=1123114608_f_group1&o_s_id=1123114608).

This repo contains:

- Rust code driving motors, servos, LEDs, and basic sensors
- A web interface for controlling the robot and observing status
- Static assets and templates for the web UI

_It is a learning project, meant to explore hardware control and UI design in Rust._

What this actually does

- Drives motors and servos via GPIO
- Controls RGB LEDs
- Serves a browser UI to control the robot in real time
- Runs entirely on a Raspberry Pi

## Requirements

- The [kit](https://www.aliexpress.com/p/tesla-landing/index.html?UTABTest=aliabtest128998_32160&src=bing&albch=shopping&acnt=135105396&albcp=555690814&albag=1304022061862648&slnk=&trgt=pla-2333301113533049&plac=&crea=81501442626688&netw=s&device=c&mtctp=e&utm_source=Bing&utm_medium=shopping&utm_campaign=PA_Bing_%20GB_PMAX_9_MAXV__AESupply_26.1.23_hylt_1004266343&utm_content=AssetGroup_102125_153753&utm_term=Adeept%204WD%20Smart%20Robot%20Kit&msclkid=1ec982150c9413f981fdeb9da447c8bd&aff_fcid=11a13a589b4648c1abf47553a9a1844a-1770563799480-09519-UneMJZVf&aff_fsk=UneMJZVf&aff_platform=aaf&sk=UneMJZVf&aff_trace_key=11a13a589b4648c1abf47553a9a1844a-1770563799480-09519-UneMJZVf&terminal_id=4274b293fbe04386a62969acb4466e0d&scenario=c_ppc_item_bridge&productId=1005009982011552&_immersiveMode=true&withMainCard=true&OLP=1123114608_f_group1&o_s_id=1123114608)
- Raspberry Pi 4 or newer
- 64-bit Raspberry Pi OS
- Rust installed on the Pi

## Setup on Raspberry Pi

1. Install Rust

Rust must be installed via rustup:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Make sure ~/.cargo/bin is in your PATH.

2. System packages

```bash
sudo apt update
sudo apt install -y \
  libopencv-dev \
  libclang-dev \
  clang \
  pkg-config \
  libcamera-apps \
  gstreamer1.0-tools \
  gstreamer1.0-libcamera
```

**Note**: `OpenCV` is used only as a Rust-friendly interface to a GStreamer camera pipeline and for JPEG encoding — no computer vision processing is performed (yet!).

---

2. Build the project

From the repo root:

```bash
cargo build
```

This compiles the robot binary directly on the Raspberry Pi in debug mode.

---

3. Run the robot

```bash
sudo ./target/debug/hello_robot
```

This starts the robot control service and the embedded web server.
Open a browser on another device and connect to the Pi’s IP to interact with the UI.

---

## Web Interface

Once the robot is running:

- Visit `http://<pi-ip>:<port>` in a browser. e.g

```bash
http://raspberrypi.local:3000
```

- Use the UI to send commands to the robot
- The interface is built with Preact (no-build mode) so standard HTML/CSS/JS served from within a Rust webserver.

The static files live in the `static/` and `templates/` folders.

---

## Project structure

- `src/` – Rust application source code
- `static/` – Web UI assets (JS/CSS/images)
- `templates/` – HTML templates for pages
- `Cargo.toml` – Rust package config

## License

MIT — see the `LICENSE` file.
