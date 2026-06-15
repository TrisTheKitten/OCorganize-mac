use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

fn extension_type_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map = HashMap::new();
        register_type(&mut map, "Image", IMAGE_EXTENSIONS);
        register_type(&mut map, "Video", VIDEO_EXTENSIONS);
        register_type(&mut map, "Audio", AUDIO_EXTENSIONS);
        register_type(&mut map, "Document", DOCUMENT_EXTENSIONS);
        register_type(&mut map, "Spreadsheet", SPREADSHEET_EXTENSIONS);
        register_type(&mut map, "Presentation", PRESENTATION_EXTENSIONS);
        register_type(&mut map, "Archive", ARCHIVE_EXTENSIONS);
        register_type(&mut map, "Code", CODE_EXTENSIONS);
        register_type(&mut map, "Font", FONT_EXTENSIONS);
        register_type(&mut map, "Disk Image", DISK_IMAGE_EXTENSIONS);
        register_type(&mut map, "Application", APPLICATION_EXTENSIONS);
        map
    })
}

fn register_type(
    map: &mut HashMap<&'static str, &'static str>,
    file_type: &'static str,
    extensions: &[&'static str],
) {
    for extension in extensions {
        map.insert(extension, file_type);
    }
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "svg", "heic", "heif", "ico",
    "raw", "cr2", "cr3", "nef", "arw", "dng", "orf", "rw2", "psd", "ai", "eps", "avif", "jfif",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "webm", "m4v", "flv", "wmv", "mpg", "mpeg", "3gp", "ogv", "mts",
    "m2ts", "ts", "vob",
];

const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "wav", "flac", "aac", "m4a", "ogg", "wma", "aiff", "aif", "opus", "mid", "midi", "caf",
];

const DOCUMENT_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "txt", "rtf", "odt", "pages", "md", "tex", "epub", "mobi", "azw", "azw3",
    "xps", "wps",
];

const SPREADSHEET_EXTENSIONS: &[&str] = &["xls", "xlsx", "csv", "numbers", "ods", "tsv"];

const PRESENTATION_EXTENSIONS: &[&str] = &["ppt", "pptx", "key", "odp"];

const ARCHIVE_EXTENSIONS: &[&str] = &[
    "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "tbz2", "lz", "lzma", "zst",
];

const CODE_EXTENSIONS: &[&str] = &[
    "js", "ts", "jsx", "tsx", "py", "rs", "go", "java", "cpp", "c", "h", "hpp", "cs", "swift", "kt",
    "rb", "php", "html", "htm", "css", "scss", "sass", "less", "json", "xml", "yaml", "yml", "toml",
    "sh", "bash", "zsh", "sql", "vue", "svelte", "m", "mm", "pl", "r", "lua", "dart", "scala", "vb",
    "fs", "clj", "ex", "exs", "hs", "elm", "wasm",
];

const FONT_EXTENSIONS: &[&str] = &["ttf", "otf", "woff", "woff2", "eot"];

const DISK_IMAGE_EXTENSIONS: &[&str] = &["dmg", "iso", "img", "toast", "sparseimage"];

const APPLICATION_EXTENSIONS: &[&str] = &["app", "pkg", "exe", "msi", "deb", "rpm", "apk"];

pub fn categorize_path(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::trim)
        .filter(|ext| !ext.is_empty());

    categorize_extension(extension)
}

pub fn categorize_extension(extension: Option<&str>) -> String {
    match extension {
        None => "no_extension".to_string(),
        Some(raw) => {
            let normalized = raw.to_lowercase();
            if normalized.is_empty() {
                return "no_extension".to_string();
            }
            extension_type_map()
                .get(normalized.as_str())
                .map(|file_type| (*file_type).to_string())
                .unwrap_or(normalized)
        }
    }
}

pub fn is_known_file_type(category: &str) -> bool {
    matches!(
        category,
        "Image"
            | "Video"
            | "Audio"
            | "Document"
            | "Spreadsheet"
            | "Presentation"
            | "Archive"
            | "Code"
            | "Font"
            | "Disk Image"
            | "Application"
    )
}
