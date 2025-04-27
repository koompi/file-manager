use crate::app::{GroupCriteria, SortCriteria, SortOrder};
use crate::constants::{THUMBNAIL_CACHE_DIR, THUMBNAIL_SIZE};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use dashmap::DashMap;
use directories::ProjectDirs;
use freedesktop_desktop_entry::DesktopEntry;
use freedesktop_icons::lookup;
use fs_extra::dir::CopyOptions;
use iced::widget::image as iced_image; // Alias iced's image module
use image::{imageops, ImageError, ImageReader}; // Use ImageReader directly
use mime_guess::{self, mime};
use once_cell::sync::Lazy;
use ron::ser::PrettyConfig;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir_all, File};
use std::io::{self, BufReader, BufWriter, Read};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::time::SystemTime;
use xdg::BaseDirectories;

// Define the desired icon size (adjust as needed)
const DESIRED_ICON_SIZE: u16 = 48;

// Type alias for the cache data structure
type IconCacheData = HashMap<String, Option<PathBuf>>;

// Static cache for resolved icon paths, loaded lazily
static ICON_CACHE: Lazy<DashMap<String, Option<PathBuf>>> =
    Lazy::new(|| match load_icon_cache_from_file() {
        Ok(data) => {
            println!("Loaded {} icon cache entries from file.", data.len());
            data.into_iter().collect::<DashMap<_, _>>()
        }
        Err(e) => {
            eprintln!(
                "Failed to load icon cache: {}. Starting with empty cache.",
                e
            );
            DashMap::new()
        }
    });

// Helper function to get the cache file path
fn get_cache_file_path() -> Result<PathBuf, String> {
    let xdg_dirs = BaseDirectories::with_prefix("file-manager")
        .map_err(|e| format!("Failed to get XDG base directories: {}", e))?;
    xdg_dirs
        .place_cache_file("icon_cache.ron")
        .map_err(|e| format!("Failed to place cache file: {}", e))
}

// Function to load the cache from a file
fn load_icon_cache_from_file() -> Result<IconCacheData, String> {
    let cache_path = get_cache_file_path()?;
    if !cache_path.exists() {
        return Ok(HashMap::new()); // No cache file yet, return empty
    }

    let file = File::open(&cache_path)
        .map_err(|e| format!("Failed to open cache file {}: {}", cache_path.display(), e))?;
    let reader = BufReader::new(file);

    ron::de::from_reader(reader).map_err(|e| format!("Failed to deserialize icon cache: {}", e))
}

// Function to save the cache to a file
pub fn save_icon_cache() -> Result<(), String> {
    let cache_path = get_cache_file_path()?;

    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create cache directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    // Convert DashMap to HashMap for serialization
    let data_to_save: IconCacheData = ICON_CACHE
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().clone()))
        .collect();

    let file = File::create(&cache_path).map_err(|e| {
        format!(
            "Failed to create cache file {}: {}",
            cache_path.display(),
            e
        )
    })?;
    let writer = BufWriter::new(file);

    let pretty_config = PrettyConfig::new()
        .depth_limit(4)
        .indentor("  ".to_string());
    ron::ser::to_writer_pretty(writer, &data_to_save, pretty_config)
        .map_err(|e| format!("Failed to serialize icon cache: {}", e))?;

    println!(
        "Saved {} icon cache entries to {}",
        data_to_save.len(),
        cache_path.display()
    );
    Ok(())
}

#[derive(Debug, Clone)]
pub enum PreviewContent {
    Image(iced_image::Handle), // Use alias
    Text(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub original_desktop_path: Option<PathBuf>,
    pub icon_name: Option<String>,
    pub resolved_icon_path: Option<PathBuf>,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
    pub mime_group: Option<String>,
    pub thumbnail: Option<iced_image::Handle>, // Use alias
}

fn get_mime_group(mime_type: &mime_guess::Mime) -> Option<String> {
    match mime_type.type_() {
        mime::TEXT => Some("Text Files".to_string()),
        mime::IMAGE => Some("Images".to_string()),
        mime::VIDEO => Some("Videos".to_string()),
        mime::AUDIO => Some("Audio".to_string()),
        mime::APPLICATION => match mime_type.subtype() {
            mime::PDF => Some("Documents & Archives".to_string()),
            subtype if subtype.as_str() == "zip" => Some("Documents & Archives".to_string()),
            subtype if subtype.as_str().contains("compressed") => {
                Some("Documents & Archives".to_string())
            }
            _ => Some("Applications & Others".to_string()),
        },
        _ => None,
    }
}

fn get_thumbnail_cache_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "YourAppName", "FileManager") // Replace with your details
        .context("Failed to get project directories")?;
    let cache_dir = proj_dirs.cache_dir().join(THUMBNAIL_CACHE_DIR);
    create_dir_all(&cache_dir).with_context(|| {
        format!(
            "Failed to create thumbnail cache directory: {:?}",
            cache_dir
        )
    })?;
    Ok(cache_dir)
}

fn get_thumbnail_path(original_path: &Path) -> Result<PathBuf> {
    let cache_dir = get_thumbnail_cache_dir()?;
    let file_stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let path_hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        original_path.hash(&mut hasher);
        hasher.finish()
    };

    let thumb_filename = format!("{}_{}_{}.png", file_stem, path_hash, THUMBNAIL_SIZE);
    Ok(cache_dir.join(thumb_filename))
}

pub fn generate_thumbnail(original_path: &Path) -> Result<iced_image::Handle, ImageError> {
    // Use alias in return type
    let thumb_path = get_thumbnail_path(original_path)
        .map_err(|e| ImageError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

    // 1. Check cache
    if thumb_path.exists() {
        // Basic cache validation: Check if original file is newer than thumbnail
        let original_meta = fs::metadata(original_path).map_err(ImageError::IoError)?;
        let thumb_meta = fs::metadata(&thumb_path).map_err(ImageError::IoError)?;

        if let (Ok(orig_modified), Ok(thumb_modified)) =
            (original_meta.modified(), thumb_meta.modified())
        {
            if orig_modified <= thumb_modified {
                println!("Loading thumbnail from cache: {:?}", thumb_path);
                // Use from_memory to avoid holding file handle
                let bytes = fs::read(&thumb_path).map_err(ImageError::IoError)?;
                return Ok(iced_image::Handle::from_memory(bytes)); // Use alias
            }
            println!("Thumbnail cache outdated for: {:?}", original_path);
        } else {
            eprintln!(
                "Could not read modification times for cache check: {:?}",
                original_path
            );
        }
    }

    // 2. Generate thumbnail if not cached or invalid
    println!("Generating thumbnail for: {:?}", original_path);
    // Use ImageReader directly
    let img_reader = ImageReader::open(original_path)?.with_guessed_format()?;
    let img = img_reader.decode()?;

    // Use imageops from image crate
    let thumbnail = img.resize(
        THUMBNAIL_SIZE,
        THUMBNAIL_SIZE,
        imageops::FilterType::Lanczos3,
    );

    // 3. Save to cache
    thumbnail.save(&thumb_path).map_err(|e| {
        eprintln!("Failed to save thumbnail to {:?}: {}", thumb_path, e);
        e // Return the original ImageError
    })?;
    println!("Saved thumbnail to cache: {:?}", thumb_path);

    // Use from_memory to avoid holding file handle after saving
    let bytes = fs::read(&thumb_path).map_err(ImageError::IoError)?;
    Ok(iced_image::Handle::from_memory(bytes)) // Use alias
}

pub async fn open_file(path: PathBuf) -> Result<(), String> {
    let mut path_to_open = path.clone();

    #[cfg(target_os = "linux")]
    {
        if path.is_symlink() {
            if let Ok(target_path) = fs::read_link(&path) {
                if target_path
                    .extension()
                    .map_or(false, |ext| ext == "desktop")
                {
                    println!(
                        "Detected symlink to .desktop file: {} -> {}",
                        path.display(),
                        target_path.display()
                    );
                    path_to_open = target_path;
                }
            } else {
                eprintln!(
                    "Failed to read symlink {}: {}. Opening link directly.",
                    path.display(),
                    io::Error::last_os_error()
                );
            }
        }
    }

    let path_str = path_to_open
        .to_str()
        .ok_or_else(|| "Invalid path encoding".to_string())?;

    let status = {
        #[cfg(target_os = "linux")]
        {
            if path_to_open
                .extension()
                .map_or(false, |ext| ext == "desktop")
            {
                println!(
                    "Launching .desktop file using 'gio launch': {}",
                    path_to_open.display()
                );
                StdCommand::new("gio").args(&["launch", path_str]).status()
            } else {
                println!(
                    "Opening file/directory using 'xdg-open': {}",
                    path_to_open.display()
                );
                StdCommand::new("xdg-open").arg(path_str).status()
            }
        }
        #[cfg(target_os = "macos")]
        {
            println!(
                "Opening file/directory using 'open': {}",
                path_to_open.display()
            );
            StdCommand::new("open").arg(path_str).status()
        }
        #[cfg(target_os = "windows")]
        {
            println!(
                "Opening file/directory using 'start': {}",
                path_to_open.display()
            );
            StdCommand::new("cmd")
                .args(&["/C", "start", "", path_str])
                .status()
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            eprintln!("Unsupported OS for open_file. Attempting xdg-open.");
            StdCommand::new("xdg-open").arg(path_str).status()
        }
    };

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                Ok(())
            } else {
                Err(format!("Command failed with status: {}", exit_status))
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

pub async fn load_preview(path: PathBuf) -> Result<PreviewContent, String> {
    let path_clone = path.clone();
    tokio::task::spawn_blocking(move || {
        let mime_type = mime_guess::from_path(&path_clone).first_or_octet_stream();

        match mime_type.type_() {
            mime::IMAGE => Ok(PreviewContent::Image(iced_image::Handle::from_path(
                path_clone,
            ))), // Use alias
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
    .map_err(|e| format!("Task join error: {}", e))?
}

pub async fn read_dir(
    path: PathBuf,
    show_hidden: bool,
    sort_criteria: SortCriteria,
    sort_order: SortOrder,
    group_criteria: GroupCriteria,
) -> Result<Vec<DirEntry>, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;
    let app_dir = home_dir.join("Applications");
    let is_app_dir = path == app_dir;

    let read_dir_iter = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?;

    let mut entries_futures = Vec::new();

    for entry_result in read_dir_iter {
        if let Ok(entry) = entry_result {
            let entry_path = entry.path();
            let entry_path_clone = entry_path.clone();

            entries_futures.push(tokio::spawn(async move {
                let file_type = entry.file_type().ok();

                let file_name = entry_path_clone
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let is_hidden = file_name.starts_with('.');
                if !show_hidden && is_hidden {
                    return None;
                }

                let mut display_name = file_name.clone();
                let mut original_desktop_path: Option<PathBuf> = None;
                let mut icon_name: Option<String> = None;
                let mut resolved_icon_path: Option<PathBuf> = None;

                if is_app_dir && file_type.map_or(false, |ft| ft.is_symlink()) {
                    if let Ok(target_path) = fs::read_link(&entry_path_clone) {
                        if target_path
                            .extension()
                            .map_or(false, |ext| ext == "desktop")
                        {
                            match DesktopEntry::from_path(&target_path, None::<&[&str]>) {
                                Ok(desktop_entry) => {
                                    display_name = desktop_entry
                                        .name(&[] as &[&str])
                                        .map(|cow| cow.into_owned())
                                        .unwrap_or(file_name.clone());
                                    original_desktop_path = Some(target_path);
                                    icon_name = desktop_entry.icon().map(|cow| cow.to_owned());

                                    if let Some(name) = &icon_name {
                                        if !name.is_empty() {
                                            resolved_icon_path = ICON_CACHE
                                                .entry(name.clone())
                                                .or_insert_with(|| {
                                                    lookup(name).with_size(DESIRED_ICON_SIZE).find()
                                                })
                                                .value()
                                                .clone();
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to parse desktop file {}: {}",
                                        original_desktop_path
                                            .as_ref()
                                            .map(|p| p.display())
                                            .unwrap_or(entry_path_clone.display()),
                                        e
                                    );
                                }
                            }
                        }
                    }
                }

                let fs_metadata = fs::metadata(&entry_path_clone).ok();
                let is_dir = fs_metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = fs_metadata.as_ref().map(|m| m.len());
                let modified = fs_metadata.as_ref().and_then(|m| m.modified().ok());

                let mime_type = if is_dir {
                    None
                } else {
                    mime_guess::from_path(&entry_path_clone).first()
                };
                let mime_group = mime_type.as_ref().and_then(get_mime_group);

                let thumbnail = if !is_dir && mime_group.as_deref() == Some("Images") {
                    let path_for_thumb = entry_path_clone.clone();
                    tokio::task::spawn_blocking(move || generate_thumbnail(&path_for_thumb).ok())
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                Some(DirEntry {
                    path: entry_path_clone,
                    display_name,
                    original_desktop_path,
                    icon_name,
                    resolved_icon_path,
                    is_dir,
                    size,
                    modified,
                    mime_group,
                    thumbnail, // Already uses the updated DirEntry struct field type
                })
            }));
        }
    }

    let mut entries: Vec<DirEntry> = Vec::new();
    for future in entries_futures {
        if let Ok(Some(entry)) = future.await {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| {
        let group_ordering = match group_criteria {
            GroupCriteria::None => Ordering::Equal,
            GroupCriteria::Type => b.is_dir.cmp(&a.is_dir),
            GroupCriteria::MimeType => match (a.is_dir, b.is_dir) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.mime_group.cmp(&b.mime_group),
            },
        };

        if group_ordering != Ordering::Equal {
            return group_ordering;
        }

        let sort_ordering = match sort_criteria {
            SortCriteria::Name => a.display_name.cmp(&b.display_name),
            SortCriteria::Size => {
                let a_size = if a.is_dir { 0 } else { a.size.unwrap_or(0) };
                let b_size = if b.is_dir { 0 } else { b.size.unwrap_or(0) };
                if a.is_dir == b.is_dir && a_size == b_size {
                    a.display_name.cmp(&b.display_name)
                } else {
                    a_size.cmp(&b_size)
                }
            }
            SortCriteria::ModifiedDate => {
                let a_mod = a.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                let b_mod = b.modified.unwrap_or(SystemTime::UNIX_EPOCH);
                if a_mod == b_mod {
                    a.display_name.cmp(&b.display_name)
                } else {
                    a_mod.cmp(&b_mod)
                }
            }
            SortCriteria::Type => {
                if a.is_dir != b.is_dir {
                    b.is_dir.cmp(&a.is_dir)
                } else {
                    let a_ext = a.path.extension().unwrap_or_default();
                    let b_ext = b.path.extension().unwrap_or_default();
                    if a_ext == b_ext {
                        a.display_name.cmp(&b.display_name)
                    } else {
                        a_ext.cmp(b_ext)
                    }
                }
            }
        };

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
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Path is neither a file nor a directory",
        ))
    };

    result.map_err(|e| format!("Failed to delete '{}': {}", path.display(), e))
}

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
        overwrite: false,
        skip_exist: true,
        buffer_size: 64000,
        copy_inside: false,
        content_only: false,
        depth: 0,
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

    match fs::rename(&source, &destination_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Failed to move '{}' to '{}': {}. (Might be cross-device operation?)",
            source.display(),
            destination_path.display(),
            e
        )),
    }
}

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

pub async fn setup_applications_directory() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;
    let app_dir = home_dir.join("Applications");

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| {
            format!(
                "Failed to create applications directory {}: {}",
                app_dir.display(),
                e
            )
        })?;
        println!("Created directory: {}", app_dir.display());
    }

    let app_dir_clone = app_dir.clone();
    tokio::spawn(async move {
        println!("Starting background task to link .desktop files using desktop_entries...");

        let locales: Vec<String> = Vec::new();
        let entries = freedesktop_desktop_entry::desktop_entries(&locales);

        let mut linked_app_names = HashSet::new();

        for entry in entries {
            if entry.type_() == Some("Application") && !entry.no_display() && !entry.terminal() {
                if let Some(app_name_cow) = entry.name(&[] as &[&str]) {
                    let app_name = app_name_cow.into_owned().to_lowercase();

                    if linked_app_names.contains(&app_name) {
                        continue;
                    }

                    let source_path = entry.path;
                    if let Some(file_name) = source_path.file_name() {
                        let link_path = app_dir_clone.join(file_name);
                        if !link_path.exists() {
                            match symlink(&source_path, &link_path) {
                                Ok(_) => {
                                    linked_app_names.insert(app_name);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Failed to link {} -> {}: {}",
                                        source_path.display(),
                                        link_path.display(),
                                        e
                                    );
                                }
                            }
                        } else {
                            linked_app_names.insert(app_name);
                        }
                    }
                } else {
                    let source_path = entry.path;
                    eprintln!(
                        "Warning: Could not get Name= field from {}",
                        source_path.display()
                    );
                }
            }
        }

        println!("Finished linking .desktop files using desktop_entries.");
    });

    Ok(())
}

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
