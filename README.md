# File Organizer

A CLI tool to organize files by their extensions into categorized folders.

## Features

- Automatically categorizes files based on their extensions
- Supports multiple file categories (images, documents, videos, etc.)
- Dry-run mode to preview changes before applying them
- Verbose output for detailed logging
- Error handling for permissions and file-in-use scenarios
- Progress display and summary reporting

## Installation

### Building from source

```bash
cargo build --release
```

The binary will be available at `target/release/file-organizer`

## Usage

### Basic usage

Organize files in the current directory:

```bash
file-organizer
```

### Specify source directory

```bash
file-organizer --source /path/to/directory
```

### Specify output directory

```bash
file-organizer --source /path/to/source --output /path/to/output
```

### Dry-run mode

Preview changes without moving files:

```bash
file-organizer --dry-run
```

### Verbose mode

Show detailed information about the organization process:

```bash
file-organizer --verbose
```

### Combined options

```bash
file-organizer --source ~/Downloads --output ~/Organized --dry-run --verbose
```

## Command-line Options

- `-s, --source <PATH>` - Source directory containing files to organize (default: current directory)
- `-o, --output <PATH>` - Output directory for organized files (default: source directory)
- `-d, --dry-run` - Preview changes without actually moving files
- `-v, --verbose` - Show verbose output
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Development

### Running tests

```bash
cargo test
```

### Running integration tests

```bash
cargo test --test integration_tests
```

## Project Structure

- `src/main.rs` - CLI entry point and argument parsing
- `src/categories.rs` - File category definitions and mapping
- `src/scanner.rs` - Directory scanning logic
- `src/organizer.rs` - File organization and moving logic
- `tests/integration_tests.rs` - Integration tests

## License

MIT
