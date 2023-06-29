use std::fmt::Display;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PublishError {
    InvalidVersionFormat,
    DuplicateVersion,
    LatestVersionNotAllowed,
    RepositoryError(String),
}
impl Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishError::InvalidVersionFormat => write!(f, "Invalid version format"),
            PublishError::DuplicateVersion => write!(f, "Duplicate version"),
            PublishError::LatestVersionNotAllowed => write!(f, "Latest version not allowed"),
            PublishError::RepositoryError(e) => write!(f, "Repository error: {}", e),
        }
    }
}
