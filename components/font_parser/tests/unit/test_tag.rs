//! Unit tests for Tag struct

use font_parser::Tag;

#[test]
fn test_tag_from_str_valid() {
    // Given a valid 4-character string
    // When creating a Tag
    // Then it should succeed
    let tag: Result<Tag, _> = "cmap".parse();
    assert!(tag.is_ok());
    let tag = tag.unwrap();
    assert_eq!(tag.as_str(), "cmap");
}

#[test]
fn test_tag_from_str_case_sensitive() {
    // Given strings with different cases
    // When creating Tags
    // Then they should be case-sensitive
    let tag1: Tag = "cmap".parse().unwrap();
    let tag2: Tag = "CMAP".parse().unwrap();
    assert_ne!(tag1, tag2);
}

#[test]
fn test_tag_from_str_too_short() {
    // Given a string shorter than 4 characters
    // When creating a Tag
    // Then it should fail
    let tag: Result<Tag, _> = "abc".parse();
    assert!(tag.is_err());
}

#[test]
fn test_tag_from_str_too_long() {
    // Given a string longer than 4 characters
    // When creating a Tag
    // Then it should fail
    let tag: Result<Tag, _> = "cmapx".parse();
    assert!(tag.is_err());
}

#[test]
fn test_tag_from_str_empty() {
    // Given an empty string
    // When creating a Tag
    // Then it should fail
    let tag: Result<Tag, _> = "".parse();
    assert!(tag.is_err());
}

#[test]
fn test_tag_equality() {
    // Given two Tags with same value
    // When comparing them
    // Then they should be equal
    let tag1: Tag = "head".parse().unwrap();
    let tag2: Tag = "head".parse().unwrap();
    assert_eq!(tag1, tag2);
}

#[test]
fn test_tag_inequality() {
    // Given two Tags with different values
    // When comparing them
    // Then they should not be equal
    let tag1: Tag = "head".parse().unwrap();
    let tag2: Tag = "cmap".parse().unwrap();
    assert_ne!(tag1, tag2);
}

#[test]
fn test_tag_display() {
    // Given a Tag
    // When converting to string
    // Then it should display the tag name
    let tag: Tag = "glyf".parse().unwrap();
    let tag_str = format!("{}", tag);
    assert_eq!(tag_str, "glyf");
}

#[test]
fn test_tag_from_bytes() {
    // Given a 4-byte value
    // When creating a Tag
    // Then it should match expected string
    let tag = Tag::from_bytes(0x636D6170); // "cmap" in ASCII
    assert_eq!(tag.as_str(), "cmap");
}

#[test]
fn test_tag_to_bytes() {
    // Given a Tag created from string
    // When converting to bytes
    // Then it should match the original bytes
    let tag: Tag = "head".parse().unwrap();
    let bytes = tag.to_bytes();
    assert_eq!(bytes, 0x68656164); // "head" in ASCII
}

#[test]
fn test_tag_common_tags() {
    // Test common OpenType table tags
    let common_tags = vec![
        "cmap", "glyf", "head", "hhea", "hmtx", "loca", "maxp", "name", "post",
    ];

    for tag_str in common_tags {
        let tag: Result<Tag, _> = tag_str.parse();
        assert!(tag.is_ok(), "Failed to create tag for: {}", tag_str);
        assert_eq!(tag.unwrap().as_str(), tag_str);
    }
}
