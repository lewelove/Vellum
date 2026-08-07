# Leland

**Leland** is an MPD client and album-centric music library engine built from the first Unix Philosophy principles for archivist-minded collectors. It brings full Lua scriptability, plain-text data management, Ahead-Of-Time library compilation guarantees, hot reloading, and incredible performance to your album collection.

## Philosophy

- **The Album as The Fundamental Collection Unit.** This project focuses solely on collection and management of music albums. The point, I guess, is to bring back the feeling of physical collecting to the digital world. An album is the fundamental unit of this project because an album is the fundamental unit of any music collection in real life.

- **Immutable Audio / Mutable Metadata.** Audio files making up the album should be a bit-perfect preservation of the original media. Audio files are inherently static. Your metadata is inherently dynamic. This is the reason why the engine treats audio as a read-only source and separates everything mutable into separate ancillary files.

- **Power to The User.** The entire creative vision of this project was conceived around a stance: **you should not be bound by your collection interface choices**. Plain-text is *the* universal interface. To build upon it — is to bring raw power and future-proof compatibility to your collection for decades to come.

## Cool Features

### Everything in Plain-Text
Entire library metadata — from song names and album lengths in milliseconds — to custom album source URLs and ReplayGain values — to **anything specific that can exist in a text form describing an album in your collection** is stored and compiled within ancillary plain-text files. Edit them in Neovim, RipGrep/Sed them, run scripts against them. Everything can be version controlled, every change can be tracked, backed up and reverted — independently of the audio's embedded tags — in human readable database-less format. Once your collection's metadata hits Git and is uploaded to a remote repo you will never lose it ever again.

### Album as a Compiled Data Object
For the analogy's sake imagine an album directory as an entry in the physical archive. This entry contains data written with the human intent (`metadata.toml` and other TOML manifests) and the source you're trying to preserve (audio, cover art, lyrics, documents). Then you take all these and run a compiler against them to produce an album's **index** in this imagined archive. When you think about an album in this way, it stops being an opaque fuzzy object interpreted by each different media player on the fly, and becomes a set of data points that can be compiled into a standardized machine-readable data object (`album.lock.json`). This object is then read by the server to register it and to provide data for any further user-album interfacing. The compilation step also brings you all cool compile time features AOT programming languages have. You can express all these — type checking, correctness enforcement, linting, key-to-manifest binding and validation — in the Lua config.

### Decoupled Frontend and Backend
**This project's primary focus is on the Rust web server — the User Interface intentionally comes second.** The separation of concerns is essential in Unix Philosophy. Want to change UI theme? Want to add some cool display feature? No need to worry. You can directly edit contents of the `web-app/` or fully rewrite your own UI in a WebDev stack and run it in a browser — wiring it up to a running backend server using its Web API **today**. Furthermore, any UI framework that supports Web API functionality can control MPD and retrieve library and album data. You can build TUI apps, Godot based game-interfaces, or you can even straight up use Curl. The project's goal is to provide robust primitives, so you can interface your album collection in any weird & brilliant way possible.

### Actions
Since every album is compiled into a plain-text JSON — every album becomes scriptable. An **action** is a standalone executable that reads intermediary JSON from stdin (provided by the engine and populated with albums and config data at runtime) and performs some kind of logic based on this data. That's it. You can write actions in any language that supports reading JSONs (or even in simple shell scripts with Jq) and use them to infinitely expand library management functionality in Unix Philosophy style. Each action is configurable via its own CLI arguments and Lua config. Every action is callable by its own `/api/actions/{action_name}/` endpoint, so you can wire them up to and execute from any future interface. For built-in actions and more context of what they may be useful for look into the `actions/` directory.

## Interface Showcase

The engine is bundled with the default Web App interface written in fast and reactive Svelte framework. You can run it as standalone headless process with Bun and access it via any browser.

### Home Album Grid

Filter, group, and sort albums by any key imaginable. The grid uses slot based row snapping and 60fps physical smooth scrolling.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/472b6151-ad39-4458-a408-a2a45c14f177" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/c3ebe389-1c40-431a-8509-05118ae92b66" />

### Modal Album Drawer

Album preview, supporting multi-disc issues.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/0e8471d7-b695-43fb-81e7-fdf58f317359" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/603ea8af-99e2-4ef1-88ee-3fbae9591f90" />

### Playing Now & The Queue

Provide any GLSL shader as the background with custom palettes for each album via config and their `theme.toml` manifests.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/b911f2a9-bf33-446b-a1f1-c5131cd8972f" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/57db8e38-9189-4494-99f6-c2f1cf68aa56" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/685b24ea-61a1-4a08-b2de-ea0fb0e8961f" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/574e7814-7744-4cc8-898f-df8c617a6650" />

## Getting Started

**Prerequisites:** 

- Nix
- Bun
- An active MPD instance

This project is in active development. To ensure a reproducible environment it's managed by a Nix Flake. All further setup assumes a Nix prerequisite. You can also setup it all without Nix by having Cargo & Bun in shell — just not reproducibly.

### 1. Set Up The Environment & Build

Clone the repository:

```bash
git clone https://github.com/lewelove/leland.git
cd leland
```

Drop into the development shell:

```bash
nix develop
```

Or if you have `direnv` installed:

```bash
direnv allow
```

For default interface to run from cloned developer repo you must `cd` into its directory, install `node_modules` and `chmod +x run.sh`:

```bash
cd interfaces/web-app
bun install
chmod +x run_dev.sh run_prod.sh
```

Build the Rust binary, Web App, and all the Actions:

```bash
build
```

The `build` places `leland` executable at `{repo_path}/rust/target/release/leland`. You may alias this path in your shell of choice to `leland`, and then abbreviate it to `ll` for ease use.

### 2. Configure It

You create `~/.config/leland/init.lua` file:

```lua
-- Since we are using Lua, a truly god-tier config language, for convenience
-- you can define cloned repository path as local string variable
local repo_dir = "Path/To/Cloned/Repo/"

ll.config({ 
  storage = { 
    -- Define a library path containing all your albums
    library = "Path/To/Your/Library/Root/"
  }
})

-- Optionally you can define all keys besides standard ones
-- you want to load from TOML manifests and be present in `album.lock.json`

-- [album.lock.json].album.keys level
ll.compile.album.key({
  album_key_name = true
})

-- [album.lock.json].tracks[].keys level
ll.compile.tracks.key({ 
  track_key_name = true
})

-- For `leland interface` command to run you point default interface
-- to the `interfaces/web-app` directory from the previous step
-- and specify the run-production-build script
ll.interfaces({ default = {
  -- Quite handy!
  directory = repo_dir .. "interfaces/web-app/"
  run = repo_dir .. "interfaces/web-app/run_prod.sh"
}})
```

For the config reference check out [my leland dotfiles](https://github.com/lewelove/nix-config/tree/main/dotfiles/.config/leland). The config documentation is coming soon...

### 3. Configure Your Library

You place a folder containing album's audio files in your library root. To make it visible to the compiler you create `metadata.toml` file in it or run `leland manifest` to read embedded tags and generate manifest from them. In this TOML you have two sections: `[album]` header and multiple of `[[tracks]]` for each audio file. Write all keys in standard `keyname = "Value"` format. The `[album]` header contains metadata *common* across an album (album artist, album title, genre, date, etc.), and each of `[[tracks]]` contains metadata *unique* to each track (track number, disc number, title).

Then you run `leland update`. It automatically finds all new or changed `metadata.toml` files and compiles them with the source files into a `album.lock.json` artifacts.

### 4. Run It

Since the interface is decoupled from the backend server, you run them as separate processes:

```bash
# Terminal 1: Start the Rust backend
leland server
```

```bash
# Terminal 2: Start the default interface (Svelte Web App)
leland interface
```

## CLI Usage

The `leland` CLI tool is the central driver for managing your library's state. 

- `leland manifest` — Scan your library root for unmanaged audio directories and generate the initial `metadata.toml` manifest.
- `leland update` — The core compiler command. Reads your TOML manifests and compiles the `album.lock.json` files.
- `leland server` — Start the Axum backend server.
- `leland interface` — Run interfaces defined in `ll.interfaces`.
- `leland x` — Run defined actions via runtime `ll.actions` router.

## AI Disclosure & Human Design
This software was developed in part with the assistance of LLMs, which were used as a tool for research and as a code-monkey for Rust syntax implementation. All business logic, architecture, UX — including complete creative vision — were designed and vetted with a great deal of intent and hard work by a human — myself. This README and all documentation were handrolled on my keyboard, from my bedroom, in my own words — as I believe this is an honest way to show you that I care about what you'll read here.

## A Note From The Developer
**I am the primary and the most active user of this software.** I am building it for myself, in hopes that **you** will find it useful too. This project was born from the unstoppable love for album collecting and archival, respect for Unix Philosophy, and genuine lack of anything close to "album-as-a-compiled-data-object" in the world of media players. Since I daily-drive it — and don't intend to stop any time soon — I'll try to maintain it as long as I am using Linux and listening to albums. Thank you for reading this README.md and (hopefully) using this software. All feedback is always appreciated.

## License
I want this project to remain free and open source software forever, which is why it is licensed under the [GNU Affero General Public License v3.0](https://www.gnu.org/licenses/agpl-3.0.en.html). Proprietary license exemptions are not offered.
