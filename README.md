# n — Interactively Manage Your Node.js Versions

`n` is a simple, no-fuss Node.js version manager. Download, cache, and switch between Node.js versions with a single command.

## Features

- Install any released Node.js version by number, alias, or LTS codename
- Interactive version picker (arrow keys)
- Version caching — no re-downloading
- Symlink-based activation (no subshells, no profile magic)
- List local and remote versions
- Run a specific version without activating it
- LTS-aware: `n lts` always gives you the latest LTS

## Supported Platforms

| OS      | Architectures      |
| ------- | ------------------ |
| Linux   | x64, arm64, armv7l |
| macOS   | x64, arm64         |
| Windows | x64                |

## Installation

### Pre-built binary (no Rust required)

```bash
curl -fsSL https://raw.githubusercontent.com/THernandez03/n/main/install.sh | sh
```

This installs `n` to `~/.local/bin/n`. You can override the destination:

```bash
INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/THernandez03/n/main/install.sh | sh
```

### From source (requires Rust)

```bash
cargo install --git https://github.com/THernandez03/n
```

### Manual

Download the latest binary from [Releases](https://github.com/THernandez03/n/releases) and place it in your `PATH`.

## Setup

Add `~/.n/bin` to your `PATH`:

```bash
# bash / zsh
export N_PREFIX="$HOME/.n"
export PATH="$HOME/.local/bin:$PATH"  # for the n binary
export PATH="$N_PREFIX/bin:$PATH"     # for managed Node.js binaries
```

Optional environment variables:

| Variable      | Default         | Description                          |
| ------------- | --------------- | ------------------------------------ |
| `N_PREFIX`    | `~/.n`          | Root installation prefix             |
| `N_CACHE_DIR` | `~/.n/versions` | Where downloaded versions are stored |

## Usage

```bash
# Install and activate a version
n 20
n 20.11.0
n lts
n latest

# Interactive picker from cached versions
n

# List cached versions
n ls

# List remote versions
n ls-remote

# Fetch into cache without activating
n fetch 18

# Show path to a cached node binary
n which 20.11.0

# Run a specific version
n run 18 -- --version

# Remove a cached version (interactive picker if no version given)
n remove v18.0.0
n rm v18.0.0        # alias

# Remove all except active
n prune

# Also remove the active version
n prune --force

# Show info
n info

# Update n itself
n update

# Fully remove n + all cached versions (requires confirmation)
n uninstall
n uninstall --yes   # skip confirmation prompt
```

## Version Aliases

| Alias     | Resolves to                     |
| --------- | ------------------------------- |
| `lts`     | Latest LTS release              |
| `stable`  | Same as `lts`                   |
| `latest`  | Newest release (may not be LTS) |
| `current` | Same as `latest`                |
| `canary`  | Same as `latest`                |
| `next`    | Same as `latest`                |
| `20`      | Latest release in major 20      |
| `20.x`    | Same as `20`                    |
| `20.11`   | Latest patch in 20.11           |

## How It Works

`n` downloads prebuilt Node.js tarballs from [nodejs.org](https://nodejs.org/dist/), caches them under `~/.n/versions/<tag>/`, and creates a symlink at `~/.n/bin/node` pointing to the selected version.

No subshells. No profile setup. Just a symlink.

## Related Projects

| Project                                | Runtime              |
| -------------------------------------- | -------------------- |
| [b](https://github.com/THernandez03/b) | Bun version manager  |
| [z](https://github.com/THernandez03/z) | Zig version manager  |
| [d](https://github.com/THernandez03/d) | Deno version manager |
| [r](https://github.com/THernandez03/r) | Rust version manager |

## License

MIT
