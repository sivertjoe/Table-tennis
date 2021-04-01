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

    ($data:expr, $sql:expr, $named_params:expr, $($t: ty),+) =>
    {
        {
            type T = ($($t,)+);
            let mut stmt = $data.conn.prepare($sql.as_ref())?;
            let res = stmt.query_map_named($named_params, |row|
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

// Rust didn't like this pattern above 🤷‍♀️
#[macro_export]
macro_rules! SQL_TUPLE_NAMED
{
    ($data:expr, $sql:expr, $named_params:expr, $($t: ty),+) =>
    {
        {
            type T = ($($t,)+);
            let mut stmt = $data.conn.prepare($sql.as_ref())?;
            let res = stmt.query_map_named($named_params, |row|
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
macro_rules! _params {
        () => {
            $crate::NO_PARAMS
        };
        ($($param:expr),+ $(,)?) => {
            Some(ParamsType::Params(&[$(&$param as &dyn rusqlite::ToSql),+] as &[&dyn $crate::ToSql]))
        };
}

#[macro_export]
macro_rules! _named_params {
        () => {
                    &[]
                            };
            // Note: It's a lot more work to support this as part of the same macro as
                     ($($param_name:literal: $param_val:expr),+ $(,)?) => {
                             Some(ParamsType::Named(&[$(($param_name, &$param_val as &dyn rusqlite::ToSql)),+]))
                                 };
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
