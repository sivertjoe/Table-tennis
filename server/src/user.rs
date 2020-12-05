use std::str::FromStr;
use crate::r#match::Match;
use serde_derive::{Serialize, Deserialize};
use crate::badge::*;


pub const USER_ROLE_REGULAR: u8 = 0;
pub const USER_ROLE_SUPERUSER: u8 = 1 << 1;
pub const USER_ROLE_INACTIVE: u8 = 1 << 2;

pub enum EditUserAction
{
    MakeUserActive,
    MakeUserRegular,
    MakeUserInactive,
    MakeUserSuperuser,
}

#[derive(Debug, Serialize)]
pub struct User
{
    pub id: i64,
    pub elo: f64,
    pub name: String,
    pub user_role: u8,
    pub match_history: Vec<Match>,
    pub badges: Vec<Badge>,
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

#[derive(Deserialize)]
pub struct EditUsersInfo
{
    pub users: Vec<String>,
    pub action: String,
    pub token: String,
}

impl FromStr for EditUserAction
{
    type Err = ();
    fn from_str(action: &str) -> Result<Self, Self::Err>
    {
        match action
        {
            "MAKE_USER_ACTIVE" => Ok(EditUserAction::MakeUserActive),
            "MAKE_USER_REGULAR" => Ok(EditUserAction::MakeUserRegular),
            "MAKE_USER_INACTIVE" => Ok(EditUserAction::MakeUserInactive),
            "MAKE_USER_SUPERUSER" => Ok(EditUserAction::MakeUserSuperuser),
            _ => Err(())
        }
    }
}
