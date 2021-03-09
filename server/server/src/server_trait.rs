use crate::user::User;
use crate::server::{ServerResult, ServerError};
use crate::server::DataBase;
use rusqlite::NO_PARAMS;


trait Sql
{
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> where Self: Sized;
}



impl Sql for User
{
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>
    {
        Ok(
            User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                user_role: row.get(5)?,
                match_history: Vec::new(),
                badges: Vec::new()

            }
        )
    }
}


impl DataBase
{
    fn sql<T, S>(statement: S) -> ServerResult<T>
    where 
        T: Sql,
        S: AsRef<str> 
    {
        unimplemented!()
    }

    fn sql_one<T, S>(&self, s: S) -> ServerResult<T>
    where 
        T: Sql,
        S: AsRef<str> 
    {
        let mut stmt = self.conn.prepare(s.as_ref())?;

        let res: Option<rusqlite::Result<T>> = stmt.query_map(NO_PARAMS, |row| {
            T::from_row(&row)
        })?.next();

        match res 
        {
            Some(Ok(r)) => Ok(r),
            _ => Err(ServerError::Critical("foo".to_string()))
        }
    }

}
