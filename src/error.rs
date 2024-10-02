use crate::constants::SEPARATOR;

#[derive(Debug)]
pub enum ProcessError {
    Extension,
    Parts,
    Separator,
    Date,

    IO(std::io::Error),
}

impl ProcessError {
    pub fn extension() -> Self {
        ProcessError::Extension
    }
    pub fn parts() -> Self {
        ProcessError::Parts
    }
    pub fn separator() -> Self {
        ProcessError::Separator
    }
    pub fn date() -> Self {
        ProcessError::Date
    }
}

impl std::error::Error for ProcessError {}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Extension => write!(f, "Extension Error: No valid file extension."),
            ProcessError::Separator => {
                write!(
                    f,
                    "Separator Error: File name does not contain expected separator: {}.",
                    SEPARATOR
                )
            }
            ProcessError::Parts => {
                write!(f, "Parts Error: File name does not contain expected parts.")
            }
            ProcessError::Date => write!(f, "Date Error: No valid date format."),
            ProcessError::IO(error) => write!(f, "IO Error: {}", error),
        }
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        ProcessError::IO(value)
    }
}
