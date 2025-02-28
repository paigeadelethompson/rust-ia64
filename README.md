# Rust IA-64 Emulator

[![CI](https://github.com/iampaigeat/rust-ia64/actions/workflows/ci.yml/badge.svg)](https://github.com/iampaigeat/rust-ia64/actions/workflows/ci.yml)
[![Documentation](https://github.com/iampaigeat/rust-ia64/actions/workflows/docs.yml/badge.svg)](https://iampaigeat.github.io/rust-ia64/)

An emulator for the Intel Itanium (IA-64) architecture written in Rust. This project aims to provide accurate emulation of IA-64 instructions and system behavior.

## Features

- Full IA-64 instruction set emulation
- Register stack engine (RSE)
- Memory management and caching
- System call handling
- Basic I/O operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-ia64 = { git = "https://github.com/iampaigeat/rust-ia64" }
```

## Usage

Here's a basic example of using the emulator:

```rust
use rust_ia64::{Cpu, Memory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cpu = Cpu::new();
    let mut memory = Memory::new();

    // Load program into memory...
    // Execute program...

    Ok(())
}
```

For more examples and detailed documentation, please visit our [documentation page](https://iampaigeat.github.io/rust-ia64/).

## Development

### Prerequisites

- Rust 1.70.0 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting and Formatting

```bash
# Run clippy
cargo clippy

# Format code
cargo fmt
```

## Project Structure

- `src/cpu/` - CPU core implementation
- `src/memory/` - Memory management
- `src/decoder/` - Instruction decoder
- `src/cpu/instructions/` - Instruction implementations

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Intel® Itanium® Architecture Software Developer's Manual
- Rust community and contributors 