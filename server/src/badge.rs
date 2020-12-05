use serde_derive::{Serialize};

#[derive(Debug, Serialize)]
pub struct Badge
{
    pub id: i64,
    pub season: i64,
    pub badge_name: String,
}


pub static BADGES: &'static [&'static str] = &["trophy", "medal", "medal", "award"];

