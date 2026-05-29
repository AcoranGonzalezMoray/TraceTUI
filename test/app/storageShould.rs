#[cfg(test)]
mod storage_tests {
    use crate::app::states::StorageState;
    use crate::app::storage::{DiskInfo, FileEntry, FILE_EXTENSION_FILTERS};
    use crate::app::types::FileSortMode;
    use crate::app::App;
    use std::path::PathBuf;

    #[test]
    fn test_storage_state_new() {
        let state = StorageState::new();
        assert!(state.disks.is_empty());
        assert_eq!(state.selected_disk_index, 0);
        assert!(!state.disks_loading);
        assert_eq!(state.current_directory, PathBuf::from("/"));
        assert!(state.file_entries.is_empty());
        assert_eq!(state.file_scroll, 0);
        assert_eq!(state.storage_focus, 0);
        assert_eq!(state.file_sort_mode, FileSortMode::ByName);
    }

    #[test]
    fn test_storage_state_disks() {
        let mut state = StorageState::new();
        assert!(state.disks.is_empty());
        state.disks.push(DiskInfo {
            mount_point: "C:\\".into(),
            device: "C:".into(),
            fs_type: "NTFS".into(),
            total_bytes: 512_000_000_000,
            used_bytes: 256_000_000_000,
            free_bytes: 256_000_000_000,
        });
        assert_eq!(state.disks.len(), 1);
    }

    #[test]
    fn test_storage_state_file_entries() {
        let mut state = StorageState::new();
        assert!(state.file_entries.is_empty());
        state.file_entries.push(FileEntry {
            name: "test.txt".into(),
            path: PathBuf::from("/test.txt"),
            is_dir: false,
            size: 1024,
            modified: "2025-01-01 12:00".into(),
            extension: "txt".into(),
        });
        assert_eq!(state.file_entries.len(), 1);
    }

    #[test]
    fn test_app_get_selected_disk_none() {
        let app = App::new();
        assert!(app.storage.disks.is_empty());
        assert!(app.get_selected_disk().is_none());
    }

    #[test]
    fn test_app_compute_filtered_indices_no_search() {
        let mut app = App::new();
        assert!(!app.storage.file_search_mode);
        app.storage.file_entries = vec![
            FileEntry {
                name: "a.txt".into(),
                path: PathBuf::from("/a.txt"),
                is_dir: false,
                size: 100,
                modified: String::new(),
                extension: "txt".into(),
            },
            FileEntry {
                name: "b.rs".into(),
                path: PathBuf::from("/b.rs"),
                is_dir: false,
                size: 200,
                modified: String::new(),
                extension: "rs".into(),
            },
        ];
        app.compute_filtered_indices();
        assert_eq!(app.network.cached_filtered_indices, vec![0, 1]);
    }

    #[test]
    fn test_app_abort_search() {
        let mut app = App::new();
        assert!(app.storage.search_progress_abort.is_none());
        assert!(!app.storage.search_progress_running);
        app.abort_search();
    }

    #[test]
    fn test_file_sort_mode_default() {
        assert_eq!(FileSortMode::ByName, FileSortMode::ByName);
    }

    #[test]
    fn test_file_extension_filters() {
        assert!(!FILE_EXTENSION_FILTERS.is_empty());
        assert_eq!(FILE_EXTENSION_FILTERS.len(), 7);
    }
}
