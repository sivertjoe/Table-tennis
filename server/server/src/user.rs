use serde_derive::{Deserialize, Serialize};
use server_core::types::FromSql;
use server_macro::Sql;

use super::{badge::*, r#match::Match};

#[derive(Debug, Serialize, Sql)]
pub struct User
{
    pub id:            i64,
    pub elo:           f64,
    pub name:          String,
    pub user_role:     u8,
    pub match_history: Vec<Match>,
    pub badges:        Vec<Badge>,
}


#[derive(Deserialize)]
pub struct LoginInfo
{
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordInfo
{
    pub username:     String,
    pub password:     String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RequestResetPassword
{
    pub name: String,
}

#[derive(Deserialize)]
pub struct EditUsersInfo
{
    pub users:  Vec<String>,
    pub action: String,
    pub token:  String,
}

#[derive(Deserialize)]
pub struct StatsUsers
{
    pub user1: String,
    pub user2: String,
}

#[derive(Deserialize)]
pub struct AdminToken
{
    pub token: String,
}
