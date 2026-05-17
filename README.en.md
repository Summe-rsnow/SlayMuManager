[🇬🇧 English](./README.en.md) | [🇨🇳 中文](./README.md)

---

# SlayMuManager

> Slay the Spire 2 Desktop Mod Manager

A cross-platform desktop app built with **Tauri 2.0 + Vue 3 + Rust**, providing mod browsing, installation, management, and save synchronization.

## Features

- **Mod Management** — Enable/disable/uninstall mods, custom tags & notes, quick filtering
- **Preset System** — Save mod combinations as presets, switch with one click
- **Discover** — Browse and install community mods via the Nexus Mods API
- **Save Management** — Separate vanilla/modded saves, pair sync, Steam cloud save checking
- **Import Mods** — Drag-and-drop .zip/.7z files or folders, automatic conflict resolution
- **Theme System** — Light/Dark/System mode, 7 accent colors
- **Multi-language** — 简体中文 / English

## Screenshots

<!-- TODO: Add screenshots -->

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri 2.0 |
| Frontend | Vue 3 (Composition API) + Vite 8 |
| Styling | UnoCSS + Naive UI |
| Icons | Lucide Vue Next |
| State Management | Pinia |
| i18n | Vue I18n |
| Backend | Rust (edition 2021) |
| Package Manager | pnpm |

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (edition 2021)

### Getting Started

```bash
# Install dependencies
pnpm install

# Start dev server
pnpm tauri dev

# Production build
pnpm tauri build
```

## Acknowledgements

- Inspired by [SlaySP2Manger](https://github.com/wakaka6/SlaySP2Manger) — thanks to the open source community

## License

Apache License 2.0 — see [LICENSE](./LICENSE)
