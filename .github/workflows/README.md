# GitHub Actions Workflows

## Release Workflow

**File:** `release.yml`

### Purpose
Builds cross-platform release binaries for the file-organizer CLI tool.

### Current Status
✅ macOS builds configured (ARM64 + Intel)
✅ Windows builds configured (x86_64 MSVC)
✅ Linux builds configured (GNU + musl)
✅ Automatic release creation and artifact uploads

### Triggering the Workflow

#### Automatic (Tag Push)
```bash
git tag v0.1.0
git push origin v0.1.0
```

#### Manual Trigger
1. Go to Actions tab in GitHub
2. Select "Release" workflow
3. Click "Run workflow"
4. Select branch and click "Run workflow"

### Outputs

#### Workflow Artifacts (All Runs)
- `file-organizer-macos-aarch64.tar.gz` (Apple Silicon)
- `file-organizer-macos-x86_64.tar.gz` (Intel Mac)
- `file-organizer-windows-x86_64.zip` (Windows x64)
- `file-organizer-linux-x86_64.tar.gz` (Linux x64 with glibc)
- `file-organizer-linux-x86_64-musl.tar.gz` (Linux x64 with musl - static)

#### GitHub Release (Tag Pushes Only)
When a version tag is pushed (e.g., `v1.0.0`), a GitHub release is automatically created with:
- All 5 platform binaries
- `SHA256SUMS` file for verification
- Auto-generated release notes

### Local Testing

Before pushing, test builds locally:

```bash
# Install targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Build both architectures
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Verify binaries
file target/aarch64-apple-darwin/release/file-organizer
file target/x86_64-apple-darwin/release/file-organizer
```

### Workflow Features

- ✅ Multi-architecture builds using matrix strategy
- ✅ Cargo dependency caching for faster builds
- ✅ Binary stripping for smaller file sizes
- ✅ Compressed archives for distribution
- ✅ Error handling with `if-no-files-found: error`
- ✅ Automatic GitHub release creation on tag push
- ✅ SHA256 checksum generation for verification

### Documentation

#### Platform-Specific
- [macOS Builds](../MACOS_WORKFLOW.md) - ARM64 and Intel architecture details
- [Windows Builds](../WINDOWS_WORKFLOW.md) - MSVC toolchain and ZIP packaging
- [Linux Builds](../LINUX_WORKFLOW.md) - GNU libc vs musl libc comparison

#### Release Management
- [Release Configuration](../RELEASE_CONFIGURATION.md) - Automatic release creation and distribution
- [Complete Build Matrix](../COMPLETE_BUILD_MATRIX.md) - Overview of all platforms
