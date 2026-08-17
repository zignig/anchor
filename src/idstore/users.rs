use geekorm::prelude::*;
use geekorm::{Data, PrimaryKeyInteger, Table};

#[derive(Data, Debug, Clone, Default)]
pub enum UserType {
    Admin,
    #[default]
    User,
    Guest,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    username: String,

    #[geekorm(unique)]
    email: String,

    user_type: UserType,

    #[geekorm(new = false)]
    active: bool,
    postcode: Option<String>,
}
