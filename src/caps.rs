use serde::{Deserialize, Serialize};
use smcan::Capability;

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Ord, Serialize, Deserialize)]
pub enum Caps {
    All,
    Info,
    Issue,
    Revoke,
    Status,
    PathTest { path: String },
}

impl Capability for Caps {
    fn permits(&self, other: &Self) -> bool {
        match (self, other) {
            (Caps::All, _) => true,
            (Caps::Info, Caps::Info) => true,
            (Caps::Issue, Caps::Issue) => true,
            (Caps::Revoke, Caps::Revoke) => true,
            (Caps::Status, Caps::Status) => true,
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