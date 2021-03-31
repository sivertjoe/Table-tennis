pub trait FromSql
{
    fn from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>
    where
        Self: Sized;
}
