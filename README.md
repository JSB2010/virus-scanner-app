# VirusTotal Scanner App

A desktop application that continuously monitors your Downloads folder for new files and scans them using the VirusTotal API.

## Features

- Continuous monitoring of Downloads folder
- Scan popups for new files
- Progress bars during scans
- Detailed scan results display
- Virus deletion prompt
- Welcome/setup screens with API key input
- System tray integration
- Modern responsive UI/UX

## Building the App

### Prerequisites

- Node.js (v16 or later)
- npm (v7 or later)
- Rust (latest stable)
- Tauri CLI (`npm install -g @tauri-apps/cli`)

### Building for Windows

```bash
npm run tauri:build:windows
```

This will create:
- An NSIS installer (.exe) at `src-tauri/target/release/bundle/nsis/`
- An MSI installer (.msi) at `src-tauri/target/release/bundle/msi/`

### Building for macOS

To build for macOS, you need to be on a Mac with Xcode installed.

```bash
npm run tauri:build:macos
```

This will create:
- A macOS application (.app) at `src-tauri/target/release/bundle/macos/`
- A DMG disk image (.dmg) at `src-tauri/target/release/bundle/dmg/`
- A macOS installer package (.pkg) at `src-tauri/target/release/bundle/pkg/`

### Building for All Platforms

To build for all supported platforms:

```bash
npm run build:all
```

Note: macOS builds can only be created on a Mac system.

## Development

To run the app in development mode:

```bash
npm run tauri:dev
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
