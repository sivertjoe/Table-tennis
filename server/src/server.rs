use rusqlite::{Connection, Result, NO_PARAMS, params, named_params};
use crate::user::User;
use elo::EloRank;
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
                name VARCHAR(20) not null unique,
                elo float  default 1500.0
                )",
                NO_PARAMS,).expect("Failed to create database");

        conn.execute(
             "create table if not exists matches (
                epoch bigint not null,
                winner_elo_diff integer,
                loser_elo_diff integer,

                winner integer,
                loser integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
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

    pub fn get_profile(&self, user: String) -> Result<User>
    {
        self.get_user(&user)
    }

    pub fn get_users(&self)  -> Result<Vec<User>>
    {
        Ok(self.get_all_users()?.collect())
    }

    pub fn register_match(&self, m: Match) -> Result<usize>
    {
        let (winner, loser) = (self.get_user_without_matches(&m.winner)?, 
                                       self.get_user_without_matches(&m.loser)?);
        let elo = EloRank { k: 32 };
        let (new_winner_elo, new_loser_elo) = elo.calculate(winner.elo, loser.elo);
    
        self.update_elo(winner.id, new_winner_elo)?;
        self.update_elo(loser.id, new_loser_elo)?;
        self.create_match(m.epoch, &winner, &loser, new_winner_elo - winner.elo,  new_loser_elo - loser.elo)?;
        Ok(0)
    }
}


// Only private  functions here ~!
impl DataBase
{
    fn create_match(&self, epoch: i64, winner: &User, loser: &User, winner_elo_diff: f64, loser_elo_diff: f64) -> Result<usize>
    {
        self.conn.execute(
            "insert into matches (epoch, winner, loser, winner_elo_diff, loser_elo_diff) values (?1, ?2, ?3, ?4, ?5)",
            params![epoch, winner.id, loser.id, winner_elo_diff, loser_elo_diff],)
    }

    fn update_elo(&self, id: i64, elo: f64) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("update users  set elo = :elo WHERE id = :id")?;
        stmt.execute_named(named_params!{":elo": elo, ":id": id})
    }

    fn get_matches(&self, id: i64) -> Result<impl Iterator<Item=Match>>
    {
        let s = "select a.name, b.name, winner_elo_diff, loser_elo_diff, epoch
                from matches 
                inner join users as a on a.id = winner
                inner join users as b on b.id = loser
                where winner = :id or loser = :id";
        let mut stmt = self.conn.prepare(s)?;
        let matches = stmt.query_map_named(named_params!{":id" : id}, |row|
        {
            Ok(Match {
                winner: row.get(0)?,
                loser: row.get(1)?,
                winner_elo_diff: row.get(2)?,
                loser_elo_diff: row.get(3)?,
                epoch: row.get(4)?,
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
        vec.sort_by(|a, b| b.epoch.partial_cmp(&a.epoch).unwrap());
        Ok(vec.into_iter())
    }


    fn get_all_users(&self) -> Result<impl Iterator<Item=User>>
    {
        let mut stmt = self.conn.prepare("select * from users;")?;
        let users = stmt.query_map(NO_PARAMS, |row|
        {
            let name: String = row.get(1)?;
            Ok(User {
                id: row.get(0)?,
                name: name,
                elo: row.get(2)?,
                match_history: Vec::new()
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
        vec.sort_by(|a, b| b.elo.partial_cmp(&a.elo).unwrap());
        Ok(vec.into_iter())
    }

    fn get_user_without_matches(&self, name: &String) -> Result<User>
    {
        let mut stmt = self.conn.prepare("select * from users where name like :name")?;
        let mut users = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                match_history: Vec::new(),
            })
    })?;

        match users.next()
        {
            Some(Ok(user)) => 
            {
                Ok(user)
            },
            Some(Err(e)) => Err(e),
            None => Err(rusqlite::Error::InvalidParameterName("User did not exist".to_string()))
        }
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

        match users.next()
        {
            Some(Ok(mut user)) => 
            {
                user.match_history = self.get_matches(user.id)?.collect();
                Ok(user)
            },
            Some(Err(e)) => Err(e),
            None => Err(rusqlite::Error::InvalidParameterName("User did not exist".to_string()))
        }
    }
}
