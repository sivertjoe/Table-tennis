use serde_derive::Serialize;

#[derive(Serialize)]
pub struct MatchNotification
{
    pub id: i64,
    pub winner: String,
    pub loser: String,
    pub epoch: i64,
}


pub struct MatchNotificationTable
{
    pub id: i64,
    pub winner_accept: u8,
    pub loser_accept: u8, 
    pub epoch: i64,
    pub elo_diff: f64,
    pub winner_elo: f64,
    pub loser_elo: f64,
    pub winner: i64,
    pub loser: i64
}
