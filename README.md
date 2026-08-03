# Vellum

Vellum is an MPD client and album-centric library manager built from the first Unix Philosophy principles for archivist-minded collectors.

> **Vellum** is prepared animal skin or membrane, typically used as writing material. — [Wikipedia](https://en.wikipedia.org/wiki/Vellum)

## Philosophy

- **The Album as The Fundamental Collection Unit.** Vellum focuses solely on collection and management of music albums. The point, I guess, is to bring back the feeling of physical collecting to the digital world. An album is the fundamental unit of Vellum because an album is the fundamental unit of any music collection in real life.

- **Immutable Audio / Mutable Metadata.** Audio files making up the album should be a bit-perfect preservation of the original media. Audio files are inherently static; Your metadata is inherently dynamic. This is the reason why Vellum treats audio strictly as a read-only source and separates everything mutable into separate ancillary files.

- **Power to The User.** The whole point of Unix Philosophy in Vellum is that you are not bounded by the collection interface choices. Plain-text is *the* universal interface. To build upon it is to bring raw power and future-proof compatibility for the decades to **your** collection.

## Cool Features

### Everything in Plaintext
Entire library metadata — from song names and album lengths in milliseconds — to custom album source URLs and ReplayGain values — to literally **anything specific that can exist in a text form describing an album in your collection** will be stored and compiled within ancillary plain-text files. Edit them in Neovim, RipGrep/Sed them, run scripts against them. Everything can be version controlled, every change can be tracked, backed up and reverted, completely independent of the audio's embedded tags — in human readable database-less format. Once your collection's metadata hits Git and is uploaded to a remote repo you will never lose it ever again.

### Album as a Compiled Data Object
For the analogy's sake let's imagine an album directory as an entry in the physical archive. This entry contains data written with human intent (`metadata.toml` and other TOML manifests) — as well as the source you're trying to preserve (audio, cover art, lyrics) and its inherit metadata (audio bit depth, duration, etc...). Then you take all of these and run a compiler against them to produce an album's **index** in this imagined archive. By thinking about an album in this way, it stops being an opaque fuzzy object interpreted by each different media player on the fly, and becomes a simple set of data points that be can compiled into a standardized machine-readable data object (`album.lock.json`), which is then read by Vellum server to register it in your collection and to provide data for any further user-album interfacing. The compilation step also brings you all of the cool compile time features AOT programming languages have. Type checking, correctness enforcement, linting, key-to-manifest binding and validation — all of these can be expressed in the Vellum Lua config.

### Decoupled Frontend and Backend
**Vellum is the Rust web server first — the User Interface intentionally comes second.** The separation of concerns is essential in Unix Philosophy. Want to change UI theme? Want to add some cool display feature? No need to worry. You can directly edit contents of the `web-app/` or completely rewrite your own UI in a WebDev stack and run it in a browser — wiring it up to a running Vellum server using its Web API **today**. Furthermore, literally any UI framework supporting Web API functionality can control MPD and retrieve library and album data through Vellum. You can build TUI apps, Godot based game-interfaces, or you can even use Curl to control it if you really want to. The project's goal is to provide robust primitives, so you can interface your album collection in any weird & brilliant way possible.

### Vellum Actions
Since every album is compiled into a plaintext JSON — every album becomes scriptable. **Vellum Action** is a concept of a standalone executable that reads intermediary JSON from stdin (provided by Vellum and populated with albums and config data at runtime) and performs some kind of logic based on this data. That's it. You can write actions in any language that supports reading JSONs (or even in simple shell scripts with Jq) and use them to infinitely expand Vellum functionality in Unix Philosophy style. Each action is configurable via its own CLI arguments and `vellum.lua` config. Every action is callable by its own `/api/actions/{action_name}/` endpoint, so they can be wired up to and executed from any future interface. For built-in actions and more context of what they may be useful for look into the `actions/` directory.

## Interface Showcase

Vellum has the default Web-App interface written in fast and fine-grained reactive Svelte framework. You can run it as a standalone headless process and access it via any browser.

### Home Album Grid

Filter, group and sort albums by any key imaginable. The grid uses slot based row snapping and 60fps physical smooth scrolling.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/472b6151-ad39-4458-a408-a2a45c14f177" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/c3ebe389-1c40-431a-8509-05118ae92b66" />

### Modal Album Drawer

Album preview, supporting multi-disc issues.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/0e8471d7-b695-43fb-81e7-fdf58f317359" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/603ea8af-99e2-4ef1-88ee-3fbae9591f90" />

### Playing Now & The Queue

Provide any GLSL shader as the background with custom palettes for each album separately via config and `theme.toml` album manifest.

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/b911f2a9-bf33-446b-a1f1-c5131cd8972f" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/57db8e38-9189-4494-99f6-c2f1cf68aa56" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/685b24ea-61a1-4a08-b2de-ea0fb0e8961f" />

<img width="1920" height="1080" alt="Image" src="https://github.com/user-attachments/assets/574e7814-7744-4cc8-898f-df8c617a6650" />

## Getting Started

**Prerequisites:** 

- Nix
- Bun
- An active MPD instance

Vellum is in the state of active development. To ensure a reproducible environment it is managed by a Nix Flake. All further setup will be based on Nix prerequisite. All of the setup can be achieved without Nix, simply by having Cargo & Bun in shell, just not reproducibly.

### 1. Setup the Environment

Clone the repository:

```bash
git clone https://github.com/lewelove/vellum.git
cd vellum
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

Build the Rust binary, Web App, as well as all of the Vellum Actions:

```bash
build
```

The `vellum` executable will be located at `./rust/target/release/vellum`. Alias this path in your shell of choice for further use.

### 2. Configure Vellum

You create `~/.config/vellum/vellum.lua` file:

```lua
-- Since we are using Lua, a truly god-tier config language, for convenience
-- you can define cloned repository path as local string variable
local repo_dir = "Path/To/Cloned/Repo/"

vl.config({ 
  storage = { 
    -- Define a library path containing all your albums
    library = "Path/To/Your/Library/Root/"
  }
})

-- Optionally you can define all keys besides standard ones
-- you want to load from toml manifests and be present in `album.lock.json`

-- [album.lock.json].album.keys level
vl.compiler.album.key({
  album_key_name = true
})

-- [album.lock.json].tracks[].keys level
vl.compiler.tracks.key({ 
  track_key_name = true
})

-- For `vellum interface` command to run you point default interface
-- to the `interfaces/web-app` directory from the previous step,
-- as well as run-production-build script
vl.interfaces({ default = {
  -- Quite handy!
  directory = repo_dir .. "interfaces/web-app/"
  run = repo_dir .. "interfaces/web-app/run_prod.sh"
}})
```

For config reference check out [my Vellum dotfiles](https://github.com/lewelove/nix-config/tree/main/dotfiles/.config/vellum). The config documentation is coming soon...

### 3. Configure Your Library

You place a folder containing album's audio files in your library root. To make it visible to Vellum you create `metadata.toml` file in it or run `vellum manifest` to read embedded tags and generate manifest from them. In this toml you have two sections: `[album]` header and multiple of `[[tracks]]` for each audio file. Tags are expressed in standard `keyname = "Value"` format. The `[album]` header contains metadata *common* across an album (album artist, album title, genre, date, etc.), and each of `[[tracks]]` contains metadata *unique* to each track (track number, disc number, title).

Then you run `vellum update`. It automatically finds all new or changed `metadata.toml` files and compiles them with the source files into a `album.lock.json` artifacts.

### 4. Run It

Because Vellum decouples the interface from the backend server, you will run them as separate processes:

```bash
# Terminal 1: Start the Rust backend
vellum server
```

```bash
# Terminal 2: Start the default interface (Svelte Web App)
vellum interface
```

## CLI Usage

The `vellum` CLI tool is the central driver for managing your library's state. 

- `vellum manifest` — Scan your library root for unmanaged audio directories and generate the initial `metadata.toml` manifest.
- `vellum update` — The core compiler command. Reads your TOML manifests and compiles the `album.lock.json` files.
- `vellum server` — Start the Axum backend server.
- `vellum interface` — Run interfaces defined in `vl.interfaces`.
- `vellum x` — Run defined actions via runtime `vl.actions` router.

## AI Disclosure & Human Design
This software was partially developed with the assistance of LLMs for research and Rust syntax implementation. Regardless of this fact, all of the business logic, architecture, UX — as well as complete creative vision — were designed and vetted with a great deal of intent and hard work by a human — myself. All of the documentation was handrolled on my keyboard, from my bedroom, in my own words — as I believe this is an honest way to show you that I care about what you'll read here.

## A Note From The Developer
**I am the primary and the most active user of Vellum.** I am building it for myself, in hopes that **you** will find it useful too. This project was born from the inexorable love for album collecting and archival, respect for Unix Philosophy, and genuine lack of anything close to "album-as-a-compiled-data-object" in the world of media players. I want Vellum to be free and open source forever, and the AGPL-3.0 license is here for it. I'll try to commit maintaining it as long as I will be using Linux and collecting and listening to albums, which gives us, according to average life expectancy, around 50+ years of time. Thank you for reading this README.md and (hopefully) using this software. All feedback is always appreciated.
