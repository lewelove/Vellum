# Vellum

Vellum is an album-centric web based MPD client and library manager built on a local first, plaintext driven architecture for archivist-minded collectors.

Utilizing **read-only audio & mutable manifest** philosophy for each album, Vellum enables plaintext library management and version-controlled metadata, leaving your binary audio files entirely untouched.

## How it works

Vellum operates in a layer between your audio files and your MPD instance. It treats each album as data object compiled from audio files and ancillary data sources. The final state of an album is resolved by merging of:

1.  **Manifest**: Your intent expressed in `metadata.toml` file, anchoring the the album root folder.
2.  **Audio Properties**: Technical data (encoding, bitrate, sample rate, duration) read directly from audio files themselves.
3.  **Computed Attributes**: Dynamic data derived during compilation via Rust functions, like static album cover art analysis.

After all data is successfully resolved and compiled, Vellum writes `metadata.lock.json` "Lock" file in album root folder. This is what the server actually reads, making library browsing near-instant even with thousands of albums.

## Why this way?

*   **Non-destructive**: Vellum won't ever corrupt your FLACs or change their checksums.
*   **Version Control**: Because your library's "brain" is a collection of TOML files, you can track your entire history of library edits using Git.
*   **Computed Metadata**: Vellum does the heavy lifting at compile time. It calculates Oklab-based color palettes and visual complexity (entropy) for every cover. This data is used to drive the WebGL background shaders and advanced sorting in the UI.

## The Stack

Vellum is built with a multi-language architecture designed for high performance and easy extensibility:

*   **Rust (Engine & Server)**: Handles the heavy lifting. It manages the parallel metadata harvester, the compilation of TOML manifests into JSON locks, and an **Axum**-based server. It uses an actor model to communicate with MPD via **mpd_client** crate.
*   **Svelte 5 (Web App)**: A modern, reactive interface utilizing **Runes** for state management. To keep the UI fluid even with thousands of albums, all library filtering, sorting, and grouping logic is offloaded to a **Web Worker**. Generative background effects are rendered via custom **WebGL/GLSL** shaders.
*   **Python (Automation)**: Python scripts handle auxiliary tasks such as, for example, fetching lyrics from Genius.
*   **Nix & Tooling**: A `flake.nix` manages the development environment, providing a reproducible toolchain across Rust, Python, and Node. **Bun & Vite** are used for fast frontend builds and proxying.

## Installation & Usage

Vellum is in active development. To ensure a reproducible toolchain across Rust, Python, Node, and their respective system dependencies, the development environment is entirely managed via [Nix](https://nixos.org/download/).

**Prerequisites:** 
* Nix (with [flakes enabled](https://nixos.wiki/wiki/Flakes#Enable_flakes))
* A running `mpd` instance

### 1. Setup the Environment
Clone the repository and drop into the unified development shell:

```bash
git clone https://github.com/lewelove/Vellum.git vellum
cd vellum
nix develop
```

Once inside the Nix shell, install `node_modules` with bun:

```bash
cd web-app
bun install
```

And build the Rust binary:

```bash
vellum build
```

### 2. Configure Your Library
Vellum requires a basic `config.toml` to know where your library and MPD instance live. By default, it looks for `~/.config/vellum/config.toml` or a `config.toml` in your project root. 

### 3. Run the Stack
Because Vellum decouples the frontend UI from the backend coordinator, you will run them as separate processes during development:

```bash
# Terminal 1: Start the Rust backend and MPD watchdog
vellum server

# Terminal 2: Start the Svelte web interface
vellum ui
```

### CLI Usage

The `vellum` CLI tool is the central driver for managing your library's state and lifecycle. 

* `vellum manifest` — Scans your library root for unmanaged audio directories and generates the initial `metadata.toml` anchor files.
* `vellum update` — The core compiler command. Reads your TOML changes, parses the physical audio properties, calculates visual attributes, and writes the resolved `metadata.lock.json` files.
* `vellum server` — Starts the Axum backend server and the MPD synchronization watchdog.
* `vellum ui` — Starts the Vite/Svelte development server for the web interface.
* `vellum run <script>` — Executes Python automation scripts against the currently playing (or specified) album, such as fetching lyrics via Genius.
