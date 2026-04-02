# S.I.M.P - Smart Integrated Markdown Previewer

Local **Markdown reader and writer** built with **Rust** and **Iced 0.14**.

- Open a folder (nested sidebar, folders as categories) or a single `.md` file
- Viewer / Editor modes with auto-save (debounced writes)
- Ignores any directory named exactly `asset` and its subtree
- Filesystem watcher refreshes the tree and reloads the current file when safe
- Create new Markdown files or subfolders from the category panel

## Requirements

- **Rust 1.88+** (required by `iced` 0.14). Install from [rustup](https://rustup.rs/) if needed.

## Build

Clone the repository, open a terminal in the project root, then run:

```bash
cargo build --release
```

The binary is written to `target/release/markdown_view` (on Windows: `target\release\markdown_view.exe`).

To build and run in one step:

```bash
cargo run --release
```

### Windows

1. Use the default **`x86_64-pc-windows-msvc`** toolchain from rustup (the usual choice on Windows).
2. Install **Microsoft C++ build tools** so Rust can link the final executable (see [Troubleshooting](#windows-linkexe-not-found) if you see `link.exe` not found).
3. From the project folder:

   ```powershell
   cargo build --release
   ```

### macOS

1. Install **Xcode Command Line Tools** (compiler and linker):

   ```bash
   xcode-select --install
   ```

2. From the project folder:

   ```bash
   cargo build --release
   ```

   This produces a native binary for your machine (Apple Silicon or Intel). To cross-compile for the other architecture, add the target with `rustup target add` and pass `--target aarch64-apple-darwin` or `--target x86_64-apple-darwin`.

### Linux

1. Install a **C toolchain** and **pkg-config** (names vary by distribution):

   - **Debian / Ubuntu** (example):

     ```bash
     sudo apt update
     sudo apt install build-essential pkg-config libssl-dev
     ```

     You may also need libraries used by the GUI stack (X11 and/or Wayland). If the build fails with missing headers or `-l` errors, install the suggested `*-dev` packages; common additions include `libwayland-dev`, `libxkbcommon-dev`, `libgtk-3-dev`, or X11-related development packages for your session.

   - **Fedora** (example): `sudo dnf install gcc pkgconf openssl-devel`

2. Ensure **GPU drivers** or **Mesa** are available so **Vulkan** (used by the renderer) can run; most desktop setups already satisfy this.

3. From the project folder:

   ```bash
   cargo build --release
   ```

## Troubleshooting

### Windows: `link.exe` not found

The default Rust host is **`x86_64-pc-windows-msvc`**, which needs Microsoft’s C++ linker (`link.exe`). Without it, the very first crates fail to compile.

**Fix (recommended): install MSVC Build Tools**

1. Download [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (or the full Visual Studio Installer).
2. Run the installer and select the workload **“Desktop development with C++”** (or at least the **MSVC** toolset and a **Windows SDK**).
3. Finish the install, then **open a new** PowerShell or Command Prompt (so `PATH` updates).
4. Confirm the linker exists: `where.exe link.exe` (should show a path under `Microsoft Visual Studio\...`).
5. From this folder: `cargo build --release`

**Alternative: full Visual Studio**  
Installing Visual Studio Community with the **“Desktop development with C++”** workload also provides `link.exe`.

**Note on the GNU toolchain**  
Using `stable-x86_64-pc-windows-gnu` avoids `link.exe` but requires a working MinGW environment (`dlltool.exe`, etc.). If those tools are missing, stick with MSVC as above.

## Keyboard shortcuts

| Shortcut        | Action              |
|----------------|---------------------|
| `Ctrl+O`       | Open file           |
| `Ctrl+Shift+O` | Open folder         |
| `Ctrl+E`       | Toggle Viewer/Editor|
| `Escape`       | Cancel create dialog|

See [`markdown_reader_writer_spec.md`](markdown_reader_writer_spec.md) for full product behavior.
