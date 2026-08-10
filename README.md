# Dale: Data-driven Album Library Engine

**Dale** is an MPD client and album-centric music library engine built from the first Unix Philosophy principles for archivist-minded collectors. It brings full Lua scriptability, plain-text data management, and Ahead-Of-Time library compilation guarantees to your album collection.

> This README and all documentation were written by me, the developer, with my own hands, on a physical keyboard, in my own words. Thank you for reading it.

## Quick Project Rundown

**Dale** is built around the compiler architecture and the radical separation of concerns. Here's the quick rundown:

### Principles

- **The Album as The Fundamental Unit.** This project focuses solely on collection and management of music albums. The point, I guess, is to bring back the feeling of physical collecting to the digital world. The importance of the album for this project reflects how you collect music in real life.
- **Immutable Audio / Mutable Metadata.** Audio files making up the album should be a bit-perfect preservation of the original media. Audio files are inherently static. Your metadata is inherently dynamic. The engine treats audio as a read-only source. Everything mutable is expected to exist as ancillary files alongside it.
- **Power to The User.** The entire creative vision of this project was conceived around a stance: you should not be bound by your collection interface choices. Every active decision made for this project reflects it.

### Architecture

- **Complete plain-text power.** Store anything that you care about in plain-text alongside the album. Custom date added values, lyrics, your own personal notes, source URLs, ReplayGain values, static album analysis results, *anything*. Edit these files in Neovim, RipGrep/Sed them. Plain-text is *the* universal interface.
- **Album as a Compiled Data Object.** Take these metadata files, take the audio source, the cover, and *compile* them. Result: machine-readable, standardized `album.lock.json`. Use this object to interface the album in any way imaginable.
- **Database-less storage.** No opaque SQLite databases somewhere in media player cache. 1 Album = 1 Directory = 1 Source of Truth. Everything in plain sight. Zero lock-in.
- **Decoupled Backend and Frontend.** Dale is the Rust web server first — the user interface intentionally comes second. The interface choice is yours. You can write interfaces in any language that supports Web API communications with the running backend. You can build TUI apps, Godot based game-interfaces, or you can even straight up use Curl. The project's goal is to provide robust primitives and allow you to build upon them.

## Why This Way?

The reasons behind architecture choices are a few cool features they unlock:

- **Version control of your metadata with Git.** Never lose your metadata after a batch edit ever again. *Or ever again, period.* Something went wrong? Just `git reset --hard`. Upload it to a remote repo and then nothing can take it away from you.
- **Full Lua scriptability at compile/interface time.** Control the data flow from ancillary files to `album.lock.json` before it hits the interface. Control the logic of how albums are separated by virtual libraries, filtered, grouped, and ordered *inside* the interface. All done in a god-tier scripting language — Lua.
- **Ahead-Of-Time Compilation Guarantees.** Since each album is compiled ahead-of-time you can enforce all cool compile time features AOT programming languages have. Type check your metadata, lint it, standardize its structure, validate it. All can be expressed in Lua as well.
- **Actions.** If every album is a JSON file — then every album is scriptable. An **action** is a standalone executable that reads intermediary JSON from stdin provided by Dale at runtime. Infinitely expand your library management functionality in Unix Philosophy style. Each action can be called by an `/api/actions/{action_name}/` endpoint from any current or future interface.

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
git clone https://github.com/lewelove/dale.git
cd dale
```

Drop into the development shell:

```bash
nix develop
```

Or if you have `direnv` installed:

```bash
direnv allow
```

For default interface to run from cloned developer repo you must install its dependencies and make interface run scripts executable:

```bash
cd interfaces/web-app
bun install
chmod +x run_dev.sh run_prod.sh
```

Build the Rust binary, Web App, and all Actions:

```bash
build
```

The `build` places `dale` executable at `{repo_path}/rust/target/release/dale`. Alias this path in your shell of choice to `dale` for future use.

### 2. Configure It

You create `~/.config/dale/init.lua` file:

```lua
-- Since we are using Lua, a truly god-tier config language, for convenience
-- you can define cloned repository path as local string variable
local repo_dir = "Path/To/Cloned/Repo/"

dl.config({ 
  storage = { 
    -- Define a music directory path containing all your albums,
    -- it must match an MPD's config `music_directory` path
    music_directory = "Path/To/Your/Music/Directory/"
  }
})

-- Optionally you can define all keys besides standard ones
-- you want to load from TOML manifests and be present in `album.lock.json`

-- [album.lock.json].album.keys level
dl.compile.album.key( "album_key_name", true )

-- [album.lock.json].tracks[].keys level
dl.compile.tracks.key( "track_key_name", true )

-- For `dale interface` command to run you point default interface
-- to the `interfaces/web-app` directory from the previous step
-- and specify the run-production-build script
dl.interface( "default", {
  -- Quite handy!
  directory = repo_dir .. "interfaces/web-app/",
  run = repo_dir .. "interfaces/web-app/run_prod.sh"
})
```

For the config reference check out [my dale dotfiles](https://github.com/lewelove/nix-config/tree/main/dotfiles/.config/dale). The config documentation is coming soon...

### 3. Configure Your Library

You place a folder containing album's audio files in your library root. To make it visible to the compiler you create `metadata.toml` file in it or run `dale manifest` to read embedded tags and generate manifest from them. In this TOML you have two sections: `[album]` header and multiple of `[[tracks]]` for each audio file. Write all keys in standard `keyname = "Value"` format. The `[album]` header contains metadata *common* across an album (album artist, album title, genre, date, etc.), and each of `[[tracks]]` contains metadata *unique* to each track (track number, disc number, title).

Then you run `dale update`. It automatically finds all new or changed `metadata.toml` files and compiles them with the source files into a `album.lock.json` artifacts.

### 4. Run It

Since the interface is decoupled from the backend server, you run them as separate processes:

```bash
# Terminal 1: Start the Rust backend
dale server
```

```bash
# Terminal 2: Start the default interface (Svelte Web App)
dale interface
```

## CLI Usage

The `dale` CLI tool is the central driver for managing your library's state. 

- `dale manifest` — Scan your library root for unmanaged audio directories and generate the initial `metadata.toml` manifest.
- `dale update` — The core compiler command. Reads your TOML manifests and compiles the `album.lock.json` files.
- `dale server` — Start the Axum backend server.
- `dale interface` — Run interface defined with `dl.interface`.
- `dale x` — Run defined action via runtime `dl.action` router.

## AI Disclosure & Human Design
This software was developed in part with the assistance of LLMs, which were used as a tool for research and as a code-monkey for Rust syntax implementation. All business logic, architecture, UX — including complete creative vision — were designed and vetted with a great deal of intent and hard work by a human — myself. All text you'll read here was written by me also, as I believe this is an honest way to show you that I care.

## A Note From The Developer
**I am the primary and the most active user of this software.** I am building it for myself, in hopes that **you** will find it useful too. This project was born from the unstoppable love for album collecting and archival, respect for Unix Philosophy, and genuine lack of anything close to "album-as-a-compiled-data-object" in the world of media players. Since I daily-drive it — and don't intend to stop any time soon — I'll try to maintain it as long as I am using Linux and listening to albums. Thank you for reading this README.md and (hopefully) using this software. All feedback is always appreciated.

## License
Dale is free software and I want it to remain free forever, which is why it is licensed under the [GNU Affero General Public License v3.0](https://www.gnu.org/licenses/agpl-3.0.en.html). Proprietary license exemptions are not offered.
