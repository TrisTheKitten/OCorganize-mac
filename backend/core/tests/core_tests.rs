use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use chrono::{TimeZone, Utc};
use ocorganize_core::{
    atomic_move_with_unique_name, atomic_move_with_unique_name_locked, get_target_folder_name,
    group_files, is_organized_category_folder_name, organize_files, restore_undo_log,
    sanitize_directory_path, write_undo_log, OrganizeOptions, PerformanceTier, RuntimeProfile,
    Strategy, PathValidationError, TargetDirLocks, UndoEntry, UndoLogWriter,
};

fn write_bytes(path: &Path, size: usize) {
    let data = vec![b'x'; size];
    fs::write(path, data).unwrap();
}

#[test]
fn sanitize_directory_path_valid() {
    let dir = tempfile::tempdir().unwrap();
    let result = sanitize_directory_path(dir.path().to_str().unwrap()).unwrap();
    assert_eq!(result, dir.path().canonicalize().unwrap());
}

#[test]
fn sanitize_directory_path_empty() {
    let result = sanitize_directory_path("");
    assert_eq!(result, Err(PathValidationError::Empty));
}

#[test]
fn sanitize_directory_path_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "data").unwrap();
    let result = sanitize_directory_path(file_path.to_str().unwrap());
    assert_eq!(result, Err(PathValidationError::NotADirectory));
}

#[cfg(unix)]
#[test]
fn sanitize_directory_path_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    fs::create_dir(&target).unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let result = sanitize_directory_path(link.to_str().unwrap());
    assert_eq!(result, Err(PathValidationError::SymlinkedDirectory));
}

#[test]
fn group_files_type_and_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("photo.jpg"), "photo").unwrap();
    fs::write(dir.path().join("clip.mp4"), "clip").unwrap();
    fs::write(dir.path().join("b"), "b").unwrap();
    let real = dir.path().join("real");
    fs::write(&real, "c").unwrap();

    #[cfg(unix)]
    {
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
    }

    let result = group_files(dir.path(), Strategy::Type);
    assert!(result
        .grouped
        .get("Document")
        .unwrap()
        .iter()
        .any(|path| path.file_name().unwrap() == "a.txt"));
    assert!(result
        .grouped
        .get("Image")
        .unwrap()
        .iter()
        .any(|path| path.file_name().unwrap() == "photo.jpg"));
    assert!(result
        .grouped
        .get("Video")
        .unwrap()
        .iter()
        .any(|path| path.file_name().unwrap() == "clip.mp4"));
    let no_ext: Vec<_> = result
        .grouped
        .get("no_extension")
        .unwrap()
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    let names: std::collections::HashSet<_> = no_ext.iter().map(String::as_str).collect();
    assert!(names.contains("b"));
    assert!(names.contains("real"));

    #[cfg(unix)]
    assert_eq!(result.symlink_skip_count, 1);
}

#[test]
fn group_files_size() {
    let dir = tempfile::tempdir().unwrap();
    let small = dir.path().join("small.bin");
    let medium = dir.path().join("medium.bin");
    write_bytes(&small, 100);
    write_bytes(&medium, 2 * 1024 * 1024);

    let result = group_files(dir.path(), Strategy::Size);
    let all_files: Vec<_> = result.grouped.values().flatten().collect();
    assert!(all_files.contains(&&small));
    assert!(all_files.contains(&&medium));
}

#[test]
fn group_files_date() {
    let dir = tempfile::tempdir().unwrap();
    let older = dir.path().join("older.txt");
    let newer = dir.path().join("newer.txt");
    fs::write(&older, "old").unwrap();
    fs::write(&newer, "new").unwrap();

    set_mtime(
        &older,
        Utc.with_ymd_and_hms(2023, 1, 15, 12, 0, 0).unwrap(),
    );
    set_mtime(
        &newer,
        Utc.with_ymd_and_hms(2024, 6, 5, 12, 0, 0).unwrap(),
    );

    let result = group_files(dir.path(), Strategy::Date);
    assert!(result.grouped.contains_key("2023-01"));
    assert!(result.grouped.contains_key("2024-06"));
}

#[test]
fn atomic_move_with_unique_name_collision() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let target_dir = dir.path().join("target");
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(&target_dir).unwrap();

    let source = source_dir.join("file.txt");
    fs::write(&source, "hello").unwrap();
    fs::write(target_dir.join("file.txt"), "existing").unwrap();

    let moved_path = atomic_move_with_unique_name(&source, &target_dir).unwrap();
    assert!(moved_path.exists());
    assert_eq!(fs::read_to_string(&moved_path).unwrap(), "hello");
    assert_eq!(moved_path.file_name().unwrap(), "file_1.txt");
    assert!(!source.exists());
    assert_eq!(
        fs::read_to_string(target_dir.join("file.txt")).unwrap(),
        "existing"
    );
}

#[test]
fn is_organized_category_folder_name_cases() {
    assert!(is_organized_category_folder_name("no_extension"));
    assert!(is_organized_category_folder_name("TXT_FILES"));
    assert!(is_organized_category_folder_name("2024-06_FILES"));
    assert!(!is_organized_category_folder_name("Photos"));
    assert!(!is_organized_category_folder_name("TXT"));
}

#[test]
fn group_files_skips_hidden_organized_and_nested_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("visible.txt"), "ok").unwrap();
    fs::write(dir.path().join(".hidden"), "secret").unwrap();
    fs::create_dir(dir.path().join("projects")).unwrap();
    fs::write(dir.path().join("projects/nested.js"), "nested").unwrap();

    let organized = dir.path().join("TXT_FILES");
    fs::create_dir(&organized).unwrap();
    fs::write(organized.join("already_sorted.txt"), "sorted").unwrap();

    let result = group_files(dir.path(), Strategy::Type);
    let all_files: Vec<_> = result.grouped.values().flatten().collect();

    assert_eq!(all_files.len(), 1);
    assert_eq!(all_files[0].file_name().unwrap(), "visible.txt");
}

#[test]
fn get_target_folder_name_cases() {
    assert_eq!(
        get_target_folder_name("no_extension", Strategy::Type),
        "no_extension"
    );
    assert_eq!(
        get_target_folder_name("Image", Strategy::Type),
        "IMAGE_FILES"
    );
    assert_eq!(
        get_target_folder_name("Small (< 1MB)", Strategy::Size),
        "SMALL_<_1MB_FILES"
    );
}

#[test]
fn loose_root_scan_skips_nested_fixture() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test/fixtures/nested");
    if !fixtures.exists() {
        return;
    }

    let result = group_files(&fixtures, Strategy::Type);
    let total: usize = result.grouped.values().map(|v| v.len()).sum();
    assert_eq!(total, 1);
}

fn set_mtime(path: &Path, time: chrono::DateTime<Utc>) {
    let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(time.timestamp() as u64);
    let _ = filetime::set_file_mtime(path, filetime::FileTime::from_system_time(system_time));
}

#[test]
fn runtime_profile_tiers() {
    let low = RuntimeProfile::from_specs(2, 2 * 1024 * 1024 * 1024);
    assert_eq!(low.tier, PerformanceTier::Low);
    assert_eq!(low.move_concurrency, 1);

    let standard = RuntimeProfile::from_specs(6, 6 * 1024 * 1024 * 1024);
    assert_eq!(standard.tier, PerformanceTier::Standard);

    let high = RuntimeProfile::from_specs(12, 16 * 1024 * 1024 * 1024);
    assert_eq!(high.tier, PerformanceTier::High);
    assert!(high.move_concurrency > 1);
}

#[test]
fn parallel_move_collision_safety() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let target_dir = dir.path().join("target");
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(&target_dir).unwrap();

    fs::write(source_dir.join("a.txt"), "a").unwrap();
    fs::write(source_dir.join("b.txt"), "b").unwrap();
    fs::write(target_dir.join("file.txt"), "existing").unwrap();

    let locks = TargetDirLocks::new();
    let target_lock = locks.lock_for(&target_dir);
    let moved_a = atomic_move_with_unique_name_locked(
        &source_dir.join("a.txt"),
        &target_dir,
        Some(target_lock.as_ref()),
    )
    .unwrap();
    let moved_b = atomic_move_with_unique_name_locked(
        &source_dir.join("b.txt"),
        &target_dir,
        Some(target_lock.as_ref()),
    )
    .unwrap();

    assert_ne!(moved_a, moved_b);
    assert!(moved_a.exists());
    assert!(moved_b.exists());
}

#[test]
fn undo_log_writer_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let writer = UndoLogWriter::new(dir.path(), 2).unwrap();
    writer
        .push(UndoEntry {
            from: dir.path().join("txt/file.txt").to_string_lossy().into_owned(),
            to: dir.path().join("txt/file_1.txt").to_string_lossy().into_owned(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        })
        .unwrap();
    writer
        .push(UndoEntry {
            from: dir.path().join("pdf/doc.pdf").to_string_lossy().into_owned(),
            to: dir.path().join("pdf/doc.pdf").to_string_lossy().into_owned(),
            timestamp: "2024-01-01T00:00:01Z".to_string(),
        })
        .unwrap();
    let undo_path = writer.finish(Vec::new()).unwrap();

    let entries = ocorganize_core::read_undo_log(&undo_path).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn organize_uses_precomputed_groups() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.pdf"), "b").unwrap();

    let grouped = group_files(dir.path(), Strategy::Type);
    let result = organize_files(OrganizeOptions {
        directory: dir.path(),
        strategy: Strategy::Type,
        create_undo: false,
        precomputed_groups: Some(grouped),
        profile: None,
        on_progress: None,
    })
    .unwrap();

    assert_eq!(result.organized_count, 2);
    assert_eq!(result.group_snapshot.total_files(), 2);
    assert!(dir.path().join("DOCUMENT_FILES").join("a.txt").exists());
    assert!(dir.path().join("DOCUMENT_FILES").join("b.pdf").exists());
}

#[test]
fn restore_undo_log_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let organized = dir.path().join("TXT_FILES");
    fs::create_dir_all(&organized).unwrap();
    let moved = organized.join("file.txt");
    fs::write(&moved, "data").unwrap();
    let original = dir.path().join("file.txt");

    let undo_path = write_undo_log(
        dir.path(),
        &[UndoEntry {
            from: moved.to_string_lossy().into_owned(),
            to: original.to_string_lossy().into_owned(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        }],
    )
    .unwrap();

    let result = restore_undo_log(&undo_path).unwrap();
    assert_eq!(result.restored_count, 1);
    assert_eq!(result.failed_count, 0);
    assert!(original.exists());
    assert!(!moved.exists());
    assert!(!undo_path.exists());
}

#[test]
fn categorize_extension_known_and_unknown() {
    use ocorganize_core::{categorize_extension, is_known_file_type};

    assert_eq!(categorize_extension(Some("jpg")), "Image");
    assert_eq!(categorize_extension(Some("MP4")), "Video");
    assert_eq!(categorize_extension(Some("pdf")), "Document");
    assert_eq!(categorize_extension(Some("xyz")), "xyz");
    assert_eq!(categorize_extension(None), "no_extension");
    assert!(is_known_file_type("Image"));
    assert!(!is_known_file_type("xyz"));
}

#[test]
fn strategy_try_from_str_accepts_legacy_extension_alias() {
    assert_eq!(Strategy::try_from_str("extension").unwrap(), Strategy::Type);
}

#[test]
fn strategy_try_from_str_rejects_unknown() {
    assert!(Strategy::try_from_str("invalid").is_err());
}

#[test]
fn write_undo_log_compat() {
    let dir = tempfile::tempdir().unwrap();
    let entries = vec![UndoEntry {
        from: "/tmp/from.txt".to_string(),
        to: "/tmp/to.txt".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    }];
    let path = write_undo_log(dir.path(), &entries).unwrap();
    let loaded = ocorganize_core::read_undo_log(&path).unwrap();
    assert_eq!(loaded, entries);
}
