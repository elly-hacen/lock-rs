use std::path::Path;

pub fn is_hidden(path: &Path) -> bool {
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) if name.starts_with('.') => true,
        _ => false,
    }
}

pub fn is_image(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    matches!(
        ext.as_deref(),
        Some("jpg")
            | Some("jpeg")
            | Some("png")
            | Some("gif")
            | Some("bmp")
            | Some("tiff")
            | Some("webp")
            | Some("avif")
    )
}
