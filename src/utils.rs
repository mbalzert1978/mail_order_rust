use crate::constants::{SEPARATOR, VALID_DAYS, VALID_MONTHS, VALID_YEARS};
use crate::prelude::*;

pub fn handle(files: std::fs::ReadDir, target: &str) -> Result<()> {
    files
        .filter_map(core::result::Result::ok)
        .map(|file| {
            process(&file.path(), std::path::Path::new(target))
                .map(|_| std::fs::remove_file(file.path()))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn process(file_path: &std::path::Path, target: &std::path::Path) -> Result<()> {
    let _ = extract_dst_path(file_path, target).map(|(parent, dst)| {
        std::fs::create_dir_all(parent).and_then(|_| std::fs::copy(file_path, &dst))
    })?;
    Ok(())
}

fn extract_dst_path(
    path: &std::path::Path,
    target: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let suffix = extract_suffix(path)?;
    Ok((
        path.parent().ok_or_else(ProcessError::parts)?.to_path_buf(),
        extract_parent_date(path).and_then(|(about, date)| {
            validate(date).and_then(|valid_date| {
                parse_date(valid_date).map(|(day, month, year)| {
                    target
                        .join(year)
                        .join(month)
                        .join(day)
                        .join(about)
                        .with_extension(suffix)
                })
            })
        })?,
    ))
}
fn extract_suffix(path: &std::path::Path) -> Result<&str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(ProcessError::extension)
}
fn extract_parent_date(path: &std::path::Path) -> Result<(&str, &str)> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(ProcessError::parts)?
        .split_once(SEPARATOR)
        .ok_or_else(ProcessError::parts)
}

fn validate(date: &str) -> Result<&str> {
    match is_valid_fmt(date) {
        true => Ok(date),
        false => Err(ProcessError::date()),
    }
}

fn is_valid_fmt(date: &str) -> bool {
    date.len() == 8 && date.chars().all(|c| c.is_ascii_digit())
}

fn parse_date(date: &str) -> Result<(&str, &str, &str)> {
    let day = &date[0..2];
    let month = &date[2..4];
    let year = &date[4..];

    match is_valid_range(day, VALID_DAYS)
        && is_valid_range(month, VALID_MONTHS)
        && is_valid_range(year, VALID_YEARS)
    {
        true => Ok((day, month, year)),
        false => Err(ProcessError::date()),
    }
}

fn is_valid_range(day: &str, max: u16) -> bool {
    day.parse::<u16>().map(|n| n <= max).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn test_extract_dst_path_valid() {
        let target = Path::new("target");
        let parent_path = PathBuf::from("parent");

        let (parent, result) =
            extract_dst_path(&parent_path.join("example-about_01102024.txt"), target).unwrap();

        assert_eq!(
            result,
            target
                .join("2024")
                .join("10")
                .join("01")
                .join("example-about")
                .with_extension("txt")
        );
        assert_eq!(parent, parent_path);
    }

    #[test]
    fn test_extract_dst_path_invalid_date_format() {
        let result = extract_dst_path(&PathBuf::from("invalid_file_name.txt"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Date Error: No valid date format."
        );
    }

    #[test]
    fn test_extract_dst_path_formatting_error() {
        let result = extract_dst_path(&PathBuf::from("example-about.txt"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Parts Error: File name does not contain expected parts."
        );
    }

    #[test]
    fn test_extract_dst_path_stem_error() {
        let result = extract_dst_path(&PathBuf::from("example-about"), Path::new("target"));

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Extension Error: No valid file extension."
        );
    }
}
