# Installation Guide

## Pre-built Binaries

The easiest way to install fontgrep is to download a pre-built binary from the [releases page](https://github.com/twardoch/fontgrep/releases).

### Linux

```bash
# Download and install (replace with latest version)
curl -L https://github.com/twardoch/fontgrep/releases/latest/download/fontgrep-linux-x86_64.tar.gz | tar -xz
sudo mv fontgrep /usr/local/bin/
```

### macOS

```bash
# Download and install (replace with latest version)
curl -L https://github.com/twardoch/fontgrep/releases/latest/download/fontgrep-macos-x86_64.tar.gz | tar -xz
sudo mv fontgrep /usr/local/bin/
```

For Apple Silicon Macs:
```bash
curl -L https://github.com/twardoch/fontgrep/releases/latest/download/fontgrep-macos-aarch64.tar.gz | tar -xz
sudo mv fontgrep /usr/local/bin/
```

### Windows

1. Download `fontgrep-windows-x86_64.exe.zip` from the [releases page](https://github.com/twardoch/fontgrep/releases)
2. Extract the zip file
3. Add the directory containing `fontgrep.exe` to your PATH

## Package Managers

### Cargo (Rust)

```bash
cargo install fontgrep
```

### Homebrew (macOS/Linux)

```bash
brew install fontgrep
```

### Chocolatey (Windows)

```bash
choco install fontgrep
```

### Scoop (Windows)

```bash
scoop install fontgrep
```

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep

# Build and install
cargo install --path .
```

### Development Build

```bash
# Clone the repository
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep

# Build for development
cargo build

# Run tests
cargo test

# Run the development binary
cargo run -- --help
```

## Verification

After installation, verify that fontgrep is working:

```bash
fontgrep --version
fontgrep --help
```

## Docker

You can also run fontgrep in a Docker container:

```bash
docker run --rm -v $(pwd):/fonts ghcr.io/twardoch/fontgrep:latest /fonts
```

## Troubleshooting

### Permission Issues (Linux/macOS)

If you get permission errors when installing to `/usr/local/bin/`, try:

```bash
# Create the directory if it doesn't exist
sudo mkdir -p /usr/local/bin

# Make sure you have write permissions
sudo chown -R $(whoami) /usr/local/bin
```

### PATH Issues

If `fontgrep` is not found after installation, make sure the installation directory is in your PATH:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="/usr/local/bin:$PATH"
```

### Rust Installation

If you don't have Rust installed, install it from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows PATH

On Windows, you may need to add the fontgrep directory to your PATH:

1. Open System Properties
2. Click "Environment Variables"
3. Under "System variables", find and select "Path"
4. Click "Edit"
5. Click "New" and add the directory containing `fontgrep.exe`

## Uninstallation

### Cargo Installation

```bash
cargo uninstall fontgrep
```

### Manual Installation

Simply remove the `fontgrep` binary from your installation directory:

```bash
sudo rm /usr/local/bin/fontgrep
```

### Package Managers

Use the appropriate uninstall command for your package manager:

```bash
# Homebrew
brew uninstall fontgrep

# Chocolatey
choco uninstall fontgrep

# Scoop
scoop uninstall fontgrep
```