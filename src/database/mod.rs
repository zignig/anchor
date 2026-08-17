// Database setup for auth and user bits
// Tables
//    endpoint
//    publisher

mod irpc;
mod db;


use std::{
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use geekorm::ConnectionManager;
use geekorm::Value;
use geekorm::{Connection, prelude::*};

use anyhow::Result;
use iroh::{EndpointId, PublicKey};
use serde::{Deserialize, Serialize};
use tracing::info;
// use turso::{Builder, Connection, params};