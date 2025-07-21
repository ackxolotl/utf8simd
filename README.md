# utf8simd

A high-performance UTF-8 validation library that uses SIMD operations for fast validation of byte sequences, based on
[simdjson](https://github.com/simdjson/simdjson)'s UTF-8 validation.

## Features

- **High Performance**: Validates cache line aligned 64 bytes per iteration using SIMD vectors
- **ASCII Fast Path**: Single instruction check for pure ASCII input  
- **Cross-Platform**: Uses portable SIMD for compatibility across x86_64 and ARM64
- **No Standard Library**: `no_std` compatible for embedded and constrained environments

## Requirements

This crate requires nightly Rust due to unstable features. The project uses the following unstable features:
- `portable_simd`
- `core_intrinsics` 
- `generic_const_exprs`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
utf8simd = "0.1.0"
```

## Usage

### Basic Usage

The library works as a drop-in replacement for `str::from_utf8()`:

```rust
fn main() -> utf8simd::Result<()> {
    let bytes = b"Hello, world!";
    let str = utf8simd::from_utf8(bytes)?;
    println!("{str}");
    Ok(())
}
```

### Advanced Usage

For certain scenarios, it may be beneficial to use the `Utf8Validator` directly:

```rust
#![feature(portable_simd)]

use core::simd::Simd;
use utf8simd::Utf8Validator;

fn main() -> utf8simd::Result<()> {
    let data = Simd::load_or_default(b"hello world");
    
    let mut validator = Utf8Validator::default();
    validator.next(&data)?;
    
    // always check the last block for incomplete bytes
    validator.finish()
}
```

## Performance

Run benchmarks with:

```bash
RUSTFLAGS="-Ctarget-cpu=native" cargo bench
```

Benchmark results are generated as HTML reports using [criterion.rs](https://github.com/bheisler/criterion.rs).

### Results

Some performance numbers for parsing 1 GB of UTF-8 data:

| CPU               | core::str::from_utf8 | utf8simd::from_utf8 | Speedup |
|-------------------|----------------------|---------------------|---------|
| AMD Ryzen 9 7950X | 5.0 GB/s             | 13.4 GB/s           | 2.7x    |
| Apple M1 8 Core   | 2.5 GB/s             | 9.0 GB/s            | 3.6x    |

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Feel free to submit a pull request.
