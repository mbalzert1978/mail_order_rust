use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    time,
};

type Result<T> = core::result::Result<T, ProcessError>;

const SOURCE: &str = "M:/Documents/familie/Markus/scans";
const TARGET: &str = "M:/Documents/familie/Markus/Briefverkehr";
const DB_NAME: &str = "briefverkehr.db";

#[derive(Debug)]
enum ProcessError {
    Extension(String),
    Parts(String),
    Date(String),
    Path(String),
    Copy(String),
}

impl std::error::Error for ProcessError {}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Extension(msg) => write!(f, "Extension Error: {}", msg),
            ProcessError::Parts(msg) => write!(f, "Parts Error: {}", msg),
            ProcessError::Date(msg) => write!(f, "Date Error: {}", msg),
            ProcessError::Path(msg) => write!(f, "Path Error: {}", msg),
            ProcessError::Copy(msg) => write!(f, "Copy Error: {}", msg),
        }
    }
}

fn main() -> Result<()> {
    let source_path = Path::new(SOURCE);

    loop {
        match fs::read_dir(source_path) {
            Ok(files) => handle(files, TARGET)?,
            Err(_) => eprintln!("Failed to read directory: {}", source_path.display()),
        }
        std::thread::sleep(time::Duration::from_secs(60));
    }
}

fn handle(files: fs::ReadDir, target: &str) -> Result<()> {
    let mut files = files.peekable();
    if files.peek().is_none() {
        return Ok(());
    }
    let target_path = Path::new(target);

    for file in files {
        let msg = "This should be a file, if not this is a possible Bug.";
        let file = file.expect(msg);
        process(&file.path(), target_path)?;
        fs::remove_file(file.path())
            .map_err(|e| ProcessError::Path(format!("Could not remove file: {}", e)))?
    }
    Ok(())
}

fn process(file_path: &Path, target: &Path) -> Result<()> {
    let dst = extract_dst_path(file_path, target)?;
    fs::create_dir_all(dst.parent().expect("This should be a correct Path."))
        .map_err(|e| ProcessError::Path(format!("Could not create Path: {}", e)))
        .and_then(|_| {
            fs::copy(file_path, &dst)
                .map_err(|e| ProcessError::Copy(format!("Could not copy file: {}", e)))
        })?;
    Ok(())
}

fn extract_dst_path(file_path: &Path, target: &Path) -> Result<PathBuf> {
    let suffix = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| ProcessError::Extension("No valid file extension".to_string()))?;
    let parts = file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| ProcessError::Parts("Invalid file stem".to_string()))?;

    let mut parts = parts.split('_');
    let about = parts.next().ok_or_else(|| {
        ProcessError::Parts("File name does not contain expected parts".to_string())
    })?;
    let iso_date = parts.next().ok_or_else(|| {
        ProcessError::Parts("File name does not contain expected parts".to_string())
    })?;

    if iso_date.len() != 8 || iso_date.chars().any(|c| !c.is_ascii_digit()) {
        return Err(ProcessError::Date("Invalid date format".to_string()));
    }
    let day = &iso_date[0..2];
    let month = &iso_date[2..4];
    let year = &iso_date[4..];

    let dst_path = target
        .join(year)
        .join(month)
        .join(day)
        .join(about)
        .with_extension(suffix);

    Ok(dst_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_dst_path_valid() {
        let target = Path::new("target");

        let result = extract_dst_path(&PathBuf::from("example-about_01102024.txt"), target)
            .expect("Test failed.");

        assert_eq!(
            result,
            target
                .join("2024")
                .join("10")
                .join("example-about")
                .with_extension("txt")
        );
    }

    #[test]
    fn test_extract_dst_path_invalid_date_format() {
        let result = extract_dst_path(&PathBuf::from("invalid_file_name.txt"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Date Error: Invalid date format"
        );
    }

    #[test]
    fn test_extract_dst_path_formatting_error() {
        let result = extract_dst_path(&PathBuf::from("example-about.txt"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Parts Error: File name does not contain expected parts"
        );
    }

    #[test]
    fn test_extract_dst_path_stem_error() {
        let result = extract_dst_path(&PathBuf::from("example-about"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Extension Error: No valid file extension"
        );
    }
}
