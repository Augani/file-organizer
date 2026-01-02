# Windows Build Workflow Documentation

## Overview

The GitHub Actions workflow for Windows builds is configured to build release binaries for the x86_64 architecture using the MSVC toolchain.

## Build Configuration

| Platform | Runner | Target Triple | Output Name |
|----------|--------|---------------|-------------|
| Windows x86_64 | `windows-latest` | `x86_64-pc-windows-msvc` | `file-organizer-windows-x86_64.zip` |

## Key Differences from macOS/Linux

### 1. Binary Extension
Windows executables have the `.exe` extension:
- Binary path: `target/x86_64-pc-windows-msvc/release/file-organizer.exe`

### 2. Archive Format
Windows uses ZIP instead of tar.gz:
- Uses PowerShell's `Compress-Archive` cmdlet
- Output: `.zip` files instead of `.tar.gz`

### 3. No Strip Step
The Windows build does not include a binary stripping step because:
- Windows binaries are already optimized by the MSVC linker
- The `strip` command is not available by default on Windows runners
- The size difference is minimal with MSVC release builds

### 4. PowerShell Shell
The archive creation step uses PowerShell (`shell: pwsh`) to ensure:
- Cross-platform PowerShell Core compatibility
- Native ZIP compression support
- Proper handling of Windows paths

## Workflow Steps

### 1. Checkout Code
Uses `actions/checkout@v4` to clone the repository.

### 2. Install Rust Toolchain
- Uses `dtolnay/rust-toolchain@stable`
- Installs the stable Rust toolchain with MSVC support
- Target: `x86_64-pc-windows-msvc`

### 3. Caching
Three separate caches to speed up builds:
- **Cargo Registry:** `~/.cargo/registry`
- **Cargo Git:** `~/.cargo/git`
- **Build Target:** `target/`

### 4. Build Release Binary
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

Produces optimized release binary: `target/x86_64-pc-windows-msvc/release/file-organizer.exe`

### 5. Create ZIP Archive
```powershell
cd target/x86_64-pc-windows-msvc/release
Compress-Archive -Path file-organizer.exe -DestinationPath file-organizer-windows-x86_64.zip
Move-Item file-organizer-windows-x86_64.zip ../../../
```

### 6. Upload Artifact
Uses `actions/upload-artifact@v4` to upload the ZIP archive as a workflow artifact.

## Cross-Compilation Notes

### Building on macOS/Linux
Cross-compiling to Windows from macOS or Linux is **not supported** in this workflow because:
- Requires MSVC linker (`link.exe`)
- Requires Windows SDK headers
- Complex setup for cross-compilation tools (e.g., `xwin`, `wine`)

The workflow uses native Windows runners (`windows-latest`) which come pre-configured with:
- Visual Studio Build Tools
- MSVC compiler and linker
- Windows SDK
- Rust toolchain support

### Alternative: MinGW Target
If cross-compilation is needed, the `x86_64-pc-windows-gnu` target can be used with MinGW:
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

However, MSVC is preferred for Windows builds because:
- Better integration with Windows APIs
- Smaller binary sizes
- Better debugging support on Windows
- Official Microsoft toolchain

## Local Testing (Windows Only)

On a Windows machine with Rust installed:

```powershell
# Build
cargo build --release --target x86_64-pc-windows-msvc

# Create archive
cd target/x86_64-pc-windows-msvc/release
Compress-Archive -Path file-organizer.exe -DestinationPath file-organizer-windows-x86_64.zip

# Test extraction
Expand-Archive file-organizer-windows-x86_64.zip -DestinationPath test
./test/file-organizer.exe --version
```

## Verification Status

✅ **YAML Syntax:** Valid
✅ **PowerShell Commands:** Verified correct syntax
✅ **Archive Format:** ZIP (standard for Windows)
✅ **Binary Extension:** `.exe` properly specified
⚠️ **Local Build:** Cannot be tested on macOS (requires Windows runner)

## Expected Output

When the workflow runs on GitHub Actions, it will produce:
- Artifact: `file-organizer-windows-x86_64`
- Contains: `file-organizer-windows-x86_64.zip`
- Archive contents: `file-organizer.exe`

## GitHub Actions Runner

The `windows-latest` runner includes:
- Windows Server 2022 (as of 2024)
- Visual Studio 2022 Build Tools
- Rust via `rustup` (installed by workflow)
- PowerShell Core 7.x
- Git for Windows

## Next Steps

1. Add Linux build configuration (Task 4)
2. Configure release artifact uploads for all platforms (Task 5)
3. Test complete workflow (Task 6)
