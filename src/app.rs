use crate::fs_utils::{
    copy_item, delete_item, move_item, open_file, read_dir, rename_item,
    setup_applications_directory, DirEntry, PreviewContent, generate_thumbnail,
};
use crate::ui::view::view;
use dirs;
use iced::executor;
use iced::{Application, Command, Element, Theme};
use iced::widget::image;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortCriteria {
    Name,
    Size,
    ModifiedDate,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupCriteria {
    None,
    Type,
    MimeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardAction {
    Copy,
    Cut,
}

#[derive(Debug)]
pub struct FileManager {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub error: Option<String>,
    pub selected_path: Option<PathBuf>,
    history: Vec<PathBuf>,
    history_index: usize,
    pub show_hidden_files: bool,
    pub sort_criteria: SortCriteria,
    pub sort_order: SortOrder,
    pub group_criteria: GroupCriteria,
    pub collapsed_groups: HashSet<String>,
    pub clipboard_item: Option<(PathBuf, ClipboardAction)>,
    pub renaming_path: Option<PathBuf>,
    pub rename_input_value: String,
    pub preview_content: Option<PreviewContent>,
    pub show_details_panel: bool,
    pub last_click_time: Option<Instant>,
    pub last_clicked_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(PathBuf),
    LoadEntries(Result<Vec<DirEntry>, String>),
    GoUp,
    GoBack,
    GoForward,
    ToggleHiddenFiles,
    ItemClicked(PathBuf),
    DeleteItem(PathBuf),
    ItemDeleted(Result<(), String>),
    CopyItem(PathBuf),
    CutItem(PathBuf),
    Paste,
    ItemPasted(Result<(), String>),
    StartRename(PathBuf),
    RenameInputChanged(String),
    ConfirmRename,
    CancelRename,
    ItemRenamed(Result<(), String>),
    SetSortCriteria(SortCriteria),
    ToggleSortOrder,
    SetGroupCriteria(GroupCriteria),
    ToggleGroupCollapse(String),
    FileOpenResult(Result<(), String>),
    LoadPreview(Result<PreviewContent, String>),
    SetupApplicationsResult(Result<(), String>),
    ToggleDetailsPanel,
    ThumbnailLoaded(PathBuf, Option<image::Handle>),
}

impl Application for FileManager {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let initial_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let initial_sort_criteria = SortCriteria::Name;
        let initial_sort_order = SortOrder::Ascending;
        let initial_group_criteria = GroupCriteria::None;
        let initial_state = FileManager {
            current_path: initial_path.clone(),
            entries: vec![],
            error: None,
            selected_path: None,
            history: vec![initial_path.clone()],
            history_index: 0,
            show_hidden_files: false,
            sort_criteria: initial_sort_criteria,
            sort_order: initial_sort_order,
            group_criteria: initial_group_criteria,
            collapsed_groups: HashSet::new(),
            clipboard_item: None,
            renaming_path: None,
            rename_input_value: String::new(),
            preview_content: None,
            show_details_panel: true,
            last_click_time: None,
            last_clicked_path: None,
        };

        let initial_commands = Command::batch([
            Command::perform(
                read_dir(
                    initial_path,
                    false,
                    initial_sort_criteria,
                    initial_sort_order,
                    initial_group_criteria,
                ),
                Message::LoadEntries,
            ),
            Command::perform(
                setup_applications_directory(),
                Message::SetupApplicationsResult,
            ),
        ]);

        (initial_state, initial_commands)
    }

    fn title(&self) -> String {
        format!("File Manager - {}", self.current_path.display())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match &message {
            Message::Navigate(_)
            | Message::LoadEntries(_)
            | Message::GoUp
            | Message::GoBack
            | Message::GoForward
            | Message::ToggleHiddenFiles
            | Message::DeleteItem(_)
            | Message::ItemDeleted(_)
            | Message::Paste
            | Message::ItemPasted(_)
            | Message::ConfirmRename
            | Message::CancelRename
            | Message::ItemRenamed(_)
            | Message::SetSortCriteria(_)
            | Message::ToggleSortOrder
            | Message::SetGroupCriteria(_) => {
                self.preview_content = None;
            }
            Message::ItemClicked(path) => if self.selected_path.as_ref() != Some(path) {},
            _ => {}
        }

        if self.renaming_path.is_some() {
            match message {
                Message::Navigate(_)
                | Message::GoUp
                | Message::GoBack
                | Message::GoForward
                | Message::ToggleHiddenFiles
                | Message::SetSortCriteria(_)
                | Message::ToggleSortOrder
                | Message::SetGroupCriteria(_)
                | Message::DeleteItem(_) => {
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                }
                _ => {}
            }
        }

        match message {
            Message::Navigate(path) => {
                if path.is_dir() {
                    let target_path = path.canonicalize().unwrap_or(path);
                    if target_path != self.current_path {
                        self.current_path = target_path.clone();
                        self.error = None;
                        self.selected_path = None;
                        self.preview_content = None;
                        self.renaming_path = None;
                        self.rename_input_value.clear();
                        self.update_history(target_path.clone());
                        Command::perform(
                            read_dir(
                                target_path,
                                self.show_hidden_files,
                                self.sort_criteria,
                                self.sort_order,
                                self.group_criteria,
                            ),
                            Message::LoadEntries,
                        )
                    } else {
                        Command::none()
                    }
                } else {
                    Command::perform(open_file(path), Message::FileOpenResult)
                }
            }
            Message::LoadEntries(result) => {
                match result {
                    Ok(entries) => {
                        self.entries = entries;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                        self.entries = vec![];
                    }
                }
                self.selected_path = None;
                self.preview_content = None;
                self.renaming_path = None;
                self.rename_input_value.clear();
                Command::none()
            }
            Message::GoUp => {
                if let Some(parent) = self.current_path.parent() {
                    let parent_path = parent.to_path_buf();
                    if parent_path != self.current_path {
                        self.current_path = parent_path.clone();
                        self.error = None;
                        self.selected_path = None;
                        self.preview_content = None;
                        self.renaming_path = None;
                        self.rename_input_value.clear();
                        self.update_history(parent_path.clone());
                        Command::perform(
                            read_dir(
                                parent_path,
                                self.show_hidden_files,
                                self.sort_criteria,
                                self.sort_order,
                                self.group_criteria,
                            ),
                            Message::LoadEntries,
                        )
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::GoBack => {
                if self.can_go_back() {
                    self.history_index -= 1;
                    let path = self.history[self.history_index].clone();
                    self.current_path = path.clone();
                    self.error = None;
                    self.selected_path = None;
                    self.preview_content = None;
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                    Command::perform(
                        read_dir(
                            path,
                            self.show_hidden_files,
                            self.sort_criteria,
                            self.sort_order,
                            self.group_criteria,
                        ),
                        Message::LoadEntries,
                    )
                } else {
                    Command::none()
                }
            }
            Message::GoForward => {
                if self.can_go_forward() {
                    self.history_index += 1;
                    let path = self.history[self.history_index].clone();
                    self.current_path = path.clone();
                    self.error = None;
                    self.selected_path = None;
                    self.preview_content = None;
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                    Command::perform(
                        read_dir(
                            path,
                            self.show_hidden_files,
                            self.sort_criteria,
                            self.sort_order,
                            self.group_criteria,
                        ),
                        Message::LoadEntries,
                    )
                } else {
                    Command::none()
                }
            }
            Message::ToggleHiddenFiles => {
                self.show_hidden_files = !self.show_hidden_files;
                self.preview_content = None;
                self.renaming_path = None;
                self.rename_input_value.clear();
                Command::perform(
                    read_dir(
                        self.current_path.clone(),
                        self.show_hidden_files,
                        self.sort_criteria,
                        self.sort_order,
                        self.group_criteria,
                    ),
                    Message::LoadEntries,
                )
            }
            Message::ItemClicked(path) => {
                let is_double_click = self.last_clicked_path.as_ref() == Some(&path) &&
                                      self.last_click_time.map_or(false, |t| t.elapsed() < Duration::from_millis(500));

                self.selected_path = Some(path.clone());
                self.last_click_time = Some(Instant::now());
                self.last_clicked_path = Some(path.clone());

                if is_double_click {
                    return Command::perform(async move { path }, Message::Navigate);
                }

                if let Some(entry) = self.entries.iter().find(|e| e.path == *self.selected_path.as_ref().unwrap()) {
                    if entry.mime_group.as_deref() == Some("Images") && entry.thumbnail.is_none() {
                         let p = entry.path.clone();
                         return Command::perform(load_thumbnail_async(p.clone()), move |handle| {
                             Message::ThumbnailLoaded(p, handle)
                         });
                    }
                }
                Command::none()
            }
            Message::DeleteItem(path) => {
                println!("Delete requested for: {}", path.display());
                self.preview_content = None;
                self.renaming_path = None;
                self.rename_input_value.clear();
                Command::perform(delete_item(path), Message::ItemDeleted)
            }
            Message::ItemDeleted(result) => {
                let command = match result {
                    Ok(_) => {
                        self.error = None;
                        Command::perform(
                            read_dir(
                                self.current_path.clone(),
                                self.show_hidden_files,
                                self.sort_criteria,
                                self.sort_order,
                                self.group_criteria,
                            ),
                            Message::LoadEntries,
                        )
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to delete item: {}", e));
                        Command::none()
                    }
                };
                self.selected_path = None;
                self.preview_content = None;
                command
            }
            Message::CopyItem(path) => {
                println!("Copy requested for: {}", path.display());
                self.clipboard_item = Some((path, ClipboardAction::Copy));
                self.error = None;
                Command::none()
            }
            Message::CutItem(path) => {
                println!("Cut requested for: {}", path.display());
                self.clipboard_item = Some((path, ClipboardAction::Cut));
                self.error = None;
                Command::none()
            }
            Message::Paste => {
                if let Some((source_path, action)) = self.clipboard_item.clone() {
                    let destination_dir = self.current_path.clone();
                    println!(
                        "Paste requested: {:?} {} to {}",
                        action,
                        source_path.display(),
                        destination_dir.display()
                    );

                    let command = match action {
                        ClipboardAction::Copy => Command::perform(
                            copy_item(source_path, destination_dir),
                            Message::ItemPasted,
                        ),
                        ClipboardAction::Cut => Command::perform(
                            move_item(source_path, destination_dir),
                            Message::ItemPasted,
                        ),
                    };
                    if action == ClipboardAction::Copy {}
                    command
                } else {
                    self.error = Some("Clipboard is empty.".to_string());
                    Command::none()
                }
            }
            Message::ItemPasted(result) => {
                let command = match result {
                    Ok(_) => {
                        self.error = None;
                        if let Some((_, ClipboardAction::Cut)) = self.clipboard_item {
                            self.clipboard_item = None;
                        }
                        Command::perform(
                            read_dir(
                                self.current_path.clone(),
                                self.show_hidden_files,
                                self.sort_criteria,
                                self.sort_order,
                                self.group_criteria,
                            ),
                            Message::LoadEntries,
                        )
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to paste item: {}", e));
                        Command::none()
                    }
                };
                self.selected_path = None;
                self.preview_content = None;
                command
            }
            Message::StartRename(path) => {
                println!("Start rename requested for: {}", path.display());
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    self.renaming_path = Some(path.clone());
                    self.rename_input_value = file_name.to_string();
                    self.error = None;
                } else {
                    self.error = Some("Cannot get file name to rename.".to_string());
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                }
                Command::none()
            }
            Message::RenameInputChanged(new_value) => {
                if self.renaming_path.is_some() {
                    self.rename_input_value = new_value;
                }
                Command::none()
            }
            Message::ConfirmRename => {
                if let Some(path_to_rename) = self.renaming_path.clone() {
                    if !self.rename_input_value.is_empty()
                        && self.rename_input_value
                            != path_to_rename
                                .file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or_default()
                    {
                        println!(
                            "Confirm rename: {} to {}",
                            path_to_rename.display(),
                            self.rename_input_value
                        );
                        let new_name = self.rename_input_value.clone();
                        self.renaming_path = None;
                        self.rename_input_value.clear();
                        Command::perform(
                            rename_item(path_to_rename, new_name),
                            Message::ItemRenamed,
                        )
                    } else {
                        self.renaming_path = None;
                        self.rename_input_value.clear();
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::CancelRename => {
                println!("Cancel rename");
                self.renaming_path = None;
                self.rename_input_value.clear();
                self.error = None;
                self.preview_content = None;
                Command::none()
            }
            Message::ItemRenamed(result) => {
                let command = match result {
                    Ok(_) => {
                        self.error = None;
                        Command::perform(
                            read_dir(
                                self.current_path.clone(),
                                self.show_hidden_files,
                                self.sort_criteria,
                                self.sort_order,
                                self.group_criteria,
                            ),
                            Message::LoadEntries,
                        )
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to rename item: {}", e));
                        Command::none()
                    }
                };
                self.selected_path = None;
                self.preview_content = None;
                command
            }
            Message::SetSortCriteria(criteria) => {
                if self.sort_criteria != criteria {
                    self.sort_criteria = criteria;
                    self.preview_content = None;
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                    Command::perform(
                        read_dir(
                            self.current_path.clone(),
                            self.show_hidden_files,
                            self.sort_criteria,
                            self.sort_order,
                            self.group_criteria,
                        ),
                        Message::LoadEntries,
                    )
                } else {
                    self.update(Message::ToggleSortOrder)
                }
            }
            Message::ToggleSortOrder => {
                self.sort_order = match self.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
                self.preview_content = None;
                self.renaming_path = None;
                self.rename_input_value.clear();
                Command::perform(
                    read_dir(
                        self.current_path.clone(),
                        self.show_hidden_files,
                        self.sort_criteria,
                        self.sort_order,
                        self.group_criteria,
                    ),
                    Message::LoadEntries,
                )
            }
            Message::SetGroupCriteria(criteria) => {
                if self.group_criteria != criteria {
                    self.group_criteria = criteria;
                    self.collapsed_groups.clear();
                    self.preview_content = None;
                    self.renaming_path = None;
                    self.rename_input_value.clear();
                    Command::perform(
                        read_dir(
                            self.current_path.clone(),
                            self.show_hidden_files,
                            self.sort_criteria,
                            self.sort_order,
                            self.group_criteria,
                        ),
                        Message::LoadEntries,
                    )
                } else {
                    Command::none()
                }
            }
            Message::ToggleGroupCollapse(group_id) => {
                if self.collapsed_groups.contains(&group_id) {
                    self.collapsed_groups.remove(&group_id);
                } else {
                    self.collapsed_groups.insert(group_id);
                }
                Command::none()
            }
            Message::FileOpenResult(result) => {
                if let Err(e) = result {
                    self.error = Some(format!("Failed to open file: {}", e));
                }
                Command::none()
            }
            Message::LoadPreview(result) => {
                match result {
                    Ok(PreviewContent::Image(handle)) => {
                        self.preview_content = Some(PreviewContent::Image(handle));
                    }
                    Ok(PreviewContent::Text(content)) => {
                        self.preview_content = Some(PreviewContent::Text(content));
                    }
                    Ok(PreviewContent::Error(e)) => {
                        self.preview_content = Some(PreviewContent::Error(e));
                    }
                    Err(e) => {
                        self.preview_content = Some(PreviewContent::Error(e));
                    }
                }
                Command::none()
            }
            Message::SetupApplicationsResult(result) => {
                if let Err(e) = result {
                    eprintln!("Failed to set up applications directory: {}", e);
                }
                Command::none()
            }
            Message::ToggleDetailsPanel => {
                self.show_details_panel = !self.show_details_panel;
                Command::none()
            }
            Message::ThumbnailLoaded(path, handle) => {
                if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
                    entry.thumbnail = handle;
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        view(self)
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }
}

impl FileManager {
    pub fn update_history(&mut self, new_path: PathBuf) {
        self.renaming_path = None;
        self.rename_input_value.clear();

        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }
        if self.history.last() != Some(&new_path) {
            self.history.push(new_path);
        }
        self.history_index = self.history.len() - 1;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index < self.history.len() - 1
    }

    pub fn is_renaming(&self, path: &PathBuf) -> bool {
        self.renaming_path.as_ref() == Some(path)
    }
}

async fn load_thumbnail_async(path: PathBuf) -> Option<image::Handle> {
    tokio::task::spawn_blocking(move || {
        match generate_thumbnail(&path) {
            Ok(handle) => Some(handle),
            Err(e) => {
                eprintln!("Failed to generate thumbnail for {:?}: {}", path, e);
                None
            }
        }
    }).await.ok().flatten()
}
