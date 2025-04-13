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

## Testing the App

To test the app, you can run unit tests and end-to-end tests.

### Running Unit Tests

```bash
npm test
```

### Running End-to-End Tests

```bash
npx playwright test
```

## Developing the App

To develop the app, follow these steps:

1. Clone the repository:

```bash
git clone https://github.com/JSB2010/virus-scanner-app.git
cd virus-scanner-app
```

2. Install dependencies:

```bash
npm install
```

3. Run the app in development mode:

```bash
npm run tauri:dev
```

## Installing the App

To install the app, follow these steps:

### Prerequisites

- Node.js (v16 or later)
- npm (v7 or later)
- Rust (latest stable)
- Tauri CLI (`npm install -g @tauri-apps/cli`)

### Installation Steps

1. Clone the repository:

```bash
git clone https://github.com/JSB2010/virus-scanner-app.git
cd virus-scanner-app
```

2. Install dependencies:

```bash
npm install
```

3. Build the app for your platform:

- For Windows:

```bash
npm run tauri:build:windows
```

- For macOS:

```bash
npm run tauri:build:macos
```

- For all platforms:

```bash
npm run build:all
```

## Repository Structure

The repository is structured as follows:

- `src/`: Contains the main source code for the app.
- `src-tauri/`: Contains the Tauri-specific code and configuration.
- `e2e-tests/`: Contains end-to-end tests for the app.
- `build-all.ps1`: Script to build the app for all platforms on Windows.
- `build-all.sh`: Script to build the app for all platforms on macOS/Linux.
- `README.md`: This file, providing an overview of the project.
- `package.json`: Contains the project's metadata and dependencies.
- `jest.config.js`: Configuration for running unit tests with Jest.
- `playwright.config.js`: Configuration for running end-to-end tests with Playwright.

## Contributing to the Project

We welcome contributions to the project! To contribute, follow these guidelines:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes and commit them with a clear message.
4. Push your changes to your fork.
5. Create a pull request to the main repository.

Please ensure that your code follows the project's coding standards and includes appropriate tests.

Thank you for contributing!
