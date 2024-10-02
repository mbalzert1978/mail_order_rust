use crate::constants::DATE_LENGTH;
use crate::constants::SEPARATOR;
use crate::constants::VALID_DAYS;
use crate::constants::VALID_MONTHS;
use crate::constants::VALID_YEARS;
use crate::prelude::*;

pub fn handle(files: std::fs::ReadDir, target: &str) -> Result<()> {
    files
        .filter_map(core::result::Result::ok)
        .map(|file| {
            extract_dst_path(&file.path(), std::path::Path::new(target))
                .map(|(parent, dst)| {
                    std::fs::create_dir_all(parent).and_then(|_| std::fs::copy(file.path(), &dst))
                })
                .map(|_| std::fs::remove_file(file.path()))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn extract_dst_path(
    path: &std::path::Path,
    target: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let suffix = extract_suffix(path)?;
    Ok((
        path.parent().ok_or_else(ProcessError::parts)?.to_path_buf(),
        split_into_parent_date(path).and_then(|(about, date)| {
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
fn split_into_parent_date(path: &std::path::Path) -> Result<(&str, &str)> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(ProcessError::parts)?
        .split_once(SEPARATOR)
        .ok_or_else(ProcessError::separator)
}

fn validate(date: &str) -> Result<&str> {
    match is_valid_fmt(date) {
        true => Ok(date),
        false => Err(ProcessError::date()),
    }
}

fn is_valid_fmt(date: &str) -> bool {
    date.len() == DATE_LENGTH && date.chars().all(|c| c.is_ascii_digit())
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

fn is_valid_range(input: &str, max: u16) -> bool {
    input.parse::<u16>().map(|n| n <= max).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn test_is_valid_range() {
        // Valid ranges
        assert!(is_valid_range("10", 15));
        assert!(is_valid_range("10", 10));

        // Invalid ranges
        assert!(!is_valid_range("20", 15));
        assert!(!is_valid_range("20", 10));
        assert!(!is_valid_range("15", 10));
        assert!(!is_valid_range("32", 10));
        assert!(!is_valid_range("10", 0));

        // Empty input
        assert!(!is_valid_range("", 10));
        assert!(!is_valid_range("10", 0));

        // Invalid characters
        assert!(!is_valid_range("abc", 10));
        assert!(!is_valid_range("10a", 10));
        assert!(!is_valid_range("a10", 10));
    }

    #[test]
    fn test_parse_date() {
        // Valid dates
        let result = parse_date("01102024");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ("01", "10", "2024"));

        // Invalid dates
        let result = parse_date("0110202a");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Date Error: No valid date format."
        );

        let result = parse_date("011020241234");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_fmt() {
        // Valid date formats
        assert!(is_valid_fmt("01102024"));
        assert!(is_valid_fmt("10102024"));

        // Invalid date formats
        assert!(!is_valid_fmt("0110202a"));
        assert!(!is_valid_fmt("101020241234"));
        assert!(!is_valid_fmt(""));
    }

    #[test]
    fn test_validate() {
        // Valid date formats
        assert!(validate("01102024").is_ok());
        assert!(validate("10102024").is_ok());

        // Invalid date formats
        assert!(validate("0110202a").is_err());
        assert!(validate("101020241234").is_err());
        assert!(validate("").is_err());
    }

    #[test]
    fn test_split_into_parent_date() {
        // Valid split has extension and separator
        let result = split_into_parent_date(Path::new("example-about_01102024.txt"));
        assert!(result.is_ok());

        assert_eq!(result.unwrap(), ("example-about", "01102024"));

        // Invalid splits
        let result = split_into_parent_date(Path::new("example-about.txt"));
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Separator Error: File name does not contain expected separator: _."
        );

        let result = split_into_parent_date(Path::new("example"));
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Separator Error: File name does not contain expected separator: _."
        );
    }

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
