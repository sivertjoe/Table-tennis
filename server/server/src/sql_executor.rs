use crate::server::DataBase;

fn get_row_value<'stmt>(row: &rusqlite::Row<'stmt>, index: usize) -> rusqlite::Result<String>
{
    use rusqlite::types::ValueRef::*;
    let res = row.get_raw_checked(index)?;
    Ok(match res
    {
        Null => "NULL".to_string(),
        Integer(i) => i.to_string(),
        Real(f) => f.to_string(),
        Text(s) => std::str::from_utf8(&s).unwrap().to_string(),
        Blob(b) => std::str::from_utf8(&b).unwrap().to_string(),
    })
}

impl DataBase
{
    pub fn execute_sql(&self, string: String) -> rusqlite::Result<String>
    {
        use rusqlite::NO_PARAMS;

        let mut stmt = self.conn.prepare(&string)?;

        let col_count = stmt.column_count();
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut output = Vec::new();
        while let Some(row) = rows.next()?
        {
            let mut row_vec = Vec::new();
            for i in 0..col_count
            {
                row_vec.push(get_row_value(row, i)?);
            }
            let s = row_vec.join(", ");
            output.push(s)
        }

        Ok(output.join("\n"))
    }
}
