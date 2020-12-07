use serde_derive::{Serialize};

#[derive(Debug, Serialize)]
pub struct Badge
{
    pub id: i64,
    pub season: i64,
    pub name: String,
    pub color: String,
}


pub static BADGES: &'static [&'static str] = &["crown", "medal", "medal", "award", "trophy"];
pub static BADGE_COLORS: &'static [&'static str] = &["yellow", "silver", "orange", "turquoise", "yellow"];

