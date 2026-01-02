# Linux Build Workflow Documentation

## Overview

The GitHub Actions workflow for Linux builds is configured to build release binaries for x86_64 architecture with two different C library variants: GNU libc (glibc) and musl libc.

## Build Matrix

| Platform | Runner | Target Triple | Output Name |
|----------|--------|---------------|-------------|
| Linux x86_64 (GNU) | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `file-organizer-linux-x86_64.tar.gz` |
| Linux x86_64 (musl) | `ubuntu-latest` | `x86_64-unknown-linux-musl` | `file-organizer-linux-x86_64-musl.tar.gz` |

## Why Two Linux Builds?

### GNU libc (glibc)
- **Target:** `x86_64-unknown-linux-gnu`
- **Standard C library** for most Linux distributions
- **Dynamic linking** to system glibc
- **Compatibility:** Requires matching or newer glibc version on target system
- **Size:** Smaller binary (shared libraries)
- **Best for:** Modern Linux distributions (Ubuntu, Debian, Fedora, etc.)

### musl libc
- **Target:** `x86_64-unknown-linux-musl`
- **Lightweight C library** designed for static linking
- **Static linking:** Fully self-contained binary
- **Compatibility:** Runs on any Linux distribution (no glibc dependency)
- **Size:** Larger binary (includes all dependencies)
- **Best for:** Alpine Linux, embedded systems, maximum portability

## Workflow Steps

### 1. Checkout Code
Uses `actions/checkout@v4` to clone the repository.

### 2. Install Rust Toolchain
- Uses `dtolnay/rust-toolchain@stable`
- Installs the stable Rust toolchain
- Adds the appropriate Linux target (GNU or musl)

### 3. Install musl Tools (conditional)
For musl builds only:
```bash
sudo apt-get update && sudo apt-get install -y musl-tools
```

This installs:
- `musl-gcc` - musl C compiler wrapper
- `musl` - musl libc headers and libraries

### 4. Caching
Three separate caches with target-specific keys:
- **Cargo Registry:** `~/.cargo/registry`
- **Cargo Git:** `~/.cargo/git`
- **Build Target:** `target/` (includes target triple in cache key)

**Note:** Cache key includes the target triple to prevent conflicts between GNU and musl builds.

### 5. Build Release Binary
```bash
cargo build --release --target <target-triple>
```

Produces optimized release binaries:
- GNU: `target/x86_64-unknown-linux-gnu/release/file-organizer`
- musl: `target/x86_64-unknown-linux-musl/release/file-organizer`

### 6. Strip Binary
```bash
strip target/<target>/release/file-organizer
```

Removes debug symbols to reduce binary size (typically 20-30% reduction).

### 7. Create Archive
```bash
cd target/<target>/release
tar -czf file-organizer-<name>.tar.gz file-organizer
mv file-organizer-<name>.tar.gz ../../../
```

Creates compressed tar.gz archives (Unix standard).

### 8. Upload Artifact
Uses `actions/upload-artifact@v4` to upload the compressed archive.

## Key Differences from macOS

### Similarities
- Uses `tar.gz` format (same as macOS)
- Includes binary stripping
- Same caching strategy

### Differences
- **musl tools:** Additional package installation for musl builds
- **Cache keys:** Include target triple for build isolation
- **Single OS:** Both builds use `ubuntu-latest` (different targets)

## Cross-Compilation Notes

### Building on macOS
Cross-compiling to Linux from macOS is **not directly supported** because:
- Requires Linux-specific linker
- Requires glibc or musl headers
- Different system call conventions

**Solutions for local testing:**
1. Use Docker with a Linux container
2. Use a Linux VM
3. Use cross-compilation tools (complex setup)
4. Test on GitHub Actions (recommended)

### Building on Linux

#### Install targets
```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
```

#### Install musl tools (for musl target)
```bash
sudo apt-get update
sudo apt-get install -y musl-tools
```

#### Build
```bash
# GNU libc build
cargo build --release --target x86_64-unknown-linux-gnu

# musl build
cargo build --release --target x86_64-unknown-linux-musl
```

#### Create archive
```bash
cd target/x86_64-unknown-linux-gnu/release
tar -czf file-organizer-linux-x86_64.tar.gz file-organizer

cd ../../../target/x86_64-unknown-linux-musl/release
tar -czf file-organizer-linux-x86_64-musl.tar.gz file-organizer
```

#### Test extraction
```bash
tar -xzf file-organizer-linux-x86_64.tar.gz
./file-organizer --version
```

## Verification Status

✅ **YAML Syntax:** Valid
✅ **Workflow Structure:** Correct (parallel builds)
✅ **musl Installation:** Conditionally installed
✅ **Cache Keys:** Include target for isolation
✅ **Archive Format:** tar.gz (Linux standard)
✅ **Binary Stripping:** Included for both variants
⚠️ **Local Build:** Cannot be tested on macOS (requires Linux)

## Expected Outputs

When the workflow runs on GitHub Actions, it will produce two artifacts:

### GNU libc variant
- **Artifact:** `file-organizer-linux-x86_64`
- **Contains:** `file-organizer-linux-x86_64.tar.gz`
- **Binary:** Dynamically linked to glibc
- **Use case:** Standard Linux distributions

### musl libc variant
- **Artifact:** `file-organizer-linux-x86_64-musl`
- **Contains:** `file-organizer-linux-x86_64-musl.tar.gz`
- **Binary:** Statically linked (self-contained)
- **Use case:** Alpine Linux, maximum portability

## GitHub Actions Runner

The `ubuntu-latest` runner includes:
- Ubuntu 22.04 (as of 2024)
- GCC and build-essential
- glibc development headers
- Rust via `rustup` (installed by workflow)
- Standard Unix tools (tar, strip, etc.)

## Binary Comparison

| Aspect | GNU libc | musl libc |
|--------|----------|-----------|
| Dependencies | glibc (dynamic) | None (static) |
| Binary Size | Smaller | Larger |
| Portability | Good (same/newer glibc) | Excellent (any Linux) |
| Performance | Slightly faster | Comparable |
| Memory | Lower (shared libs) | Higher (static) |

## Choosing the Right Binary

### Use GNU libc if:
- Targeting modern Linux distributions
- Users have package managers
- You want smaller downloads

### Use musl if:
- Maximum portability is needed
- Targeting Alpine Linux
- Users may have old glibc versions
- Deploying in containers

## Next Steps

1. ✅ Linux build configuration complete
2. ⏳ Configure release artifact uploads for all platforms (Task 5)
3. ⏳ Test complete workflow (Task 6)
