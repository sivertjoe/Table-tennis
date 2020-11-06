use chrono::prelude::*;
use uuid::Uuid;
use rusqlite::{Connection, Result, NO_PARAMS, params, named_params};
use crate::user::User;
use elo::EloRank;
use crate::r#match::Match;

struct MatchNotification
{
    id: i64,
    winner_accept: u8,
    loser_accept: u8, 
    epoch: i64,
    elo_diff: f64,
    winner_elo: f64,
    loser_elo: f64,
    winner: i64,
    loser: i64
}

pub struct DataBase
{
    pub conn: Connection
}

const ACCEPT_MATCH: u8 = 1;
const DECLINE_MATCH: u8 = 2;


impl DataBase
{
    pub fn new(file: &str) -> Self
    {
        let conn = match Connection::open(file)
        {
            Err(e) => panic!(format!("Could not create connection, {}", e)),
            Ok(c) => c
        };


        conn.execute(
            "create table if not exists users (
                id              integer primary key autoincrement,
                name            VARCHAR(20) not null unique,
                elo             float  default 1500.0,
                password_hash   varchar(64) not null,
                uuid            varchar(36) not null
                )",
                NO_PARAMS,).expect("Creating user table");

        conn.execute(
             "create table if not exists matches (
                epoch           bigint not null,
                elo_diff        integer,
                winner_elo      float,
                loser_elo       float,
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
                NO_PARAMS,).expect("Creating matches table");

        conn.execute(
             "create table if not exists match_notification (
                id              integer primary key autoincrement,
                winner_accept   smallint default 0,
                loser_accept    smallint default 0,
                epoch           bigint not null,
                elo_diff        integer,
                winner_elo      float,
                loser_elo       float,
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
                NO_PARAMS,).expect("Creating match_notification table");
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

    pub fn respond_to_match(&self, id: i64, ans: u8, token: String) -> Result<usize>
    {
        self.try_respond_to_notification(id, ans, token)
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
        let (new_winner_elo, _) = elo.calculate(winner.elo, loser.elo);

        self.create_match_notification(&winner, &loser, new_winner_elo - winner.elo)
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
    fn try_respond_to_notification(&self, id: i64, ans: u8, token: String) -> Result<usize>
    {
        let user = self.get_user_without_matches_by("uuid", "like", token.as_str())?;
        let mut stmt = self.conn.prepare(
            "select id, winner_accept, loser_accept, epoch, elo_diff, winner_elo, loser_elo,
            winner, loser from match_notification where id = :id")?;
        let mut match_notification = stmt.query_map_named(named_params!{":id": id}, |row|
        {
            Ok(
                MatchNotification {
                    id: row.get(0)?,
                    winner_accept: row.get(1)?,
                    loser_accept: row.get(2)?,
                    epoch: row.get(3)?,
                    elo_diff: row.get(4)?,
                    winner_elo: row.get(5)?,
                    loser_elo: row.get(6)?,
                    winner: row.get(7)?,
                    loser: row.get(8)?
                })
        })?.next().expect("Unwrapping element").expect("Unwrapping Option");

        if user.id != match_notification.winner && user.id != match_notification.loser
        {
            return Err(rusqlite::Error::InvalidParameterName("User can't accept this match".to_string()));
        }
        
        if user.id == match_notification.winner
        {
            match_notification.winner_accept = ans;
        }
        else
        {
            match_notification.loser_accept = ans;
        }
        self.handle_notification_answer(&user, ans, &match_notification)
    }

    fn handle_notification_answer(&self, user: &User, ans: u8, match_notification: &MatchNotification) -> Result<usize>
    {
        // If both accepted, we need to 
        //  * create the match notification,
        //  * delet the match notification
        //  * update both elos
        //
        //  Here alternativly, we can delete the notification if the other'
        //  person have already responded
        if match_notification.loser_accept == ACCEPT_MATCH 
            && match_notification.winner_accept == ACCEPT_MATCH
        {
            self.create_match_from_notification(&match_notification)?;
            self.delete_match_notification(&match_notification)?;
            self.update_elo(match_notification.winner, match_notification.winner_elo)?;
            self.update_elo(match_notification.loser, match_notification.loser_elo)?;
            return Ok(0);
        }
        else
        {
            let col = if user.id == match_notification.winner { "winner_accept" } else { "loser_accept" };
            self.update_match_notification_answer(col, ans, match_notification.id)
        }
    }

    fn update_match_notification_answer(&self, col: &str, ans: u8, id: i64) -> Result<usize>
    {
        let mut stmt = self.conn.prepare(&format!("update match_notification set {} = :ans where id = :id", col))?;
        stmt.execute_named(named_params!{":id": id, ":ans": ans})
    }

    fn delete_match_notification(&self, m: &MatchNotification) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("delete from match_notification where id = :id")?;
        stmt.execute_named(named_params!{":id": m.id})
    }

    fn create_match_from_notification(&self, m: &MatchNotification) -> Result<usize>
    {
            self.conn.execute(
                "insert into matches (epoch, winner, loser, elo_diff, winner_elo, loser_elo) 
                values (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    m.epoch, 
                    m.winner,
                    m.loser,
                    m.elo_diff, 
                    m.winner_elo,
                    m.loser_elo],)
    }

    fn create_match_notification(&self, winner: &User, loser: &User, elo_diff: f64) -> Result<usize>
    {
        self.conn.execute(
            "insert into match_notification (epoch, winner, loser, elo_diff, winner_elo, loser_elo) values (?1, ?2, ?3, ?4, ?5, ?6)",
            params![self.epoch(), winner.id, loser.id, elo_diff, winner.elo + elo_diff, loser.elo - elo_diff],)
    }

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
        self.get_user_without_matches_by("name", "like", name.as_str())
    }

    fn get_user(&self, name: &String) -> Result<User>
    {
        let mut user = self.get_user_without_matches_by("name", "like", name.as_str())?;
        user.match_history = self.get_matches(user.id)?.collect();
        Ok(user)
    }

    fn  get_user_without_matches_by(&self, col: &str, comp: &str, val: &str) -> Result<User>
    {
        let mut stmt = self.conn.prepare(&format!("select id, name, elo from users where {} {} :val", col, comp))?;
        let mut users = stmt.query_map_named(named_params!{":val": val}, |row|
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

    fn epoch(&self) -> i64
    {
        Utc::now().timestamp_millis()
    }
}



#[cfg(test)]
mod test
{
    use super::*;
    use rusqlite::{NO_PARAMS};


    fn respond_to_match(s: &DataBase, name: &str)
    {
        let mut stmt = s.conn
                        .prepare("select uuid from users where name like :name")
                        .expect("Creating query");
        let token:  String = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            let token: String = row.get(0).expect("Getting first row");
            Ok(token)
        }).expect("Executing stmt").next().unwrap().unwrap();

        // Kind of hacky, in this case I know the ID of the match_notification
        // will be 1
        s.respond_to_match(1, ACCEPT_MATCH, token).expect("Responding true");
    }

    #[test]
    fn test_match_notification_both_accepted()
    {
        let db_file = "temp1.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let m = Match { 
            winner: "Sivert".to_string(),
            loser: "Lars".to_string(),
            epoch: 0,
            elo_diff: 0.,
            winner_elo: 0.,
            loser_elo: 0.
        };


        s.register_match(m).expect("Creating match");
        respond_to_match(&s, "Sivert");
        respond_to_match(&s, "Lars");

        let mut stmt = s.conn.prepare("select * from match_notification")
                             .expect("creating statement");

        let find  = stmt.query_map(NO_PARAMS, |row| 
        {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        assert!(find.expect("unwrapping quey map").next().is_none());

        let mut stmn = s.conn.prepare("select COUNT(*) from matches")
                             .expect("creating statement");

        let find  = stmn.query_map(NO_PARAMS, |row| 
        {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp2");
        assert!(find.unwrap()
                    .next()
                    .unwrap() 
                    .unwrap() == 1);
    }

    #[test]
    fn test_can_respond_to_match()
    {
        let db_file = "temp2.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let m = Match { 
            winner: "Sivert".to_string(),
            loser: "Lars".to_string(),
            epoch: 0,
            elo_diff: 0.,
            winner_elo: 0.,
            loser_elo: 0.
        };


        s.register_match(m).expect("Creating match");
        respond_to_match(&s, "Sivert");


        let mut stmt = s.conn
                    .prepare("select * from match_notification")
                    .expect("creating statement");

        let find  = stmt.query_map(NO_PARAMS, |row| 
        {
            let c: i64 = row.get(1).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp2");
        assert!(
                find.unwrap()
                    .next()
                    .unwrap() 
                    .unwrap()
                == 1);
    }


    #[test]
    fn test_can_register_match()
    {
        let db_file = "temp3.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let m = Match { 
            winner: "Sivert".to_string(),
            loser: "Lars".to_string(),
            epoch: 0,
            elo_diff: 0.,
            winner_elo: 0.,
            loser_elo: 0.
        };

        s.register_match(m).expect("Creating match");

        let mut stmn = s.conn.prepare("select COUNT(*) from match_notification")
                             .expect("creating statement");

        let find  = stmn.query_map(NO_PARAMS, |row| 
        {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(find.unwrap()
                    .next()
                    .unwrap() 
                    .unwrap() == 1);
    }

    #[test]
    fn test_can_get_user_by_name()
    {
        let db_file = "temp4.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivertt".to_string(), "password".to_string()).expect("Creating Sivertt");
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");

        let user = s.get_user_without_matches_by("name", "like", "Sivert");
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(user.is_ok() && user.unwrap().name == "Sivert");
    }

    #[test]
    fn test_can_get_user_by_id()
    {
        let db_file = "temp5.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");

        let user = s.get_user_without_matches_by("id", "=", "1");
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(user.is_ok() && user.unwrap().name == "Sivert");
    }
    
    #[test]
    fn test_login_returns_uuid()
    {
        let db_file = "temp6.db";
        let s = DataBase::new(db_file);
        let (name, password) = ("Sivert".to_string(), "password".to_string());
        s.create_user(name.clone(), password.clone()).expect("Creating Sivert");

        let uuid = s.login(name, password);
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(uuid.is_ok(), uuid.unwrap().len() == 36);
    }

    #[test]
    fn test_login_cant_log_in_with_wrong_password()
    {
        let db_file = "temp7.db";
        let s = DataBase::new(db_file);
        let (name, password) = ("Sivert".to_string(), "password".to_string());
        s.create_user(name.clone(), password.clone()).expect("Creating Sivert");

        let uuid = s.login(name, "Not correct password".to_string());
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(uuid.is_err());
    }

    #[test]
    fn test_can_change_password()
    {
        let db_file = "temp8.db";
        let s = DataBase::new(db_file);
        let (name, password, new) = ("Sivert".to_string(), "password".to_string(), "new".to_string());
        s.create_user(name.clone(), password.clone()).expect("Creating Sivert");
        s.change_password(name.clone(), password.clone(), new.clone());

        let err = s.login(name.clone(), password);
        let uuid = s.login(name, new);

        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(err.is_err() && (uuid.is_ok() && uuid.unwrap().len() == 36));
    }

    #[test]
    fn test_unix_time_in_ms()
    {
        let db_file = "temp9.db";
        let s = DataBase::new(db_file);

        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(s.epoch().to_string().len() == 13);
    }
}
