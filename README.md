# iMAGE

Photo storage app using GitHub as backend. Part of Vortex Interface.

## Stack

- Tauri 2 (Rust backend)
- Vue 3 + TypeScript
- Tailwind CSS 4
- GitHub API (Device Flow OAuth)

## Features

- Upload photos to any GitHub repo
- Drag & drop support
- LFS support for files >50MB
- AMOLED dark theme with accent colors
- Offline token persistence

## Setup

### Prerequisites

- Node.js 20+
- pnpm
- Rust

### GitHub OAuth

1. Go to https://github.com/settings/developers
2. Create new OAuth App
3. Enable "Device Flow"
4. Copy Client ID to `src-tauri/src/github.rs`:

```rust
const CLIENT_ID: &str = "your_client_id";
```

### Install & Run

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## Project Structure

```
src/
├── components/
│   ├── AccentPicker.vue    # Theme color selector
│   ├── AuthButton.vue      # GitHub login/logout
│   ├── PhotoGallery.vue    # Image grid display
│   ├── PhotoUploader.vue   # Upload queue UI
│   └── SpaceLoader.vue     # Loading animation
├── composables/
│   ├── useAccentColor.ts   # Theme state
│   ├── useGitHubAuth.ts    # OAuth device flow
│   └── usePhotoUpload.ts   # Upload queue logic
├── App.vue
├── main.ts
└── style.css

src-tauri/
├── src/
│   ├── github.rs           # GitHub API + OAuth
│   ├── lib.rs              # Tauri commands
│   └── main.rs
└── tauri.conf.json
```

## How It Works

1. User authenticates via GitHub Device Flow (no client secret needed)
2. Token stored locally via `@tauri-apps/plugin-store`
3. Photos uploaded to `photos/` folder in specified repo
4. Files >50MB use Git LFS

## Config

Settings stored in `settings.json`:
- `token` - GitHub access token
- `repo` - Target repository (owner/repo)
- `accent` - Theme color

## License

MIT
