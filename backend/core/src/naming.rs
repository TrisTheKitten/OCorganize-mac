use std::path::Path;

use crate::group::Strategy;

pub const ORGANIZED_CATEGORY_MARKER: &str = ".ocorganize-category";

pub fn is_organized_category_folder_name(name: &str) -> bool {
    name == "no_extension" || name.ends_with("_FILES")
}

pub fn is_organized_category_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_organized_category_folder_name)
        || path.join(ORGANIZED_CATEGORY_MARKER).is_file()
}

pub fn ensure_category_output_dir(target_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target_dir)?;
    let marker = target_dir.join(ORGANIZED_CATEGORY_MARKER);
    if !marker.exists() {
        std::fs::write(marker, "")?;
    }
    Ok(())
}

pub fn get_target_folder_name(category: &str, strategy: Strategy) -> String {
    if strategy == Strategy::Type && category == "no_extension" {
        return "no_extension".to_string();
    }
    let normalized = category
        .replace(' ', "_")
        .replace('(', "")
        .replace(')', "")
        .to_uppercase();
    format!("{normalized}_FILES")
}
