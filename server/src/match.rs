use serde_derive::{Serialize, Deserialize};
//use serde::Deserialize;


#[derive(Debug, Serialize, Deserialize)]
pub struct Match
{
    pub winner: String,
    pub loser: String,
    pub epoch: i64
}
