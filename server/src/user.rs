use crate::r#match::Match;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct User 
{
    pub id: i64, 
    pub elo: f64,
    pub name: String,
    pub user_role: u8,
    pub match_history: Vec<Match>
}

#[derive(Deserialize)]
pub struct LoginInfo
{
    pub username: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct ChangePasswordInfo
{
    pub username: String,
    pub password: String,
    pub new_password: String
}
