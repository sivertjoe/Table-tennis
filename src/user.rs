use crate::r#match::Match;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct User 
{
    pub id: i64, 
    pub elo: f64,
    pub name: String,
    pub match_history: Vec<Match>
}
