
#[macro_export]
macro_rules! SQL_TUPLE
{
    ($data:expr, $sql:expr, $($t: ty),+) => 
    {
        {
            type T = ($($t,)+);
            let mut stmt = $data.conn.prepare($sql.as_ref())?;
            let res = stmt.query_map(NO_PARAMS, |row|
            {
                $crate::SQL_TUPLE!(@CREATE_TYPE; row, $($t),+)
            })?;
            
            let mut vec = Vec::new();
            for elem in res
            {
                if let Ok(e) = elem
                {
                    vec.push(e);
                }
            }
            let _final: ServerResult<Vec<T>> = Ok(vec);
            _final
        }
    };


    (@CREATE_TYPE; $row: expr, $($t: ty),+) =>
    {
        {
        let mut counter: usize = 0; 
        Ok(($({ counter += 1;  $row.get::<_, $t>(counter - 1)? },)+))
        }
    };
}


#[macro_export]
macro_rules! SQL
{
    ($data:expr, $sql:expr, $func: expr) =>
    {
        {
            let mut stmt = $data.conn.prepare($sql.as_ref())?;
            let res = stmt.query_map(NO_PARAMS, |row|
            {
                $func(row)
            })?;
            
            let mut vec = Vec::new();
            for elem in res
            {
                if let Ok(e) = elem
                {
                    vec.push(e);
                }
            }
            let _final: ServerResult<Vec<_>> = Ok(vec);
            _final
        }
    };
}


#[macro_export]
macro_rules! TYPE
{
    (USER) => 
    {
        |row: &rusqlite::Row<'_>| -> rusqlite::Result<User> 
        { 
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                user_role: row.get(3)?,
                match_history: Vec::new(),
                badges: Vec::new(),
            })
        }
    };

    (MATCH) => 
    {
        |row: &rusqlite::Row<'_>| -> rusqlite::Result<Match> 
        {
            Ok(Match {
                winner:     row.get(0)?,
                loser:      row.get(1)?,
                elo_diff:   row.get(2)?,
                winner_elo: row.get(3)?,
                loser_elo:  row.get(4)?,
                epoch:      row.get(5)?,
                season:     -1,
            })
        }
    };

    (SEASON) => 
    {
        |row: &rusqlite::Row<'_>| -> rusqlite::Result<Season> 
        {
            Ok(Season {
                id: row.get(0)?, start_epoch: row.get(1)?
            })
        }
    }
}

// Kneel before my one-liner
#[macro_export]
macro_rules! GET_OR_CREATE_DB_VAR {
    ($conn: expr, $id: expr, $default_value: expr) => {
        $conn
            .prepare("select value from variables where id = :id")?
            .query_map_named(named_params! {":id": $id}, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })?
            .next()
            .map_or_else(
                || {
                    $conn
                        .execute("insert into variables (id, value) values (?1, ?2)", params![
                            $id,
                            $default_value
                        ])
                        .expect(&format!("Inserting into variables <{}, {}>", $id, $default_value));
                    Ok($default_value)
                },
                |val| Ok(val.unwrap()),
            )
    };
}


#[macro_export]
macro_rules! FILL
{
    (USER) => 
    {
       i64, f64, String, u8, Vec<Match>, Vec<Badge>
    }
}



#[cfg(test)]
mod test
{
    use crate::test_util::*;
    use crate::user::*;
    use crate::server::{ServerResult, DataBase};
    use rusqlite::{NO_PARAMS};

    #[test]
    fn test_insanity() -> ServerResult<()>
    {
        let db_file = "tempL1.db";
        let s = DataBase::new(db_file);
        let token = create_user(&s, "Sivert");

        let res = SQL_TUPLE!(s, "select id, name from users", i64, String).unwrap();
        let expected = (1, "Sivert".to_string());

        let res2 = SQL_TUPLE!(s, "select * from users", i64, String, f64, String, String, i64).unwrap();
        let expected2 = (1, "Sivert".to_string(), 1500., "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8".to_string(), token, (USER_ROLE_SOFT_INACTIVE | USER_ROLE_REGULAR) as i64);

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(expected, res[0]);
        assert_eq!(expected2, res2[0]);
        Ok(())
    }

    #[test]
    fn test_can_get_structures() -> ServerResult<()>
    {
        let db_file = "tempL2.db";
        let s = DataBase::new(db_file);
        let sql = "select id, name, elo, user_role from users;";
        create_user(&s, "Sivert");

        let users1 = SQL!(s, sql, TYPE!(USER)).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(users1.len() > 0);


        Ok(())
    }
}
