# Release Configuration Documentation

## Overview

The GitHub Actions workflow now includes automatic release creation and artifact uploads when version tags are pushed to the repository.

## Release Job Configuration

### Job: `create-release`

**Purpose:** Automatically create GitHub releases with all platform binaries when a version tag is pushed.

**Dependencies:** All three build jobs must complete successfully
- `build-macos`
- `build-windows`
- `build-linux`

**Trigger Condition:** Only runs when pushing tags (e.g., `v1.0.0`, `v0.1.0`)
```yaml
if: startsWith(github.ref, 'refs/tags/')
```

**Permissions:** Requires write access to repository contents
```yaml
permissions:
  contents: write
```

## Release Process

### Step 1: Checkout Code
Clones the repository to access release notes and documentation.

### Step 2: Download All Artifacts
Downloads all 5 build artifacts from previous jobs:
- `file-organizer-macos-aarch64`
- `file-organizer-macos-x86_64`
- `file-organizer-windows-x86_64`
- `file-organizer-linux-x86_64`
- `file-organizer-linux-x86_64-musl`

### Step 3: Display Artifact Structure
Lists all downloaded artifacts for debugging purposes.

### Step 4: Prepare Release Assets
Collects all archive files from artifacts into a single directory:
```bash
mkdir -p release-assets
find artifacts -type f \( -name "*.tar.gz" -o -name "*.zip" \) -exec cp {} release-assets/ \;
```

### Step 5: Generate Checksums
Creates SHA256 checksums for all release assets:
```bash
cd release-assets
sha256sum * > SHA256SUMS
```

This allows users to verify download integrity.

### Step 6: Create GitHub Release
Uses `softprops/action-gh-release@v2` to:
- Create a new release for the tag
- Upload all 5 platform archives
- Upload SHA256SUMS file
- Generate release notes automatically from commit history
- Mark as non-draft, non-prerelease

## Creating a Release

### Method 1: Tag and Push (Recommended)

```bash
# Create a new version tag
git tag v1.0.0

# Push the tag to trigger the release workflow
git push origin v1.0.0
```

### Method 2: Create Tag on GitHub

1. Go to repository on GitHub
2. Click "Releases" → "Create a new release"
3. Click "Choose a tag" → Type new tag (e.g., `v1.0.0`)
4. Click "Create new tag on publish"
5. Fill in release title and description (optional, will be auto-generated)
6. Click "Publish release"

Note: Creating a tag via GitHub UI will also trigger the workflow.

## Release Artifacts

Each release includes:

### Platform Binaries (5 files)
1. `file-organizer-macos-aarch64.tar.gz` - macOS Apple Silicon
2. `file-organizer-macos-x86_64.tar.gz` - macOS Intel
3. `file-organizer-windows-x86_64.zip` - Windows 64-bit
4. `file-organizer-linux-x86_64.tar.gz` - Linux with glibc
5. `file-organizer-linux-x86_64-musl.tar.gz` - Linux with musl (static)

### Checksum File (1 file)
6. `SHA256SUMS` - SHA256 checksums for all binaries

### Auto-Generated Content
- Release notes generated from commits since last tag
- Tag name used as release title

## Verifying Downloads

Users can verify downloaded files using the checksums:

```bash
# Download the binary and checksums file
# Then verify:
sha256sum -c SHA256SUMS --ignore-missing
```

Example output:
```
file-organizer-linux-x86_64.tar.gz: OK
```

## Release Configuration Options

### Current Settings

| Option | Value | Description |
|--------|-------|-------------|
| `draft` | `false` | Immediately publish release |
| `prerelease` | `false` | Mark as stable release |
| `generate_release_notes` | `true` | Auto-generate from commits |
| `fail_on_unmatched_files` | `true` | Fail if any file missing |

### Customization Options

To create draft releases (for review before publishing):
```yaml
draft: true
```

To mark as pre-release (beta, alpha, etc.):
```yaml
prerelease: true
```

To provide custom release notes:
```yaml
body: |
  ## What's New
  - Feature 1
  - Feature 2
```

To use release notes from a file:
```yaml
body_path: RELEASE_NOTES.md
```

## Workflow Execution Flow

```
Tag Push (v1.0.0)
    ↓
Trigger Workflow
    ↓
┌───────────────┬──────────────┬──────────────┐
│               │              │              │
│  build-macos  │ build-windows│ build-linux  │ (Parallel)
│  (2 configs)  │  (1 config)  │ (2 configs)  │
│               │              │              │
└───────┬───────┴──────┬───────┴──────┬───────┘
        │              │              │
        └──────────────┴──────────────┘
                       ↓
              create-release (Sequential)
                       ↓
         ┌─────────────────────────┐
         │ 1. Download artifacts   │
         │ 2. Prepare assets       │
         │ 3. Generate checksums   │
         │ 4. Create GitHub release│
         │ 5. Upload all files     │
         └─────────────────────────┘
                       ↓
            Release Published! 🎉
```

## Manual Workflow Dispatch

The workflow can also be triggered manually without creating a release:

1. Go to Actions tab on GitHub
2. Select "Release" workflow
3. Click "Run workflow"
4. Select branch
5. Click "Run workflow"

**Note:** Manual dispatch will build binaries but won't create a release (only available as workflow artifacts).

## Troubleshooting

### Release Not Created

**Issue:** Workflow runs but no release appears

**Solutions:**
- Verify tag starts with `v` (e.g., `v1.0.0` not `1.0.0`)
- Check workflow permissions in repository settings
- Review workflow logs for errors in `create-release` job

### Missing Artifacts

**Issue:** Some binaries missing from release

**Solutions:**
- Check all build jobs completed successfully
- Review `Display artifact structure` step output
- Verify artifact names match expected pattern

### Checksum Verification Fails

**Issue:** `sha256sum -c SHA256SUMS` reports mismatch

**Solutions:**
- Re-download the file (may be corrupted)
- Verify you're checking the correct file
- Ensure no modifications were made after download

## Security Considerations

### GITHUB_TOKEN

The workflow uses the automatic `GITHUB_TOKEN` provided by GitHub Actions:
- Scoped to the repository
- Automatically expires after job completion
- Has write permissions for releases (via `permissions: contents: write`)

### Checksum Verification

Users should always verify checksums:
1. Prevents tampered downloads
2. Detects corrupted files
3. Ensures download integrity

### Release Artifacts

All artifacts are built by GitHub Actions runners:
- No local development environment contamination
- Reproducible builds
- Auditable via workflow logs

## Best Practices

### Version Tagging

Use semantic versioning:
- `v1.0.0` - Major release
- `v1.1.0` - Minor release (new features)
- `v1.1.1` - Patch release (bug fixes)
- `v2.0.0-beta.1` - Pre-release

### Release Frequency

- Tag releases when features are complete
- Don't create releases for every commit
- Use draft releases for testing if needed

### Release Notes

The auto-generated notes include:
- All commits since last tag
- Pull request references
- Contributor credits

For major releases, consider custom release notes highlighting key features.

## Testing the Release Process

### Test with Manual Dispatch

1. Trigger workflow manually
2. Verify all builds complete
3. Check artifacts are created
4. Review artifact contents

### Test with Tag

1. Create a test tag (e.g., `v0.0.1-test`)
2. Push to trigger release
3. Verify release is created
4. Download and test binaries
5. Delete test release and tag if successful

## Next Steps

After configuring releases:
1. ✅ Release configuration complete
2. ⏳ Test complete workflow (Task 6)
3. 🎯 Create first release tag
4. 📦 Publish and distribute binaries
