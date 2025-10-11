#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    match path.file_name() {
        Some(name) => {
            #[cfg(unix)]
            {
                name.as_bytes().first().map_or(false, |b| *b == b'.')
            }
            #[cfg(not(unix))]
            {
                name.to_string_lossy().starts_with(".")
            }
        }
        None => false,
    }
}

pub fn is_image(path: &Path) -> bool {
    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(e) => e,
        None => return false,
    };

    const IMAGE_EXTENSIONS: &[&str] = &[
        "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "avif",
        "heic", "heif", "heics", "heifs",  // iPhone / HEIF
        "dng", "cr2", "cr3", "nef", "arw", "raf", "rw2", "orf", "sr2", "pef", "raw"  // RAW formats
    ];

    IMAGE_EXTENSIONS.iter().any(|&ext_name| ext.eq_ignore_ascii_case(ext_name))
}
