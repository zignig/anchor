use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Ord, Serialize, Deserialize)]
pub enum Caps {
    All,
    Info,
    Issue,
    Revoke,
    Status,
    PathTest { path: String },
}