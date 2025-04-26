# File Manager

A modern, feature-rich file manager application built with Rust and the Iced GUI toolkit. This application provides a clean, intuitive interface for navigating your filesystem with support for file operations, previews, and customizable views.

![File Manager](https://example.com/screenshot.png) <!-- Consider adding an actual screenshot of your application here -->

## Features

- **Modern UI**: Clean, responsive interface with customizable views
- **File Operations**: Copy, cut, paste, delete, and rename files and directories
- **Navigation**: Breadcrumb path navigation, forward/back history, and quick access shortcuts
- **Preview Support**: Preview images and text files directly in the application
- **Sorting & Grouping**: Sort files by name, size, date, or type with group by functionality
  - Custom sort icons for ascending/descending order
- **File Details**: View detailed information about selected files
- **Customization**: Show/hide hidden files, adjust view preferences
- **Custom Font Support**: Using InterKhmerLooped font for better readability

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

- **[Iced](https://github.com/iced-rs/iced)** (0.12): GUI framework with a focus on simplicity and type-safety
- **[Iced_aw](https://github.com/iced-rs/iced_aw)** (0.9.3): Additional widgets for Iced
- **[Tokio](https://tokio.rs/)** (1.x): Asynchronous runtime for non-blocking operations
- **[Chrono](https://github.com/chronotope/chrono)** (0.4): Date and time library
- **[mime_guess](https://github.com/abonander/mime_guess)** (2.0): MIME type detection from file extensions
- **[xdg](https://github.com/whitequark/rust-xdg)** (2.5): XDG Base Directory specification support
- **[dirs](https://github.com/dirs-dev/dirs-rs)** (5.0): Cross-platform directories for user and application data
- **[once_cell](https://github.com/matklad/once_cell)** (1.19): Single assignment cells for better static initialization
- **[fs_extra](https://github.com/webdesus/fs_extra)** (1.3.0): Extended filesystem operations

## Icons and Resources

The application includes a set of custom icons for:

- File and folder representation
- Navigation controls
- Sorting indicators
- Special locations (home, documents, downloads, etc.)

These resources help provide a cohesive visual experience throughout the application.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin feature/my-new-feature`
5. Submit a pull request

## License

This project is licensed under the [MIT License](LICENSE) - see the LICENSE file for details.
