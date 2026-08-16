use std::path::Path;

const MAX_NAME_LEN: usize = 240;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "json", "csv", "bin", "png", "jpg", "jpeg", "gif", "webp", "pdf", "zip", "gz", "tar",
    "rs", "ts", "tsx", "js", "md", "toml", "yaml", "yml", "xml", "html", "css", "wasm", "so",
    "dll", "exe", "dat", "log", "parquet", "arrow", "npy", "npz",
];

pub fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err("invalid file name".into());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("file name must not contain path separators".into());
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    match ext {
        Some(ext) if ALLOWED_EXTENSIONS.contains(&ext.as_str()) => Ok(()),
        Some(_) => Err("file extension is not in the analysis allowlist".into()),
        None => Ok(()),
    }
}

pub fn validate_size(size: u64, max: u64) -> Result<(), String> {
    if size > max {
        Err(format!("file exceeds maximum size of {max} bytes"))
    } else {
        Ok(())
    }
}

pub fn sniff_mime(name: &str, header: Option<&str>) -> String {
    if let Some(declared) = header {
        if !declared.is_empty() && declared != "application/octet-stream" {
            return declared.to_string();
        }
    }
    match Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "txt" | "md" | "log" => "text/plain",
        "json" => "application/json",
        "csv" => "text/csv",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string()
}
