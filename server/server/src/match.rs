use serde_derive::{Deserialize, Serialize};
use server_core::types::FromSql;
use server_macro::Sql;


#[derive(Debug, Serialize, Sql)]
pub struct Match
{
    pub winner:     String,
    pub loser:      String,
    pub epoch:      i64,
    pub elo_diff:   f64,
    pub winner_elo: f64,
    pub loser_elo:  f64,
    pub season:     i64,
}

#[derive(Debug, Serialize, Sql)]
pub struct EditMatchInfo
{
    pub winner: String,
    pub loser:  String,
    pub epoch:  i64,
    pub id:     i64,
}

#[derive(Debug, Deserialize)]
pub struct DeleteMatchInfo
{
    pub id:    i64,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct NewEditMatchInfo
{
    pub winner: String,
    pub loser:  String,
    pub epoch:  i64,
    pub id:     i64,
    pub token:  String,
}

// Match info sent from the front end
#[derive(Debug, Deserialize)]
pub struct MatchInfo
{
    pub winner: String,
    pub loser:  String,
    pub token:  String,
}
