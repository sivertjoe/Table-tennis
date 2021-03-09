use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub struct Badge
{
    pub id:     i64,
    pub season: i64,
    pub name:   String,
}

pub const NUM_SEASON_PRIZES: usize = 3;
pub static BADGES: &'static [&'static str] =
    &["first_place.png", "second_place.png", "third_place.png", "default_tournament_win.png"];
