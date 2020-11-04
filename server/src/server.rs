use uuid::Uuid;
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
                elo float  default 1500.0,
                password_hash varchar(64) not null,
                uuid varchar(36) not null
                )",
                NO_PARAMS,).expect("Failed to create database");

        conn.execute(
             "create table if not exists matches (
                epoch bigint not null,
                elo_diff integer,
                winner_elo float,
                loser_elo float,
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
    pub fn migrate(&self)
    {
        let default_password = DataBase::get_default_password();
        self.conn.execute(
            &format!("alter table users
                add password_hash varchar(64) default \"{}\"",
                default_password),
                NO_PARAMS,).expect("Adding password_hash field");

        self.conn.execute(
            "alter table users
                add uuid varchar(36)",
                NO_PARAMS,).expect("Adding uuid field");


        let users = self.get_all_users().expect("Getting all users");
        let mut stmt = self.conn.prepare("update users  set uuid = :uuid WHERE id = :id").expect("creating statement");
        for user in users
        {
            let uuid = format!("{}", Uuid::new_v4());
            stmt.execute_named(named_params!{":uuid": uuid, ":id": user.id}).expect("Adding user uuid");
        }
    }

    pub fn login(&self, name: String, password: String) -> Result<String> // String = Uuid
    {
        self.try_login(name, password)
    }

    pub fn create_user(&self, new_user: String, password: String) -> Result<usize>
    {
        self.conn.execute(
            "insert into users (name, password_hash, uuid) values (?1, ?2, ?3)",
            params![new_user, self.hash(&password), format!("{}", Uuid::new_v4())],
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
        self.create_match(m.epoch, &winner, &loser, new_winner_elo - winner.elo)?;
        Ok(0)
    }

    pub fn get_history(&self) -> Result<Vec<Match>>
    {
        self.get_all_matches()
    }

    pub fn change_password(&self, name: String, password: String, new_password: String) -> Result<usize>
    {
        self.try_change_password(name, password, new_password)
    }
}


// Only private  functions here ~!
impl DataBase
{
    fn try_change_password(&self, name: String, password: String, new_password: String) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("select id, password_hash from users where name like :name;")?;
        let info = stmt.query_map_named(named_params!{":name" : name}, |row|
        {
            let id: i64 = row.get(0)?;
            let passwd: String = row.get(1)?;
            Ok((id, passwd))
        })?.next().expect("getting user");

        match info
        {
            Ok((id, p)) =>
            {
                if self.hash(&password) == p
                {
                    return self.update_password(id, new_password);
                }

                return Err(rusqlite::Error::InvalidParameterName("Password did not match".to_string()))
            },
            _ => Err(rusqlite::Error::InvalidParameterName("Error finding user..".to_string()))
        }
    }

    fn update_password(&self, id: i64, new_password: String) -> Result<usize>
    {
        let hash = self.hash(&new_password);
        let mut stmt = self.conn.prepare("update users  set password_hash = :hash WHERE id = :id")?;
        stmt.execute_named(named_params!{":hash": hash, ":id": id})
    }

    fn try_login(&self, name: String, password: String) -> Result<String>
    {
        let mut stmt = self.conn.prepare("select password_hash, uuid from users where name like :name;")?;
        let info = stmt.query_map_named(named_params!{":name" : name}, |row|
        {
            let passwd: String = row.get(0)?;
            let uuid: String = row.get(1)?;
            Ok((passwd, uuid))
        })?.next().expect("getting user");

        match info
        {
            Ok((p, u)) =>
            {
                if self.hash(&password) == p
                {
                    return Ok(u);
                }

                return Err(rusqlite::Error::InvalidParameterName("Password did not match".to_string()))
            },
            _ => Err(rusqlite::Error::InvalidParameterName("Error finding user..".to_string()))
        }

    }

    fn hash(&self, word: &String) -> String
    {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(word);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    fn get_all_matches(&self) -> Result<Vec<Match>>
    {
        let mut stmt 
          = self.conn.prepare("select a.name, b.name, elo_diff, winner_elo, loser_elo, epoch from matches
                               inner join users as a on a.id = winner
                               inner join users as b on b.id = loser;")?;
        let matches = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(Match {
                winner: row.get(0)?,
                loser: row.get(1)?,
                elo_diff: row.get(2)?,
                winner_elo: row.get(3)?,
                loser_elo: row.get(4)?,
                epoch: row.get(5)?,

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
        Ok(vec)

    }
    fn create_match(&self, epoch: i64, winner: &User, loser: &User, elo_diff: f64) -> Result<usize>
    {
        self.conn.execute(
            "insert into matches (epoch, winner, loser, elo_diff, winner_elo, loser_elo) values (?1, ?2, ?3, ?4, ?5, ?6)",
            params![epoch, winner.id, loser.id, elo_diff, winner.elo + elo_diff, loser.elo - elo_diff],)
    }

    fn update_elo(&self, id: i64, elo: f64) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("update users  set elo = :elo WHERE id = :id")?;
        stmt.execute_named(named_params!{":elo": elo, ":id": id})
    }

    fn get_matches(&self, id: i64) -> Result<impl Iterator<Item=Match>>
    {
        let s = "select a.name, b.name, elo_diff, winner_elo, loser_elo, epoch
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
                elo_diff: row.get(2)?,
                winner_elo: row.get(3)?,
                loser_elo: row.get(4)?,
                epoch: row.get(5)?,
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
        let mut stmt = self.conn.prepare("select id, name, elo from users;")?;
        let users = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
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
        let mut stmt = self.conn.prepare("select id, name, elo from users where name like :name")?;
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
        let mut stmt = self.conn.prepare("select id, name, elo from users where name like :name")?;
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

    fn get_default_password() -> String
    {
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;
        use sha2::{Sha256, Digest};

        let mut buffer = String::new();
        let file = File::open("default_password.txt").expect("Opening default password file");
        let mut reader = BufReader::new(file);
        reader.read_line(&mut buffer).expect("Reading line");

        let mut hasher = Sha256::new();
        hasher.update(&buffer.trim_end_matches("\n"));
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
