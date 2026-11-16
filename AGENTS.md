# Project rules

- Product UI uses Angular Material components with the Material 3 theme; do not recreate component behavior with custom egui widgets.
- Tauri/Rust owns filesystem access, discovery, parsing, diagnostics, and local image loading. The frontend calls explicit commands and never edits game files.
- Localisation text and script-derived relationships are separate. Only show relationships supported by parsed script data; preserve diagnostics for uncertain associations.
- Keep paths platform-neutral with Rust Path APIs. Steam/Workshop discovery is best-effort and manual settings remain available.
- Persist user settings and reading progress locally. Never bundle or distribute game/mod assets.
- Commit the verified changes after completing each task that changes the project. Keep unrelated work out of the task commit.
