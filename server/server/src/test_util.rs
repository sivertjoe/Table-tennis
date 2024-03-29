// These tags are kind of ungly, but idk how else to do it

#[cfg(test)] use rusqlite::{named_params, params, NO_PARAMS};
#[cfg(test)]
use server_core::constants::{USER_ROLE_REGULAR, USER_ROLE_SOFT_INACTIVE, USER_ROLE_SUPERUSER};
#[cfg(test)] use server_core::{constants::*, types::*};
#[cfg(test)] use uuid::Uuid;

#[cfg(test)] use super::DataBase;


#[cfg(test)]
pub fn get_table_size(s: &DataBase, table: &str) -> i64
{
    s.conn
        .prepare(&format!("select count(*) from {}", table))
        .unwrap()
        .query_map(NO_PARAMS, |row| row.get::<_, i64>(0))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
}

#[cfg(test)]
pub fn create_user(s: &DataBase, name: &str) -> String
{
    let uuid = format!("{}", Uuid::new_v4());
    s.conn
        .execute(
            "insert into users (name, password_hash, uuid, user_role) values (?1, ?2, ?3, ?4)",
            params![
                name,
                hash(&"password".to_string()),
                uuid,
                USER_ROLE_REGULAR | USER_ROLE_SOFT_INACTIVE,
            ],
        )
        .unwrap();
    uuid
}

#[cfg(test)]
pub fn hash(word: &String) -> String
{
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(word);
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
pub fn respond_to_match(s: &DataBase, name: &str, id: i64)
{
    let mut stmt = s
        .conn
        .prepare("select uuid from users where name = :name")
        .expect("Creating query");
    let token: String = stmt
        .query_map_named(named_params! {":name": name}, |row| {
            let token: String = row.get(0).expect("Getting first row");
            Ok(token)
        })
        .expect("Executing stmt")
        .next()
        .unwrap()
        .unwrap();

    s.respond_to_match(id, ACCEPT_REQUEST, token).expect("Responding true");
}

#[cfg(test)]
pub fn make_user_admin(s: &DataBase, name: String) -> ServerResult<usize>
{
    s.conn.execute(
        &format!(
            "update users
            set user_role = {}
            where name = \"{}\"",
            USER_ROLE_SUPERUSER, name
        ),
        NO_PARAMS,
    )?;
    Ok(0)
}

#[cfg(test)]
pub fn create_season(s: &DataBase, id: i64, time: i64) -> ServerResult<usize>
{
    s.conn
        .execute("insert into seasons (id, start_epoch) values (?1, ?2)", params![id, time])?;
    Ok(0)
}

#[cfg(test)]
pub fn create_tournament_image(s: &DataBase)
{
    s.conn.execute("insert into images (name) values (?1)", params![format!("3.png")]).unwrap();
}
