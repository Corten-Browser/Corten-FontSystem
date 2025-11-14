//! Error types for font parsing

use std::fmt;
use std::io;

/// Font parsing errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Invalid font format
    InvalidFormat,
    /// Required table is missing
    MissingTable(String),
    /// Font data is corrupted
    CorruptedData(String),
    /// Unsupported font version
    UnsupportedVersion,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "Invalid font format"),
            ParseError::MissingTable(table) => write!(f, "Missing required table: {}", table),
            ParseError::CorruptedData(msg) => write!(f, "Corrupted font data: {}", msg),
            ParseError::UnsupportedVersion => write!(f, "Unsupported font version"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::CorruptedData(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_variants() {
        // Test all variants can be created
        let _ = ParseError::InvalidFormat;
        let _ = ParseError::MissingTable("test".to_string());
        let _ = ParseError::CorruptedData("test".to_string());
        let _ = ParseError::UnsupportedVersion;
    }

    #[test]
    fn test_parse_error_display_messages() {
        assert_eq!(
            format!("{}", ParseError::InvalidFormat),
            "Invalid font format"
        );
        assert_eq!(
            format!("{}", ParseError::MissingTable("cmap".to_string())),
            "Missing required table: cmap"
        );
        assert_eq!(
            format!("{}", ParseError::CorruptedData("bad data".to_string())),
            "Corrupted font data: bad data"
        );
        assert_eq!(
            format!("{}", ParseError::UnsupportedVersion),
            "Unsupported font version"
        );
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "EOF");
        let parse_err = ParseError::from(io_err);
        match parse_err {
            ParseError::CorruptedData(_) => (),
            _ => panic!("Expected CorruptedData variant"),
        }
    }
}
