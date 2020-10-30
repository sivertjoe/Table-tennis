use rusqlite::{Connection, Result, NO_PARAMS, params, named_params};
use crate::user::User;
use elo::EloRank;
use itertools::join;
use crate::r#match::Match;





const DATABASE_FILE: &'static str = "db.db";

pub struct DataBase
{
    conn: Connection
}


impl DataBase
{
    pub fn new() -> Self
    {
        let conn = match Connection::open(DATABASE_FILE)
        {
            Err(e) => panic!(format!("Could not create connection, {}", e)),
            Ok(c) => c
        };

        conn.execute(
            "create table if not exists users (
                id integer primary key autoincrement,
                name VARCHAR(20) not null,
                elo float  default 1500.0
                )",
                NO_PARAMS,).expect("Failed to create database");

        conn.execute(
             "create table if not exists matches (
                winner VARCHAR(20) not null references users(name),
                loser VARCHAR(20) not null references users(name)
                )",
                NO_PARAMS,).expect("Failed to create database");

        DataBase {
            conn: conn
        }
    }

    pub fn create_user(&self, new_user: String) -> Result<usize>
    {
        self.conn.execute(
            "insert into users (name) values (?1)",
            params![new_user],
            )
    }

    pub fn get_profile(&self, user: String) -> Result<String>
    {
        self.get_user(&user)
            .map(|u| u.to_string())
    }

    pub fn get_users(&self)  -> Result<String>
    {
        let users = self.get_all_users()?;
        Ok(format!("{}\n", join(users.map(|u| u.name), "\n")))
    }

    pub fn register_match(&self, winner: String, loser: String) -> Result<usize>
    {
        let (winner_elo, loser_elo) = self.get_elo_scores(&winner, &loser)?;
        let elo = EloRank { k: 32 };
        let (new_winner_elo, new_loser_elo) = elo.calculate(winner_elo, loser_elo);
    
        self.update_elo(&winner, new_winner_elo)?;
        self.update_elo(&loser, new_loser_elo)?;
        self.create_match(&winner, &loser)?;
        Ok(0)
    }
}


// Only private  functions here ~!
impl DataBase
{
    fn create_match(&self, winner: &String, loser: &String) -> Result<usize>
    {
        self.conn.execute(
            "insert into matches (winner, loser) values (?1, ?2)",
            params![winner, loser],)
    }

    fn update_elo(&self, name: &String, elo: f64) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("update users  set elo = :elo WHERE name like :name")?;
        stmt.execute_named(named_params!{":elo": elo, ":name": name})
    }

    fn get_matches(&self, name: &String) -> Result<impl Iterator<Item=Match>>
    {
        let mut stmt = self.conn.prepare("select * from matches where winner like :name or loser like  :name;")?;
        let matches = stmt.query_map_named(named_params!{":name" : name}, |row|
        {
            Ok(Match {
                winner: row.get(0)?,
                loser: row.get(1)?,
            })
        })?;

        let mut vec = Vec::new();
        for m in matches
        {
            if let Ok(u) = m
            {
                vec.push(u);
            };
        }
        Ok(vec.into_iter())
    }


    fn get_all_users(&self) -> Result<impl Iterator<Item=User>>
    {
        let mut stmt = self.conn.prepare("select * from users;")?;
        let users = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                match_history: vec![]
            })
        })?;

        let mut vec = Vec::new();
        for user in users
        {
            if let Ok(u) = user
            {
                vec.push(u);
            };
        }
        Ok(vec.into_iter())
    }

    fn get_user(&self, name: &String) -> Result<User>
    {
        let mut stmt = self.conn.prepare("select * from users where name like :name")?;
        let mut users = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                match_history: vec![]
            })
        })?;
        // @TODO: Get match history

        match users.next()
        {
            Some(Ok(mut user)) => 
            {
                user.match_history = self.get_matches(&user.name)?.collect();
                Ok(user)
            },
            Some(Err(e)) => Err(e),
            None => panic!("something went wrong")
        }
    }

    fn get_elo_scores(&self, name1: &String, name2: &String) -> Result<(f64, f64)>
    {
        let score1 = self.get_user(name1)?.elo;
        let score2 = self.get_user(name2)?.elo;

        Ok((score1, score2))
    }
}
