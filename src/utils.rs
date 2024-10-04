use crate::constants::DATE_LENGTH;
use crate::constants::SEPARATOR;
use crate::constants::VALID_DAYS;
use crate::constants::VALID_MONTHS;
use crate::constants::VALID_YEARS;
use crate::prelude::*;

pub fn handle(files: std::fs::ReadDir, target: &str) -> Result<()> {
    files
        .filter_map(core::result::Result::ok)
        .try_for_each(|file| {
            let (parent, dst) = extract_dst_path(&file.path(), std::path::Path::new(target))?;
            std::fs::create_dir_all(&parent)?;
            std::fs::copy(file.path(), &dst)?;
            std::fs::remove_file(file.path())?;
            Ok(())
        })
}

fn extract_dst_path(
    path: &std::path::Path,
    target: &std::path::Path,
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    let suffix = extract_suffix(path)?;
    let parent = path.parent().ok_or_else(ProcessError::parts)?.to_path_buf();

    let (about, date) = split_into_parent_date(path)?;
    validate(date)?;
    let (day, month, year) = parse(date)?;

    let destination = target
        .join(year)
        .join(month)
        .join(day)
        .join(about)
        .with_extension(suffix);

    Ok((parent, destination))
}

fn extract_suffix(path: &std::path::Path) -> Result<&str> {
    path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(ProcessError::extension)
}

fn split_into_parent_date(path: &std::path::Path) -> Result<(&str, &str)> {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(ProcessError::parts)?;

    stem.split_once(SEPARATOR)
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

fn parse(date: &str) -> Result<(&str, &str, &str)> {
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
    use super::*;
    use std::path::{Path, PathBuf};

    fn assert_invalid_range(input: &str, max: u16) {
        assert!(!is_valid_range(input, max));
    }

    fn assert_valid_range(input: &str, max: u16) {
        assert!(is_valid_range(input, max));
    }

    #[test]
    fn test_is_valid_range_valid() {
        assert_valid_range("10", 15);
        assert_valid_range("10", 10);
    }

    #[test]
    fn test_is_valid_range_invalid() {
        assert_invalid_range("20", 15);
        assert_invalid_range("20", 10);
        assert_invalid_range("15", 10);
        assert_invalid_range("32", 10);
        assert_invalid_range("10", 0);
        assert_invalid_range("", 10);
        assert_invalid_range("abc", 10);
        assert_invalid_range("10a", 10);
        assert_invalid_range("a10", 10);
    }

    #[test]
    fn test_parse_date_valid() {
        let result = parse("01102024");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ("01", "10", "2024"));
    }

    #[test]
    fn test_parse_date_invalid() {
        let result1 = parse("0110202a");
        assert!(result1.is_err());
        assert_eq!(
            format!("{}", result1.unwrap_err()),
            "Date Error: No valid date format."
        );

        let result2 = parse("011020241234");
        assert!(result2.is_err());
        assert_eq!(
            format!("{}", result2.unwrap_err()),
            "Date Error: No valid date format."
        );
    }

    #[test]
    fn test_is_valid_fmt_valid() {
        assert!(is_valid_fmt("01102024"));
        assert!(is_valid_fmt("10102024"));
    }

    #[test]
    fn test_is_valid_fmt_invalid() {
        assert!(!is_valid_fmt("0110202a"));
        assert!(!is_valid_fmt("101020241234"));
        assert!(!is_valid_fmt(""));
    }

    #[test]
    fn test_validate_valid() {
        assert!(validate("01102024").is_ok());
        assert!(validate("10102024").is_ok());
    }

    #[test]
    fn test_validate_invalid() {
        assert!(validate("0110202a").is_err());
        assert!(validate("101020241234").is_err());
        assert!(validate("").is_err());
    }

    #[test]
    fn test_split_into_parent_date_valid() {
        let result = split_into_parent_date(Path::new("example-about_01102024.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ("example-about", "01102024"));
    }

    #[test]
    fn test_split_into_parent_date_invalid() {
        let result1 = split_into_parent_date(Path::new("example-about.txt"));
        assert!(result1.is_err());

        let result2 = split_into_parent_date(Path::new("example"));
        assert!(result2.is_err());
    }

    #[test]
    fn test_extract_dst_path_valid() {
        let target = Path::new("target");
        let parent_path = PathBuf::from("parent");

        let (parent, destination) =
            extract_dst_path(&parent_path.join("example-about_01102024.txt"), target)
                .expect("Unwrap failed on valid example.");
        assert_eq!(
            destination,
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
    fn test_extract_dst_path_invalid() {
        let target = Path::new("target");

        let result1 = extract_dst_path(&PathBuf::from("invalid_file_name.txt"), target);
        assert!(result1.is_err());

        let result2 = extract_dst_path(&PathBuf::from("example-about.txt"), target);
        assert!(result2.is_err());

        let result3 = extract_dst_path(&PathBuf::from("example-about"), target);
        assert!(result3.is_err());
    }
}
