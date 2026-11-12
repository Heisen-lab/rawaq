use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported first-class languages in Rawaq (Arabic RTL & English LTR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ar,
    En,
}

/// Canonical layout direction associated with a language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    Rtl,
    Ltr,
}

impl Language {
    pub fn direction(self) -> TextDirection {
        match self {
            Self::Ar => TextDirection::Rtl,
            Self::En => TextDirection::Ltr,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Ar => "ar",
            Self::En => "en",
        }
    }
}

/// Errors emitted by `CommandGate` and `PathSanitizer` when validating paths or user inputs.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SecurityError {
    #[error("Path cannot be empty or whitespace only")]
    EmptyPath,

    #[error("Path contains null bytes or control characters")]
    ControlOrNullByteDetected,

    #[error("Path exceeds maximum safe length ({max} characters)")]
    PathTooLong { max: usize },

    #[error("Path component exceeds maximum safe length ({max} characters)")]
    ComponentTooLong { max: usize },

    #[error("Path contains relative traversal or suspicious dot sequence")]
    PathTraversalDetected,

    #[error("Path contains URL-encoded traversal or null sequence")]
    EncodedTraversalDetected,

    #[error("UNC or network share paths are not permitted")]
    UncNetworkPathRejected,

    #[error("Path targets a reserved OS device name: {name}")]
    ReservedDeviceName { name: String },

    #[error("Path targets a restricted operating system directory")]
    SystemDirectoryForbidden,

    #[error("Path is outside user-authorized folder scope")]
    OutsideAuthorizedScope,

    #[error("URI uses a disallowed or dangerous protocol scheme")]
    DangerousUriScheme,
}
