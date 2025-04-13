#!/bin/bash

# Build script for VirusTotal Scanner App
echo -e "\033[0;32mBuilding VirusTotal Scanner App for all platforms...\033[0m"

# Check if we're on macOS
if [ "$(uname)" == "Darwin" ]; then
    # We're on macOS, so we can build for macOS
    echo -e "\033[0;36mBuilding for all platforms...\033[0m"
    npm run tauri:build
    if [ $? -ne 0 ]; then
        echo -e "\033[0;31mBuild failed\033[0m"
        exit 1
    fi
    echo -e "\033[0;32mBuild completed successfully!\033[0m"
    echo -e "\033[0;32mmacOS builds are available at: src-tauri/target/release/bundle/\033[0m"
else
    # We're not on macOS, so we can't build for macOS
    echo -e "\033[0;33mThis script must be run on a Mac to build macOS bundles\033[0m"
    echo -e "\033[0;33mPlease run this script on a Mac or use the Windows build script on Windows\033[0m"
    exit 1
fi

echo -e "\033[0;32mBuild completed successfully!\033[0m"
