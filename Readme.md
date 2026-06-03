# yama

yama is a simple, well-architected package manager written in Rust with C FFI integration.

## Features

- **Semver Support**: Resolves dependencies using semantic versioning.
- **Registry Abstraction**: Supports different metadata sources (Mock, Local, Remote).
- **FFI Integration**: Uses a C-based `file_manager` for certain low-level operations.
- **Dependency Resolution**: Recursive resolution of package dependencies.
- **Robust Error Handling**: Clear and actionable error types.
- **Testing**: Includes unit tests for the core logic.

## Architecture

- `yama` (Library): Core logic for resolution, downloading, and extraction.
- `yama-cli` (Binary): Command-line interface for managing packages.
- `file_manager` (C): Static library for platform-specific file tasks.

## Getting Started

### Prerequisites

- Rust (Cargo)
- GCC (for building the C library)

### Building

```bash
cargo build
```

### Usage

```bash
# Install a package
./target/debug/yama install yama

# List installed packages
./target/debug/yama list

# Remove a package
./target/debug/yama remove yama
```

### Running Tests

```bash
cargo test
```

## Fun Facts

- **What's in a name?**: The name "Yama" refers to a significant role player in Hinduism, symbolizing self-discipline and control. Just like its namesake, this package manager aims to bring order and efficiency to your software management tasks.
- **Inspiration**: Yama is inspired by the legendary YUM package manager and aims to bring a modern, Rust-powered touch to package management.
