use serde_derive::Serialize;

#[derive(Debug, Serialize)]
pub struct Badge
{
    pub id:     i64,
    pub season: i64,
    pub name:   String,
}
