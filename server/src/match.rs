use serde_derive::{Serialize, Deserialize};
//use serde::Deserialize;


#[derive(Debug, Serialize, Deserialize)]
pub struct Match
{
    pub epoch: i64,
    pub winner: String,
    pub loser: String,
    #[serde(skip_deserializing)]
    pub elo_diff: f64,
    #[serde(skip_deserializing)]
    pub winner_elo: f64,
    #[serde(skip_deserializing)]
    pub loser_elo: f64,
}

