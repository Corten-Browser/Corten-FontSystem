//! Unit tests for ParseError enum

use font_parser::ParseError;

#[test]
fn test_parse_error_invalid_format() {
    /// Given an invalid format error
    /// When the error is created
    /// Then it should be the InvalidFormat variant
    let error = ParseError::InvalidFormat;
    match error {
        ParseError::InvalidFormat => assert!(true),
        _ => panic!("Expected InvalidFormat variant"),
    }
}

#[test]
fn test_parse_error_missing_table() {
    /// Given a missing table error with table name
    /// When the error is created
    /// Then it should contain the table name
    let error = ParseError::MissingTable("cmap".to_string());
    match error {
        ParseError::MissingTable(name) => assert_eq!(name, "cmap"),
        _ => panic!("Expected MissingTable variant"),
    }
}

#[test]
fn test_parse_error_corrupted_data() {
    /// Given corrupted data error with description
    /// When the error is created
    /// Then it should contain the description
    let error = ParseError::CorruptedData("Invalid checksum".to_string());
    match error {
        ParseError::CorruptedData(msg) => assert_eq!(msg, "Invalid checksum"),
        _ => panic!("Expected CorruptedData variant"),
    }
}

#[test]
fn test_parse_error_unsupported_version() {
    /// Given an unsupported version error
    /// When the error is created
    /// Then it should be the UnsupportedVersion variant
    let error = ParseError::UnsupportedVersion;
    match error {
        ParseError::UnsupportedVersion => assert!(true),
        _ => panic!("Expected UnsupportedVersion variant"),
    }
}

#[test]
fn test_parse_error_display() {
    /// Given a parse error
    /// When converted to string
    /// Then it should have a meaningful message
    let error = ParseError::MissingTable("head".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("head") || error_msg.contains("table"));
}

#[test]
fn test_parse_error_from_io() {
    /// Given an IO error
    /// When converting to ParseError
    /// Then it should be CorruptedData variant
    use std::io::{Error, ErrorKind};
    let io_error = Error::new(ErrorKind::UnexpectedEof, "EOF");
    let parse_error = ParseError::from(io_error);
    match parse_error {
        ParseError::CorruptedData(_) => assert!(true),
        _ => panic!("Expected CorruptedData variant from IO error"),
    }
}
