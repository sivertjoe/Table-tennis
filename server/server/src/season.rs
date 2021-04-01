use server_core::types::FromSql;
use server_macro::Sql;

#[derive(Sql)]
pub struct Season
{
    pub id:          i64,
    pub start_epoch: i64,
}
