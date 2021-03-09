use serde_derive::{Deserialize, Serialize};


#[derive(Serialize)]
pub struct MatchNotification
{
    pub id:     i64,
    pub winner: String,
    pub loser:  String,
    pub epoch:  i64,
}

pub struct MatchNotificationTable
{
    pub id:            i64,
    pub winner_accept: u8,
    pub loser_accept:  u8,
    pub epoch:         i64,
    pub winner:        i64,
    pub loser:         i64,
}

#[derive(Serialize)]
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
