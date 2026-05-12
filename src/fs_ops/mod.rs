//! Pure path helpers for sidebar file operations: validate a user-typed
//! name and resolve a target path under a parent directory.
//!
//! All filesystem I/O lives in `app.rs`; these functions never touch disk.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    Empty,
    ContainsSeparator,
    ContainsNull,
    Reserved,
    DotOrDoubleDot,
}

impl NameError {
    pub fn message(&self) -> &'static str {
        match self {
            NameError::Empty => "Name cannot be empty",
            NameError::ContainsSeparator => "Name cannot contain '/' or '\\'",
            NameError::ContainsNull => "Name cannot contain NUL bytes",
            NameError::Reserved => "Name is reserved by the OS",
            NameError::DotOrDoubleDot => "Name cannot be '.' or '..'",
        }
    }
}

/// Validate a user-typed file/folder name. Rejects empty strings,
/// names with path separators, NUL bytes, and the special `.`/`..`
/// values. Otherwise returns the name unchanged.
pub fn validate_name(name: &str) -> Result<&str, NameError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(NameError::Empty);
    }
    if trimmed == "." || trimmed == ".." {
        return Err(NameError::DotOrDoubleDot);
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(NameError::ContainsSeparator);
    }
    if trimmed.contains('\0') {
        return Err(NameError::ContainsNull);
    }
    // Common Windows reserved names — cheap to reject everywhere.
    let upper = trimmed.to_ascii_uppercase();
    let upper_stem = upper.split('.').next().unwrap_or("");
    if matches!(
        upper_stem,
        "CON" | "PRN" | "AUX" | "NUL"
            | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9"
            | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    ) {
        return Err(NameError::Reserved);
    }
    Ok(trimmed)
}

/// Build a target path under `parent` for the given `name`. Validates
/// the name first; the parent is treated as opaque and assumed to be a
/// directory that already exists.
pub fn resolve_under(parent: &Path, name: &str) -> Result<PathBuf, NameError> {
    let validated = validate_name(name)?;
    Ok(parent.join(validated))
}

/// New path produced by renaming `existing` to `new_name`. Keeps the
/// parent directory; updates only the final component.
pub fn rename_target(existing: &Path, new_name: &str) -> Result<PathBuf, NameError> {
    let validated = validate_name(new_name)?;
    let parent = existing.parent().unwrap_or_else(|| Path::new(""));
    Ok(parent.join(validated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_name_rejected() {
        assert_eq!(validate_name(""), Err(NameError::Empty));
        assert_eq!(validate_name("   "), Err(NameError::Empty));
    }

    #[test]
    fn dot_and_double_dot_rejected() {
        assert_eq!(validate_name("."), Err(NameError::DotOrDoubleDot));
        assert_eq!(validate_name(".."), Err(NameError::DotOrDoubleDot));
    }

    #[test]
    fn slash_or_backslash_rejected() {
        assert_eq!(validate_name("foo/bar"), Err(NameError::ContainsSeparator));
        assert_eq!(validate_name("foo\\bar"), Err(NameError::ContainsSeparator));
    }

    #[test]
    fn null_byte_rejected() {
        let s = format!("foo{}bar", '\0');
        assert_eq!(validate_name(&s), Err(NameError::ContainsNull));
    }

    #[test]
    fn reserved_windows_names_rejected() {
        assert_eq!(validate_name("CON"), Err(NameError::Reserved));
        assert_eq!(validate_name("con"), Err(NameError::Reserved));
        assert_eq!(validate_name("NUL.txt"), Err(NameError::Reserved));
        assert_eq!(validate_name("COM1.log"), Err(NameError::Reserved));
    }

    #[test]
    fn normal_names_pass() {
        assert!(validate_name("hello.txt").is_ok());
        assert!(validate_name(".gitignore").is_ok());
        assert!(validate_name("My File.md").is_ok());
        assert!(validate_name("café.txt").is_ok());
    }

    #[test]
    fn name_is_trimmed_in_result() {
        assert_eq!(validate_name("  hello  "), Ok("hello"));
    }

    #[test]
    fn resolve_under_joins_to_parent() {
        let p = Path::new("/work/src");
        let r = resolve_under(p, "main.rs").unwrap();
        assert_eq!(r, PathBuf::from("/work/src/main.rs"));
    }

    #[test]
    fn resolve_under_propagates_validation_error() {
        let p = Path::new("/work/src");
        assert_eq!(
            resolve_under(p, "evil/path"),
            Err(NameError::ContainsSeparator)
        );
    }

    #[test]
    fn rename_target_keeps_parent_dir() {
        let r = rename_target(Path::new("/work/src/main.rs"), "lib.rs").unwrap();
        assert_eq!(r, PathBuf::from("/work/src/lib.rs"));
    }

    #[test]
    fn rename_target_works_at_filesystem_root() {
        let r = rename_target(Path::new("/foo.txt"), "bar.txt").unwrap();
        // Parent of "/foo.txt" is "/", so the rename target is "/bar.txt".
        assert_eq!(r, PathBuf::from("/bar.txt"));
    }
}
