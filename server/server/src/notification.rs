use serde_derive::{Deserialize, Serialize};
use server_core::types::FromSql;
use server_macro::Sql;
use serde::ser::{SerializeSeq, SerializeMap};
use std::collections::HashMap;




#[derive(Serialize, Sql)]
pub struct MatchNotification
{
    pub id:     i64,
    pub winner: String,
    pub loser:  String,
    pub epoch:  i64,
}

#[derive(Sql)]
pub struct MatchNotificationTable
{
    pub id:            i64,
    pub winner_accept: u8,
    pub loser_accept:  u8,
    pub epoch:         i64,
    pub winner:        i64,
    pub loser:         i64,
}

#[derive(Serialize, Sql)]
pub struct AdminNotification
{
    pub id:   i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct AdminNotificationAns
{
    pub id:    i64,
    pub token: String,
    pub ans:   u8,
}

#[derive(Debug, Deserialize)]
pub struct NotificationInfo
{
    pub r#type: String,
    pub token: String
}

pub enum NotificationType
{
    Admin,
    Match,
}

pub enum Notification
{
    Admin(HashMap<String, Vec<AdminNotification>>),
    Match(Vec<MatchNotification>),
}

impl serde::Serialize for Notification {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self
        {
            Notification::Admin(n) => {
                let mut map = serializer.serialize_map(Some(n.len()))?;
                for (k, v) in n
                {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Notification::Match(n) => {
                let mut seq = serializer.serialize_seq(Some(n.len()))?;
                for e in n
                {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
        }
    }
}


impl std::convert::TryFrom<String> for NotificationType
{
    type Error = ();

    fn try_from(action: String) -> Result<Self, Self::Error>
    {
        match action.as_str()
        {
            "match" => Ok(NotificationType::Match),
            "admin" => Ok(NotificationType::Admin),
            _ => Err(()),
        }
    }
}
