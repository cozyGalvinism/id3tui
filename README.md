# ID3 Tagger TUI

A simple and intuitive terminal user interface (TUI) application for editing ID3v2 tags and metadata for audio files.

## Features

- **Easy-to-use TUI interface** with keyboard navigation
- **Multiple format support**:
  - MP3 (ID3v2 tags)
  - FLAC (Vorbis comments)
  - M4A/MP4/AAC (iTunes-style metadata)
- **File browser** for navigating directories and selecting audio files
- **In-place metadata editing** with real-time preview
- **Keyboard shortcuts** for efficient workflow
- **Help screen** with all available commands

## Installation

Make sure you have Rust installed, then build the project:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/id3tui`.

## Usage

Run the application from your terminal:

```bash
cargo run
```

Or use the compiled binary:

```bash
./target/release/id3tui
```

You can optionally specify a path to start browsing from:

```bash
# Start from a specific directory
cargo run -- /path/to/music

# Or with the binary
./target/release/id3tui ~/Music

# Relative paths work too
./target/release/id3tui ../audio-files
```

### Keyboard Shortcuts

#### Navigation
- `↑` or `k` - Move up
- `↓` or `j` - Move down
- `Enter` - Select file/directory
- `Backspace` - Go to parent directory
- `Tab` - Switch between file browser and metadata view

#### Editing
- `e` - Enter edit mode (when metadata view is focused)
- `↑`/`↓` - Move between fields (in edit mode)
- `←`/`→` - Move cursor within field (in edit mode)
- `Enter` - Save changes (in edit mode)
- `Esc` - Cancel editing without saving (in edit mode)

#### General
- `h` - Toggle help screen
- `q` - Quit application

## Supported Metadata Fields

- Title
- Artist
- Album
- Year
- Track Number
- Genre
- Comment

## How It Works

1. Use the file browser on the left to navigate to your audio files
2. Select an audio file (MP3, FLAC, or M4A) by pressing `Enter`
3. The metadata view on the right will display the current tags
4. Press `Tab` to focus the metadata view
5. Press `e` to enter edit mode
6. Use arrow keys to navigate between fields and edit the values
7. Press `Enter` to save your changes
8. Changes are written back to the file immediately

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `id3` - MP3 ID3 tag support
- `metaflac` - FLAC metadata support
- `mp4ameta` - M4A/MP4/AAC metadata support
- `anyhow` - Error handling
- `walkdir` - Directory traversal

## License

This project is open source and available under the MIT License.
