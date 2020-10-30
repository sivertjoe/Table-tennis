use serde_derive::Serialize;


#[derive(Debug, Serialize)]
pub struct Match
{
    pub winner: String,
    pub loser: String,
}
