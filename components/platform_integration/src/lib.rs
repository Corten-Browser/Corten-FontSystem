//! platform_integration - Platform-specific font discovery (Linux, Windows, macOS)

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::collections::HashMap;
use std::path::PathBuf;

pub mod types;

pub use types::{FontCategory, FontStyle, FontWeight, Platform, PlatformFontInfo};

/// Discover system fonts with detailed metadata (family, weight, style)
///
/// This function uses platform-specific APIs to discover fonts and parse their metadata.
/// On Linux, it uses fontconfig. On Windows and macOS, this is currently a stub.
///
/// # Returns
///
/// A vector of `PlatformFontInfo` structures containing detailed font metadata.
///
/// # Examples
///
/// ```no_run
/// use platform_integration::discover_system_fonts_detailed;
///
/// let fonts = discover_system_fonts_detailed();
/// for font in fonts {
///     println!("{}: {:?} {:?}", font.family_name, font.weight, font.style);
/// }
/// ```
pub fn discover_system_fonts_detailed() -> Vec<PlatformFontInfo> {
    #[cfg(target_os = "linux")]
    return linux::discover_fonts_detailed();

    #[cfg(target_os = "windows")]
    return windows::discover_fonts_detailed();

    #[cfg(target_os = "macos")]
    return macos::discover_fonts_detailed();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    Vec::new()
}

/// Discover system fonts for the current platform
///
/// Returns a vector of paths to font files found on the system.
///
/// # Examples
///
/// ```no_run
/// use platform_integration::discover_system_fonts;
///
/// let fonts = discover_system_fonts();
/// println!("Found {} fonts", fonts.len());
/// ```
pub fn discover_system_fonts() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    return linux::discover_fonts();

    #[cfg(target_os = "windows")]
    return windows::discover_fonts();

    #[cfg(target_os = "macos")]
    return macos::discover_fonts();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    Vec::new()
}

/// Get default font families for each category on the current platform
///
/// Returns a HashMap mapping font categories to lists of font family names.
///
/// # Examples
///
/// ```no_run
/// use platform_integration::{get_default_font_families, FontCategory};
///
/// let defaults = get_default_font_families();
/// if let Some(monospace_fonts) = defaults.get(&FontCategory::Monospace) {
///     println!("Monospace fonts: {:?}", monospace_fonts);
/// }
/// ```
pub fn get_default_font_families() -> HashMap<FontCategory, Vec<String>> {
    #[cfg(target_os = "linux")]
    return linux::get_defaults();

    #[cfg(target_os = "windows")]
    return windows::get_defaults();

    #[cfg(target_os = "macos")]
    return macos::get_defaults();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    HashMap::new()
}

/// Get the platform-specific font configuration directory path
///
/// Returns the path to the font configuration directory if it exists.
///
/// # Examples
///
/// ```no_run
/// use platform_integration::get_font_config_path;
///
/// if let Some(config_path) = get_font_config_path() {
///     println!("Font config at: {:?}", config_path);
/// }
/// ```
pub fn get_font_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    return linux::get_config_path();

    #[cfg(target_os = "windows")]
    return windows::get_config_path();

    #[cfg(target_os = "macos")]
    return macos::get_config_path();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    None
}

/// Detect current platform
///
/// # Examples
///
/// ```
/// use platform_integration::detect_platform;
///
/// let platform = detect_platform();
/// println!("Running on: {:?}", platform);
/// ```
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(target_os = "macos")]
    return Platform::MacOS;

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    Platform::Unknown
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::fs;
    use std::process::Command;

    /// Discover fonts with detailed metadata using fontconfig
    pub fn discover_fonts_detailed() -> Vec<PlatformFontInfo> {
        let mut fonts = Vec::new();

        // Try to use fontconfig library for detailed metadata
        match use_fontconfig_library() {
            Ok(fc_fonts) => {
                fonts.extend(fc_fonts);
            }
            Err(e) => {
                eprintln!(
                    "Fontconfig library error: {}. Falling back to basic discovery.",
                    e
                );
                // Fall back to basic discovery without metadata
                let paths = discover_fonts();
                for path in paths {
                    // Create default font info for paths we found
                    fonts.push(PlatformFontInfo::new(
                        extract_family_from_path(&path),
                        path,
                        FontWeight::Regular,
                        FontStyle::Normal,
                        true,
                    ));
                }
            }
        }

        fonts
    }

    /// Extract family name from font file path
    fn extract_family_from_path(path: &std::path::Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Use fontconfig library to discover fonts with detailed metadata
    fn use_fontconfig_library() -> Result<Vec<PlatformFontInfo>, String> {
        use fontconfig::Fontconfig;

        let fc = Fontconfig::new().ok_or("Failed to initialize fontconfig")?;

        // Use sort_fonts with an empty pattern to get all fonts
        let pattern = fontconfig::Pattern::new(&fc);
        let font_set = fontconfig::sort_fonts(&pattern, true);

        let mut fonts = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        // Iterate through fonts
        // Note: We need to be careful as fontconfig can return duplicates
        for font in font_set.iter() {
            // Extract file path
            let file_path = match font.filename() {
                Some(f) => {
                    let path = PathBuf::from(f);
                    // Skip if we've already seen this path
                    if !seen_paths.insert(path.clone()) {
                        continue;
                    }
                    path
                }
                None => continue, // Skip fonts without file path
            };

            // Only process existing files
            if !file_path.exists() {
                continue;
            }

            // Extract family name
            let family = font.name().unwrap_or("Unknown").to_string();

            // Parse weight (fontconfig uses different scale than we expect)
            let fc_weight = font.weight().unwrap_or(80); // Default: Regular
            let weight = map_fontconfig_weight(fc_weight);

            // Parse style/slant
            let fc_slant = font.slant().unwrap_or(0); // Default: Normal
            let style = map_fontconfig_slant(fc_slant);

            // Determine if system font based on path
            let is_system_font = file_path
                .to_str()
                .map(|s| s.starts_with("/usr/") || s.starts_with("/lib/") || s.starts_with("/etc/"))
                .unwrap_or(false);

            fonts.push(PlatformFontInfo::new(
                family,
                file_path,
                weight,
                style,
                is_system_font,
            ));
        }

        if fonts.is_empty() {
            Err("No fonts found via fontconfig".to_string())
        } else {
            Ok(fonts)
        }
    }

    /// Map fontconfig weight value to FontWeight enum
    ///
    /// Fontconfig weight values:
    /// - 0 (Thin) to 210+ (Black)
    /// - 80 is Regular
    /// - 200 is Bold
    fn map_fontconfig_weight(fc_weight: i32) -> FontWeight {
        match fc_weight {
            0..=40 => FontWeight::Thin,
            41..=55 => FontWeight::ExtraLight,
            56..=75 => FontWeight::Light,
            76..=90 => FontWeight::Regular,
            91..=110 => FontWeight::Medium,
            111..=180 => FontWeight::SemiBold,
            181..=200 => FontWeight::Bold,
            201..=209 => FontWeight::ExtraBold,
            _ => FontWeight::Black,
        }
    }

    /// Map fontconfig slant value to FontStyle enum
    ///
    /// Fontconfig slant values:
    /// - 0: Roman (Normal)
    /// - 100: Italic
    /// - 110: Oblique
    fn map_fontconfig_slant(fc_slant: i32) -> FontStyle {
        match fc_slant {
            0 => FontStyle::Normal,
            100 => FontStyle::Italic,
            110 => FontStyle::Oblique(10.0), // ~10 degree oblique angle
            _ => FontStyle::Normal,
        }
    }

    /// Discover fonts on Linux using fontconfig and common directories
    pub fn discover_fonts() -> Vec<PathBuf> {
        let mut fonts = Vec::new();

        // Try using fc-list first
        if let Ok(output) = Command::new("fc-list").arg("--format=%{file}\n").output() {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                for path in paths.lines() {
                    let path = path.trim();
                    if !path.is_empty() {
                        if let Ok(pb) = PathBuf::from(path).canonicalize() {
                            if pb.exists() {
                                fonts.push(pb);
                            }
                        }
                    }
                }
            }
        }

        // Also check common directories
        let common_dirs = [
            "/usr/share/fonts",
            "/usr/local/share/fonts",
            "~/.fonts",
            "~/.local/share/fonts",
        ];

        for dir in &common_dirs {
            let expanded = expand_home(dir);
            if let Ok(entries) = fs::read_dir(&expanded) {
                for entry in entries.filter_map(Result::ok) {
                    scan_directory_recursive(entry.path(), &mut fonts);
                }
            }
        }

        // Remove duplicates
        fonts.sort();
        fonts.dedup();
        fonts
    }

    fn scan_directory_recursive(path: PathBuf, fonts: &mut Vec<PathBuf>) {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.filter_map(Result::ok) {
                    scan_directory_recursive(entry.path(), fonts);
                }
            }
        } else if is_font_file(&path) {
            fonts.push(path);
        }
    }

    fn is_font_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc" | "woff" | "woff2" | "pfb" | "pfa"
            )
        } else {
            false
        }
    }

    fn expand_home(path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = std::env::var_os("HOME") {
                let mut result = PathBuf::from(home);
                result.push(stripped);
                return result;
            }
        }
        PathBuf::from(path)
    }

    /// Get default font families for Linux
    pub fn get_defaults() -> HashMap<FontCategory, Vec<String>> {
        let mut defaults = HashMap::new();

        defaults.insert(
            FontCategory::Serif,
            vec![
                "DejaVu Serif".to_string(),
                "Liberation Serif".to_string(),
                "Noto Serif".to_string(),
                "Times New Roman".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::SansSerif,
            vec![
                "DejaVu Sans".to_string(),
                "Liberation Sans".to_string(),
                "Noto Sans".to_string(),
                "Arial".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Monospace,
            vec![
                "DejaVu Sans Mono".to_string(),
                "Liberation Mono".to_string(),
                "Noto Mono".to_string(),
                "Courier New".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Cursive,
            vec!["URW Chancery L".to_string(), "Comic Sans MS".to_string()],
        );

        defaults.insert(
            FontCategory::Fantasy,
            vec!["Impact".to_string(), "Copperplate".to_string()],
        );

        defaults.insert(
            FontCategory::Emoji,
            vec!["Noto Color Emoji".to_string(), "Emoji One".to_string()],
        );

        defaults
    }

    /// Get font config path for Linux
    pub fn get_config_path() -> Option<PathBuf> {
        let paths = vec![
            PathBuf::from("/etc/fonts"),
            expand_home("~/.config/fontconfig"),
        ];

        paths.into_iter().find(|p| p.exists())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    /// Discover fonts with detailed metadata on Windows
    ///
    /// # TODO
    ///
    /// This is currently a stub. Full implementation will use DirectWrite API:
    /// - Use windows-rs crate for DirectWrite bindings
    /// - Query IDWriteFontCollection for system fonts
    /// - Parse font properties (family, weight, style, stretch)
    /// - Map DirectWrite enums to our FontWeight/FontStyle enums
    ///
    /// See: https://docs.microsoft.com/en-us/windows/win32/directwrite/direct-write-portal
    pub fn discover_fonts_detailed() -> Vec<PlatformFontInfo> {
        eprintln!("WARNING: discover_system_fonts_detailed() not yet implemented for Windows");
        eprintln!("         Falling back to basic path discovery");

        // Fall back to basic discovery
        let paths = discover_fonts();
        paths
            .into_iter()
            .map(|path| {
                let family = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                PlatformFontInfo::new(family, path, FontWeight::Regular, FontStyle::Normal, true)
            })
            .collect()
    }

    /// Discover fonts on Windows
    pub fn discover_fonts() -> Vec<PathBuf> {
        let mut fonts = Vec::new();

        // Windows fonts directory
        if let Some(windir) = std::env::var_os("WINDIR") {
            let fonts_dir = PathBuf::from(windir).join("Fonts");
            if let Ok(entries) = std::fs::read_dir(&fonts_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if is_font_file(&path) {
                        fonts.push(path);
                    }
                }
            }
        }

        fonts
    }

    fn is_font_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc"
            )
        } else {
            false
        }
    }

    /// Get default font families for Windows
    pub fn get_defaults() -> HashMap<FontCategory, Vec<String>> {
        let mut defaults = HashMap::new();

        defaults.insert(
            FontCategory::Serif,
            vec![
                "Times New Roman".to_string(),
                "Georgia".to_string(),
                "Palatino Linotype".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::SansSerif,
            vec![
                "Arial".to_string(),
                "Verdana".to_string(),
                "Tahoma".to_string(),
                "Segoe UI".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Monospace,
            vec![
                "Courier New".to_string(),
                "Consolas".to_string(),
                "Lucida Console".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Cursive,
            vec!["Comic Sans MS".to_string(), "Brush Script MT".to_string()],
        );

        defaults.insert(
            FontCategory::Fantasy,
            vec!["Impact".to_string(), "Copperplate Gothic".to_string()],
        );

        defaults.insert(FontCategory::Emoji, vec!["Segoe UI Emoji".to_string()]);

        defaults
    }

    /// Get font config path for Windows
    pub fn get_config_path() -> Option<PathBuf> {
        if let Some(windir) = std::env::var_os("WINDIR") {
            Some(PathBuf::from(windir).join("Fonts"))
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    /// Discover fonts with detailed metadata on macOS
    ///
    /// # TODO
    ///
    /// This is currently a stub. Full implementation will use CoreText API:
    /// - Use core-foundation-rs crate for CoreText bindings
    /// - Query CTFontCollection for system fonts
    /// - Parse font descriptors for properties (family, traits, weight)
    /// - Map CoreText traits to our FontWeight/FontStyle enums
    ///
    /// See: https://developer.apple.com/documentation/coretext
    pub fn discover_fonts_detailed() -> Vec<PlatformFontInfo> {
        eprintln!("WARNING: discover_system_fonts_detailed() not yet implemented for macOS");
        eprintln!("         Falling back to basic path discovery");

        // Fall back to basic discovery
        let paths = discover_fonts();
        paths
            .into_iter()
            .map(|path| {
                let family = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                PlatformFontInfo::new(family, path, FontWeight::Regular, FontStyle::Normal, true)
            })
            .collect()
    }

    /// Discover fonts on macOS
    pub fn discover_fonts() -> Vec<PathBuf> {
        let mut fonts = Vec::new();

        let font_dirs = vec![
            PathBuf::from("/System/Library/Fonts"),
            PathBuf::from("/Library/Fonts"),
        ];

        if let Some(home) = std::env::var_os("HOME") {
            let mut user_fonts = PathBuf::from(home);
            user_fonts.push("Library/Fonts");
            font_dirs.push(user_fonts);
        }

        for dir in font_dirs {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if is_font_file(&path) {
                        fonts.push(path);
                    }
                }
            }
        }

        fonts
    }

    fn is_font_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc" | "dfont"
            )
        } else {
            false
        }
    }

    /// Get default font families for macOS
    pub fn get_defaults() -> HashMap<FontCategory, Vec<String>> {
        let mut defaults = HashMap::new();

        defaults.insert(
            FontCategory::Serif,
            vec![
                "Times New Roman".to_string(),
                "Georgia".to_string(),
                "Palatino".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::SansSerif,
            vec![
                "Helvetica".to_string(),
                "Arial".to_string(),
                "Verdana".to_string(),
                "San Francisco".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Monospace,
            vec![
                "Courier".to_string(),
                "Monaco".to_string(),
                "Menlo".to_string(),
            ],
        );

        defaults.insert(
            FontCategory::Cursive,
            vec!["Apple Chancery".to_string(), "Comic Sans MS".to_string()],
        );

        defaults.insert(
            FontCategory::Fantasy,
            vec!["Papyrus".to_string(), "Impact".to_string()],
        );

        defaults.insert(FontCategory::Emoji, vec!["Apple Color Emoji".to_string()]);

        defaults
    }

    /// Get font config path for macOS
    pub fn get_config_path() -> Option<PathBuf> {
        Some(PathBuf::from("/Library/Fonts"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_platform() {
        let platform = detect_platform();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_discover_system_fonts_returns_vec() {
        let _fonts = discover_system_fonts();
        // Should return a Vec (may be empty on some systems)
        // Just checking it doesn't panic
    }

    #[test]
    fn test_get_default_font_families_returns_map() {
        let defaults = get_default_font_families();
        // Should have entries for all categories
        assert!(defaults.contains_key(&FontCategory::Serif));
        assert!(defaults.contains_key(&FontCategory::SansSerif));
        assert!(defaults.contains_key(&FontCategory::Monospace));
    }

    #[test]
    fn test_get_font_config_path_returns_option() {
        let path = get_font_config_path();
        // Should return Some or None
        match path {
            Some(_) => assert!(true),
            None => assert!(true),
        }
    }
}
