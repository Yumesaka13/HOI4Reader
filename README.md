# HOI4Reader

**English** · [简体中文](docs/README.zh-Hans.md) · [正體中文](docs/README.zh-Hant.md) · [日本語](docs/README.ja.md)

A desktop story reader for Hearts of Iron IV mods. Pick a country, read its national focuses and events, and choose branches as the story unfolds.

## Home screen

The TFR-themed home screen uses sample data.

![Home screen in the TFR theme](docs/screenshots/tfr-home.png)

## Features

- Detects local Steam libraries and installed Workshop mods. You can also choose folders manually.
- Connects focuses, events, decisions, and world news using the mod's scripts. The reader shows dates and offers choices when the relevant war or political outcome cannot be simulated.
- Saves your place locally and lets you undo a branch choice.
- Shows locally installed illustrations and super events, and plays music from the installed mod or its music addon. New tracks interrupt by default, with an optional queue.
- Offers Material Design and The Fire Rises themes. English is the default. Switch to Simplified Chinese, Traditional Chinese, or Japanese in Settings. Story text comes from installed localisation, with English fallback where available.

## Mod support

**The Fire Rises (TFR)** is the primary supported mod and has the most complete reading integration. The New Order can be discovered and indexed, with more limited route coverage. HOI4Reader interprets selected script structures. Some events still lack a verified trigger or date and appear as story leads.

## Run from source

Install Node.js, Rust and the [Tauri system prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm --prefix frontend ci
npm run tauri:dev
```

Run `npm run tauri:build` to build for the current platform. For checks, run `cargo test` and `npm --prefix frontend test -- --watch=false`.

The app reads game files and mod files locally. They are not included in this repository. The bundled [Fusion Pixel font](https://github.com/TakWolf/fusion-pixel-font) is distributed under the included [font license](frontend/public/fonts/OFL.txt).
