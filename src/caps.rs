use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use smcan::Capability;

#[derive(PartialOrd, PartialEq, Eq, Clone, Ord, Serialize, Deserialize)]
pub enum Caps {
    All,
    Info,
    Issue,
    Revoke,
    Status,
    PathTest { path: String },
    Other,
    Empty,
}

impl Default for Caps {
    fn default() -> Self {
        Self::Empty
    }
}

impl Debug for Caps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Info => write!(f, "Info"),
            Self::Issue => write!(f, "Issue"),
            Self::Revoke => write!(f, "Revoke"),
            Self::Status => write!(f, "Status"),
            Self::PathTest { path } => f.debug_struct("PathTest").field("path", path).finish(),
            Self::Other => write!(f, "Other"),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl Capability for Caps {
    const KIND: &'static str = "anchor - ahoy";
    fn permits(&self, other: &Self) -> bool {
        match (self, other) {
            (Caps::All, _) => true,
            (Caps::Info, Caps::Info) => true,
            (Caps::Issue, Caps::Issue) => true,
            (Caps::Revoke, Caps::Revoke) => true,
            (Caps::Status, Caps::Status | Caps::Info) => true,
            (Caps::PathTest { path }, Caps::PathTest { path: otherpath }) => {
                self.path_check(path, otherpath)
            }
            (_, _) => false,
        }
    }
}

impl Caps {
    fn path_check(&self, source: &String, other: &String) -> bool {
        if !source.starts_with("/") {
            return false;
        }
        if !other.starts_with("/") {
            return false;
        }
        if other.starts_with(source) {
            return true;
        }
        false
    }
}
