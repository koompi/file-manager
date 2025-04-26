use crate::app::{GroupCriteria, SortCriteria, SortOrder};
use chrono::{DateTime, Local};
use fs_extra::dir::CopyOptions;
use iced::widget::image;
use mime_guess::{self, mime};
use std::cmp::Ordering;
use std::fs::{self};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::time::SystemTime;

// Make PreviewContent public and derive Debug and Clone
#[derive(Debug, Clone)]
pub enum PreviewContent {
    Image(image::Handle),
    Text(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub mime_group: Option<String>,
}

fn get_mime_group(mime_type: &mime_guess::Mime) -> Option<String> {
    match mime_type.type_() {
        mime::TEXT => Some("Text Files".to_string()),
        mime::IMAGE => Some("Images".to_string()),
        mime::VIDEO => Some("Videos".to_string()),
        mime::AUDIO => Some("Audio".to_string()),
        mime::APPLICATION => match mime_type.subtype() {
            mime::PDF => Some("Documents & Archives".to_string()),
            // Check subtype for "zip" using as_str()
            subtype if subtype.as_str() == "zip" => Some("Documents & Archives".to_string()),
            subtype if subtype.as_str().contains("compressed") => {
                Some("Documents & Archives".to_string())
            }
            _ => Some("Applications & Others".to_string()),
        },
        _ => None,
    }
}

pub async fn open_file(path: PathBuf) -> Result<(), String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())?;

    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "windows")]
    let command = "cmd";

    #[cfg(not(target_os = "windows"))]
    let status = StdCommand::new(command).arg(path_str).status();

    #[cfg(target_os = "windows")]
    let status = StdCommand::new(command)
        .args(&["/C", "start", "", path_str])
        .status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                Err(format!(
                    "Command '{}' failed with status: {}",
                    command, exit_status
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute command '{}': {}", command, e)),
    }
}

// --- Preview Loading ---
pub async fn load_preview(path: PathBuf) -> Result<PreviewContent, String> {
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || {
        let mime_type = mime_guess::from_path(&path_clone).first_or_octet_stream();

        match mime_type.type_() {
            mime::IMAGE => {
                // Correctly wrap the handle creation
                Ok(PreviewContent::Image(image::Handle::from_path(path_clone)))
            }
            mime::TEXT => {
                let mut file = fs::File::open(&path_clone)
                    .map_err(|e| format!("Failed to open text file: {}", e))?;
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read text file: {}", e))?;
                Ok(PreviewContent::Text(content))
            }
            _ => Err(format!("Unsupported file type for preview: {}", mime_type)),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))? // Handle potential join error
}
// --- End Preview Loading ---

pub async fn read_dir(
    path: PathBuf,
    show_hidden: bool,
    sort_criteria: SortCriteria,
    sort_order: SortOrder,
    group_criteria: GroupCriteria, // Added group criteria
) -> Result<Vec<DirEntry>, String> {
    let read_dir = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?;

    let mut entries: Vec<DirEntry> = read_dir
        .filter_map(|entry_result| {
            entry_result.ok().and_then(|entry| {
                let path = entry.path();
                let metadata = entry.metadata().ok();
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let is_hidden = file_name.starts_with('.');
                if !show_hidden && is_hidden {
                    return None;
                }

                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len());
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                let mime_type = if is_dir {
                    None // No MIME for directories
                } else {
                    mime_guess::from_path(&path).first() // Get the first guess
                };
                let mime_group = mime_type.as_ref().and_then(get_mime_group); // Use helper

                Some(DirEntry {
                    path,
                    is_dir,
                    size,
                    modified,
                    mime_group, // Store the determined group
                })
            })
        })
        .collect();

    // --- Grouping Logic (before sorting within groups) ---
    entries.sort_by(|a, b| {
        let group_ordering = match group_criteria {
            GroupCriteria::None => Ordering::Equal, // No grouping applied here
            GroupCriteria::Type => b.is_dir.cmp(&a.is_dir), // Folders first
            GroupCriteria::MimeType => {
                // Prioritize folders, then group by mime_group, putting None (errors/unknown) last
                match (a.is_dir, b.is_dir) {
                    (true, false) => Ordering::Less, // a (folder) comes before b (file)
                    (false, true) => Ordering::Greater, // b (folder) comes before a (file)
                    _ => a.mime_group.cmp(&b.mime_group), // Both folders or both files, compare mime group
                }
            }
        };

        if group_ordering != Ordering::Equal {
            return group_ordering;
        }

        // --- Sorting Logic (applied after grouping or if no grouping) ---
        let sort_ordering = match sort_criteria {
            SortCriteria::Name => a.path.file_name().cmp(&b.path.file_name()),
            SortCriteria::Size => {
                // Treat folders as having size 0 for sorting purposes if comparing with files
                let a_size = if a.is_dir { 0 } else { a.size.unwrap_or(0) };
                let b_size = if b.is_dir { 0 } else { b.size.unwrap_or(0) };
                // If both are folders or both files with same size, sort by name
                if a.is_dir == b.is_dir && a_size == b_size {
                    a.path.file_name().cmp(&b.path.file_name())
                } else {
                    a_size.cmp(&b_size)
                }
            }
            SortCriteria::ModifiedDate => {
                let a_mod = a.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                let b_mod = b.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                // If dates are the same, sort by name
                if a_mod == b_mod {
                    a.path.file_name().cmp(&b.path.file_name())
                } else {
                    a_mod.cmp(&b_mod)
                }
            }
            SortCriteria::Type => {
                // Primarily sort by directory status, then by extension (or name if no extension)
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir) // Folders first
                } else {
                    let a_ext = a.path.extension().unwrap_or_default();
                    let b_ext = b.path.extension().unwrap_or_default();
                    if a_ext == b_ext {
                        a.path.file_name().cmp(&b.path.file_name()) // Same extension, sort by name
                    } else {
                        a_ext.cmp(b_ext) // Sort by extension
                    }
                }
            }
        };

        // Apply sort order (Ascending/Descending)
        match sort_order {
            SortOrder::Ascending => sort_ordering,
            SortOrder::Descending => sort_ordering.reverse(),
        }
    });

    Ok(entries)
}

pub async fn delete_item(path: PathBuf) -> Result<(), String> {
    println!("Attempting to delete: {}", path.display());
    let result = if path.is_dir() {
        fs::remove_dir_all(&path)
    } else if path.is_file() {
        fs::remove_file(&path)
    } else {
        // Handle symlinks or other file types if necessary
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Path is neither a file nor a directory",
        ))
    };

    result.map_err(|e| format!("Failed to delete '{}': {}", path.display(), e))
}

// --- Added copy_item function ---
pub async fn copy_item(source: PathBuf, destination_dir: PathBuf) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "Source path '{}' does not exist.",
            source.display()
        ));
    }
    if !destination_dir.is_dir() {
        return Err(format!(
            "Destination path '{}' is not a valid directory.",
            destination_dir.display()
        ));
    }

    let item_name = source
        .file_name()
        .ok_or_else(|| "Could not get file/folder name from source.".to_string())?;
    let destination_path = destination_dir.join(item_name);

    println!(
        "Copying {} to {}",
        source.display(),
        destination_path.display()
    );

    let options = CopyOptions {
        overwrite: false,   // Don't overwrite existing files/folders
        skip_exist: true,   // Skip copying if destination already exists
        buffer_size: 64000, // Default buffer size
        copy_inside: false, // Copy the item itself, not its content into the dest dir
        content_only: false,
        depth: 0, // No depth limit for recursion
    };

    let items_to_copy = vec![&source];

    match fs_extra::copy_items(&items_to_copy, &destination_dir, &options) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Failed to copy '{}' to '{}': {}",
            source.display(),
            destination_dir.display(),
            e
        )),
    }
}

// --- Added move_item function ---
pub async fn move_item(source: PathBuf, destination_dir: PathBuf) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "Source path '{}' does not exist.",
            source.display()
        ));
    }
    if !destination_dir.is_dir() {
        return Err(format!(
            "Destination path '{}' is not a valid directory.",
            destination_dir.display()
        ));
    }

    let item_name = source
        .file_name()
        .ok_or_else(|| "Could not get file/folder name from source.".to_string())?;
    let destination_path = destination_dir.join(item_name);

    if destination_path.exists() {
        return Err(format!(
            "Destination '{}' already exists. Cannot move.",
            destination_path.display()
        ));
    }

    println!(
        "Moving {} to {}",
        source.display(),
        destination_path.display()
    );

    // std::fs::rename works for both files and directories and is atomic on the same filesystem
    match fs::rename(&source, &destination_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // fs::rename might fail across different filesystems (devices).
            // A more robust solution would involve copying then deleting,
            // but for simplicity, we'll report the error for now.
            Err(format!(
                "Failed to move '{}' to '{}': {}. (Might be cross-device operation?)",
                source.display(),
                destination_path.display(),
                e
            ))
        }
    }
}

// --- Added rename_item function ---
pub async fn rename_item(path: PathBuf, new_name: String) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Path '{}' does not exist.", path.display()));
    }
    if new_name.is_empty()
        || new_name.contains('/')
        || new_name.contains('\\')
        || new_name == "."
        || new_name == ".."
    {
        return Err(format!("Invalid new name: '{}'", new_name));
    }

    if let Some(parent) = path.parent() {
        let new_path = parent.join(new_name);
        if new_path.exists() {
            return Err(format!(
                "Target path '{}' already exists.",
                new_path.display()
            ));
        }

        println!("Renaming {} to {}", path.display(), new_path.display());

        match fs::rename(&path, &new_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "Failed to rename '{}' to '{}': {}",
                path.display(),
                new_path.display(),
                e
            )),
        }
    } else {
        Err("Cannot rename root directory or item without parent.".to_string())
    }
}

// --- Helper Functions ---
pub fn format_size(size: Option<u64>) -> String {
    match size {
        Some(s) => {
            if s < 1024 {
                format!("{} B", s)
            } else if s < 1024 * 1024 {
                format!("{:.1} KB", s as f64 / 1024.0)
            } else if s < 1024 * 1024 * 1024 {
                format!("{:.1} MB", s as f64 / (1024.0 * 1024.0))
            } else {
                format!("{:.1} GB", s as f64 / (1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "-".to_string(),
    }
}

pub fn format_modified(modified: Option<SystemTime>) -> String {
    match modified {
        Some(time) => {
            let datetime: DateTime<Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "-".to_string(),
    }
}
