# Brilliant File Manager

A modern, feature-rich file manager application built with Rust and the Iced GUI toolkit. This application provides a clean, intuitive interface for navigating your filesystem with support for file operations, previews, and customizable views.

![Brilliant File Manager](https://example.com/screenshot.png) <!-- Consider adding a screenshot here -->

## Features

- **Modern UI**: Clean, responsive interface with customizable views
- **File Operations**: Copy, cut, paste, delete, and rename files and directories
- **Navigation**: Breadcrumb path navigation, forward/back history, and quick access shortcuts
- **Preview Support**: Preview images and text files directly in the application
- **Sorting & Grouping**: Sort files by name, size, date, or type with group by functionality
- **File Details**: View detailed information about selected files
- **Customization**: Show/hide hidden files, adjust view preferences

## Building and Running

### Prerequisites

- Rust toolchain (rustc, cargo) - install via [rustup](https://rustup.rs/)
- GStreamer libraries for media support

#### Install GStreamer (required for media functionality)

**Ubuntu/Debian:**

```bash
sudo apt install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-good
```

**Fedora:**

```bash
sudo dnf install gstreamer1-devel gstreamer1-plugins-base-devel gstreamer1-plugins-good
```

**Arch Linux:**

```bash
sudo pacman -S gstreamer gst-plugins-base gst-plugins-good
```

### Build and Run

1. **Clone the repository:**

   ```bash
   git clone <repository-url>
   cd file-manager
   ```

2. **Build the project:**

   ```bash
   cargo build
   ```

3. **Run the application:**

   ```bash
   cargo run
   ```

   For better performance, use the release build:

   ```bash
   cargo build --release
   ./target/release/file-manager
   ```

## Project Architecture

The project is structured using a modular approach:

- **Core Application Logic**

  - `app.rs`: Main application state, message handling, and lifecycle management
  - `fs_utils.rs`: File system operations and utilities
  - `constants.rs`: Application constants and resource paths

- **User Interface**
  - `ui/view.rs`: Main layout orchestration
  - `ui/top_bar.rs`: Navigation controls, breadcrumb path, and sort options
  - `ui/sidebar.rs`: Quick access locations and bookmarks
  - `ui/file_grid.rs`: Main file display with grid layout and grouping
  - `ui/details_panel.rs`: Selected file information and preview
  - `ui/styles.rs`: Custom styling and theming

## Dependencies

- **[Iced](https://github.com/iced-rs/iced)**: GUI framework with a focus on simplicity and type-safety
- **[Tokio](https://tokio.rs/)**: Asynchronous runtime for non-blocking operations
- **[Chrono](https://github.com/chronotope/chrono)**: Date and time library
- **[mime_guess](https://github.com/abonander/mime_guess)**: MIME type detection from file extensions
- **[dirs](https://github.com/dirs-dev/dirs-rs)**: Cross-platform directories for user and application data
- **[fs_extra](https://github.com/webdesus/fs_extra)**: Extended filesystem operations
- **[GStreamer](https://gstreamer.freedesktop.org/)**: Media framework for audio/video support

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin feature/my-new-feature`
5. Submit a pull request

## License

This project is licensed under the [MIT License](LICENSE) - see the LICENSE file for details.
