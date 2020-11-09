use serde_derive::{Serialize, Deserialize};


#[derive(Debug, Serialize)]
pub struct Match
{
    pub winner: String,
    pub loser: String,
    pub epoch: i64,
    pub elo_diff: f64,
    pub winner_elo: f64,
    pub loser_elo: f64,
}


// Match info sent from the front end
#[derive(Debug, Deserialize)]
pub struct MatchInfo
{
    pub winner: String,
    pub loser: String,
    pub token: String
}

#[derive(Debug, Deserialize)]
pub struct MatchResponse
{
    pub match_notification_id: i64,
    pub ans: u8,
    pub user_token: String,
}

