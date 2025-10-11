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

    let eq = |want: &str| ext.eq_ignore_ascii_case(want);

    eq("jpg")
        || eq("jpeg")
        || eq("png")
        || eq("gif")
        || eq("bmp")
        || eq("tif")
        || eq("tiff")
        || eq("webp")
        || eq("avif")
        // iPhone / HEIF
        || eq("heic")
        || eq("heif")
        || eq("heics")
        || eq("heifs")
        // RAW formats
        || eq("dng")
        || eq("cr2")
        || eq("cr3")
        || eq("nef")
        || eq("arw")
        || eq("raf")
        || eq("rw2")
        || eq("orf")
        || eq("sr2")
        || eq("pef")
        || eq("raw")
}
