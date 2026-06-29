[🇬🇧 English](./README.en.md) | [🇨🇳 中文](./README.md)

---

# SlayMuManager

> Slay the Spire 2 Desktop Mod Manager

A cross-platform desktop app built with **Tauri 2.0 + Vue 3 + Rust**, providing mod browsing, installation, management, and save synchronization.

## Features

- **Mod Management** — Enable/disable/uninstall mods, custom tags & notes, quick filtering
- **Preset System** — Save mod combinations as presets, switch with one click
- **Preset Bundles** — Export presets with associated mods as `.7z` bundles for sharing
- **Discover** — Browse and install community mods via the Nexus Mods API (event-driven backend search, non-blocking UI)
- **Steam Workshop** — Search, subscribe/unsubscribe Workshop mods (requires Steam running)
- **Save Management** — Separate vanilla/modded saves, pair sync, Steam cloud save detection & diff
- **Save Backup** — Manual/auto backup, restore to any slot, auto-clean old backups
- **Import Mods** — Drag-and-drop `.zip`/`.7z`/`.rar` files or folders, automatic conflict resolution
- **Game Launch** — One-click launch (Steam/direct), `--nomods` vanilla mode
- **Mod Update Check** — Compare local vs online file hashes to detect updatable mods
- **Mod Description Translation** — Translate English descriptions to Chinese via backend API
- **Filesystem Watcher** — Auto-detect external changes in `mods/` directory
- **Theme System** — Light/Dark/System mode, 9 accent colors
- **App Auto-Update** — Check for new versions on startup, GitHub Release & netdisk download
- **Activity Log** — Track mod/profile/save operation history
- **Proxy Support** — HTTP/SOCKS proxy for Nexus Mods API
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
| State Management | Pinia (stores) + Composables |
| Router | Vue Router 5 |
| i18n | Vue I18n |
| Backend | Rust (edition 2024, rust-version 1.94.1) |
| Package Manager | pnpm |

### Backend Dependencies

| Purpose | Crate |
|---------|-------|
| Serialization | serde / serde_json |
| HTTP Client | reqwest (blocking, json, socks) |
| Archive Extraction | zip / sevenz-rust |
| File Dialogs | rfd |
| Steam Integration | steamworks |
| File Watcher | notify |
| Registry Query | winreg |
| UUID | uuid (v4) |

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (edition 2024)

### Getting Started

```bash
# Install dependencies
pnpm install

# Start dev server
pnpm tauri dev

# Production build
pnpm tauri build
```

### Project Structure

```
src/                    # Vue frontend source
├── pages/              # Page components (5)
├── components/         # Shared components (28)
├── stores/             # Pinia global state (8)
├── composables/        # Scoped composable logic (4)
├── utils/              # Pure utility functions (4)
├── router/             # Route config
├── i18n/               # Internationalization (zh-CN / en)
├── theme/              # Theme system
└── types/              # TypeScript type definitions

src-tauri/              # Rust backend source
└── src/
    ├── app/            # Tauri commands & state
    ├── domain/         # Domain models
    ├── services/       # Business logic
    ├── repositories/   # Data persistence
    ├── workflows/      # Complex workflows
    ├── integrations/   # External services (Steam/Nexus)
    └── utils/          # Utilities & error handling
```

## Acknowledgements

- Inspired by [SlaySP2Manger](https://github.com/wakaka6/SlaySP2Manger) — thanks to the open source community

## License

Apache License 2.0 — see [LICENSE](./LICENSE)
