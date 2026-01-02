# macOS Build Workflow Documentation

## Overview

The GitHub Actions workflow for macOS builds is configured to build release binaries for both Intel (x86_64) and Apple Silicon (ARM64) architectures.

## Workflow File

**Location:** `.github/workflows/release.yml`

## Trigger Events

- **Tag Push:** Automatically runs when a tag matching `v*` is pushed (e.g., `v1.0.0`, `v0.1.0`)
- **Manual Trigger:** Can be manually triggered via GitHub Actions UI using `workflow_dispatch`

## Build Matrix

The workflow builds for two macOS architectures:

| Platform | Runner | Target Triple | Output Name |
|----------|--------|---------------|-------------|
| Apple Silicon (ARM64) | `macos-latest` | `aarch64-apple-darwin` | `file-organizer-macos-aarch64.tar.gz` |
| Intel (x86_64) | `macos-13` | `x86_64-apple-darwin` | `file-organizer-macos-x86_64.tar.gz` |

## Workflow Steps

### 1. Checkout Code
Uses `actions/checkout@v4` to clone the repository.

### 2. Install Rust Toolchain
- Uses `dtolnay/rust-toolchain@stable`
- Installs the stable Rust toolchain
- Adds the appropriate target for cross-compilation

### 3. Caching
Three separate caches to speed up builds:
- **Cargo Registry:** `~/.cargo/registry`
- **Cargo Git:** `~/.cargo/git`
- **Build Target:** `target/`

Cache keys are based on the `Cargo.lock` hash to ensure cache invalidation when dependencies change.

### 4. Build Release Binary
```bash
cargo build --release --target <target-triple>
```

Produces optimized release binaries in `target/<target>/release/file-organizer`.

### 5. Strip Binary
```bash
strip target/<target>/release/file-organizer
```

Removes debug symbols to reduce binary size (typically 15-20% reduction).

### 6. Create Archive
Creates a compressed `.tar.gz` archive containing the binary:
```bash
tar -czf file-organizer-<platform>.tar.gz file-organizer
```

### 7. Upload Artifact
Uses `actions/upload-artifact@v4` to upload the compressed archive as a workflow artifact.

## Local Testing

### Build for ARM64 (Apple Silicon)
```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

### Build for x86_64 (Intel)
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

### Test Strip and Archive
```bash
# Build
cargo build --release --target aarch64-apple-darwin

# Strip
strip target/aarch64-apple-darwin/release/file-organizer

# Archive
cd target/aarch64-apple-darwin/release
tar -czf file-organizer-macos-aarch64.tar.gz file-organizer

# Test extraction
tar -xzf file-organizer-macos-aarch64.tar.gz
./file-organizer --version
```

## Verification Results

✅ **YAML Syntax:** Valid
✅ **ARM64 Build:** Successful (987KB → 825KB after strip)
✅ **x86_64 Build:** Successful (1.0MB → similar reduction expected)
✅ **Binary Architecture:** Correct (verified with `file` command)
✅ **Archive Creation:** Working (416KB compressed)
✅ **Archive Extraction:** Working
✅ **Binary Execution:** Verified with `--version` flag

## Expected Outputs

When the workflow runs, it will produce two artifacts:
- `file-organizer-macos-aarch64` (contains `file-organizer-macos-aarch64.tar.gz`)
- `file-organizer-macos-x86_64` (contains `file-organizer-macos-x86_64.tar.gz`)

Each archive contains a single executable binary named `file-organizer`.

## Next Steps

1. Add Windows build configuration (Task 3)
2. Add Linux build configuration (Task 4)
3. Configure release artifact uploads for all platforms (Task 5)
4. Test complete workflow (Task 6)
