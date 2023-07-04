use std::fmt::Display;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PublishError {
    InvalidVersionFormat,
    DuplicateVersionName,
    DuplicateVersionNameAndUri,
    LatestVersionNotAllowed,
    RepositoryError(String),
}
impl Display for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishError::InvalidVersionFormat => write!(f, "Invalid version format"),
            PublishError::DuplicateVersionName => write!(f, "Duplicate version name"),
            PublishError::DuplicateVersionNameAndUri => write!(f, "Duplicate version name and URI"),
            PublishError::LatestVersionNotAllowed => write!(f, "Latest version not allowed"),
            PublishError::RepositoryError(e) => write!(f, "Repository error: {}", e),
        }
    }
}
