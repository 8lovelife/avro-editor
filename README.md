# 🛠 Avro Editor

**Schema-Driven Avro Editor** — A native & Web dual-platform Avro data viewing and editing tool built with Rust, allowing you to load, view, edit, and export Avro data files without writing any code.

## ✨ Features

* **Schema-Driven:** Automatically parses the embedded writer schema after loading an `.avro` data file, rendering a visual editing interface based on the schema structure.
* **Deep Nesting Support:** Supports multi-level nesting display and editing for complex types such as `record`, `array`, `map`, `union`, `enum`, and `fixed`.
* **Logical Types:** Recognizes Avro logical types including `date`, `timestamp-millis`, and `decimal`.
* **Native + Web Dual-Platform:** Built on `eframe`/`egui`. The same codebase can be compiled into a native desktop application or targeted to `wasm32` to run in the browser.
* **File Import / Export:** Uses `rfd` to provide native file selection dialogs, supporting reading from and writing back to local `.avro` files.
* **Multiple Codecs:** Based on `apache-avro`, it supports `null` / `deflate` compression, with optional on-demand support for `snappy`, `zstandard`, `bzip2`, and `xz`.
* **Containerized Deployment:** Includes a built-in `Dockerfile` and `docker-compose.yml` for one-click building and deploying of the Web version image.

## 🖥 Tech Stack

| Component | Description |
| :--- | :--- |
| **egui / eframe** | Immediate mode GUI framework, drives interface rendering. |
| **apache-avro** | Avro data read/write and schema parsing. |
| **WebAssembly (wasm32)** | Browser execution support via `wasm-bindgen` / `web-sys`. |

## 🚀 Quick Start

### Prerequisites
* Rust (stable, installation via `rustup` is recommended)
* To build the Web version, install the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

### Run Locally (Native Desktop)
```bash
git clone https://github.com/8lovelife/avro-editor.git
cd avro-editor
cargo run --release
```

### Build the Web (WASM) Version
The project includes an `index.html` and can be built into a static site using `trunk` or `wasm-pack`. For example, using `trunk`:

```bash
cargo install --locked trunk
trunk serve
```
*This will start a local development server by default. Open the corresponding address in your browser to preview.*

### Run with Docker
```bash
docker compose up --build
```
Or build the image directly using the `Dockerfile`:
```bash
docker build -t avro-editor .
docker run -p 8080:8080 avro-editor
```
*(Please refer to the actual configurations in the `Dockerfile` and `docker-compose.yml` within the repository for specific port mappings and startup commands.)*

## 📖 Usage Guide

1. After launching the application, use the file selection dialog to load an `.avro` data file.
2. The application will automatically parse the writer schema from the file header and render a visual editing panel based on the field structure (`record`, `array`, `map`, `union`, `enum`, `fixed`, etc.).
3. Edit field values directly within the panel. Nested structures can be expanded and collapsed layer by layer.
4. Once editing is complete, export the data as a new `.avro` file.

## 🤝 Contributing

Issues and Pull Requests are highly welcome. If you discover any bugs or have feature suggestions during use, please provide feedback via Issues.

## 📄 License

MIT License © 2025 [8lovelife](https://github.com/8lovelife)
