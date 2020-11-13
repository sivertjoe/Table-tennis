use chrono::prelude::*;
use uuid::Uuid;
use rusqlite::{Connection, Result, NO_PARAMS, params, named_params};
use crate::user::User;
use elo::EloRank;
use crate::r#match::Match;
use crate::notification::{MatchNotificationTable, MatchNotification};

pub struct DataBase
{
    pub conn: Connection
}

const MATCH_NO_ANS: u8 = 0;
const ACCEPT_MATCH: u8 = 1;
const DECLINE_MATCH: u8 = 2;

const USER_ROLE_USER: u8 = 0;
const USER_ROLE_SUPERUSER: u8 = 1;


impl DataBase
{
    pub fn new(file: &str) -> Self
    {
        let conn = match Connection::open(file)
        {
            Err(e) => panic!(format!("Could not create connection, {}", e)),
            Ok(c) => c
        };


        conn.execute(&format!(
            "create table if not exists users (
                id              integer primary key autoincrement,
                name            VARCHAR(20) not null unique,
                elo             float  default 1500.0,
                password_hash   varchar(64) not null,
                uuid            varchar(36) not null,
                user_role       smallint default {}
                )", USER_ROLE_USER),
                NO_PARAMS,).expect("Creating user table");

        conn.execute(
             "create table if not exists matches (
                id              integer primary key autoincrement,
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
        self.migrate_user_role();
        self.migrate_password_and_uuid();
        self.migrate_match_id();
    }

    pub fn login(&self, name: String, password: String) -> Result<String> // String = Uuid
    {
        self.try_login(name, password)
    }

    pub fn respond_to_match(&self, id: i64, ans: u8, token: String) -> Result<usize>
    {
        self.try_respond_to_notification(id, ans, token)
    }

    pub fn create_user(&self, new_user: String, password: String) -> Result<String>
    {
        let uuid = format!("{}", Uuid::new_v4());
        self.conn.execute(
            "insert into users (name, password_hash, uuid) values (?1, ?2, ?3)",
            params![new_user, self.hash(&password), uuid],
            )?;
        Ok(uuid)
    }

    pub fn get_profile(&self, user: String) -> Result<User>
    {
        self.get_user(&user)
    }

    pub fn get_users(&self)  -> Result<Vec<User>>
    {
        Ok(self.get_all_users()?.collect())
    }

    pub fn register_match(&self, winner_name: String, loser_name: String, token: Option<String>) -> Result<usize>
    {
        let (winner, loser) = (self.get_user_without_matches(&winner_name)?, 
                               self.get_user_without_matches(&loser_name)?);
        let winner_elo = match self.get_newest_elo(winner.id) { Ok(elo) => elo, _ => winner.elo };
        let loser_elo = match self.get_newest_elo(loser.id) { Ok(elo) => elo, _ => loser.elo };
        let elo = EloRank { k: 32 };
        let (new_winner_elo, _) = elo.calculate(winner_elo, loser_elo);

        self.create_match_notification(&winner, &loser, new_winner_elo - winner.elo, token)
    }

    pub fn get_history(&self) -> Result<Vec<Match>>
    {
        self.get_all_matches()
    }

    pub fn change_password(&self, name: String, password: String, new_password: String) -> Result<usize>
    {
        self.try_change_password(name, password, new_password)
    }

    pub fn get_notifications(&self, user_token: String) -> Result<Vec<MatchNotification>>
    {
        self.try_get_notifications(user_token)
    }
}

// Migrate functions here (Will be deleted later on)
impl DataBase
{
    fn migrate_match_id(&self)
    {
        self.conn.execute(
            "alter table matches rename to old_matches",
             NO_PARAMS).expect("Change name");

        self.conn.execute(
             "create table if not exists matches (
                id              integer primary key autoincrement,
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
        self.conn.execute("insert into matches (epoch, elo_diff, winner_elo, loser_elo, winner, loser) 
                          select epoch, elo_diff, winner_elo, loser_elo, winner, loser from old_matches", NO_PARAMS).expect("Transfering");
        self.conn.execute("drop table old_matches", NO_PARAMS).expect("Transfering");
    }
    fn migrate_password_and_uuid(&self)
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

    fn migrate_user_role(&self)
    {
       self.conn.execute(
           &format!("alter table users
           add user_role smallint default {}", USER_ROLE_USER),
           NO_PARAMS,).expect("Adding user_role field");
    }
}

// Only private  functions here ~!
impl DataBase
{
    fn get_newest_elo(&self, id: i64) -> Result<f64>
    {
        let mut stmt = self.conn.prepare(
            "select winner, winner_elo, loser_elo from match_notification
            where winner = :id or loser = id
            order by epoch asc
            limit 1;")?;
        let elo = stmt.query_map_named(named_params!{":id": id}, |row|
        {
            let winner: i64 = row.get(0)?;
            let idx = if winner == id { 1 } else { 2 };
            let elo: f64 = row.get(idx)?;
            Ok(elo)
        })?.next();

        if elo.is_none() 
        {
            return Err(rusqlite::Error::InvalidParameterName(String::new()));
        }
        let elo = elo.unwrap();

        if elo.is_err()
        {
            return Err(rusqlite::Error::InvalidParameterName(String::new()));
        }

        Ok(elo.unwrap())
    }
    // Get all notifications for the users, exclude already 
    // answered ones
    fn try_get_notifications(&self, token: String) -> Result<Vec<MatchNotification>>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let mut stmt = self.conn.prepare(
            "select id, winner, loser, epoch from match_notification
            where 
                winner = :id and winner_accept = 0
            union
            select id, winner, loser, epoch from match_notification
            where 
                loser = :id and loser_accept = 0
            ")?;

        let notifications = stmt.query_map_named(named_params!{":id": user.id}, |row|
        {
            let winner_id: i64 = row.get(1)?;
            let loser_id: i64 = row.get(2)?;

            let winner_name: String = self.get_user_without_matches_by("id", "=", &winner_id.to_string())?.name;
            let loser_name: String = self.get_user_without_matches_by("id", "=", &loser_id.to_string())?.name;
            Ok(
                MatchNotification {
                    id: row.get(0)?,
                    winner: winner_name,
                    loser: loser_name,
                    epoch: row.get(3)?
                })
        })?;

        let mut vec: Vec<MatchNotification> = Vec::new();
        for n in notifications
        {
            if let Ok(mn) = n
            {
                vec.push(mn);
            };
        }
        vec.sort_by(|a, b| a.epoch.partial_cmp(&b.epoch).unwrap());
        Ok(vec)
    }
    fn try_respond_to_notification(&self, id: i64, ans: u8, token: String) -> Result<usize>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let mut stmt = self.conn.prepare(
            "select id, winner_accept, loser_accept, epoch, elo_diff, winner_elo, loser_elo,
            winner, loser from match_notification where id = :id")?;
        let mut match_notification = stmt.query_map_named(named_params!{":id": id}, |row|
        {
            Ok(
                MatchNotificationTable {
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

    fn handle_notification_answer(&self, user: &User, ans: u8, match_notification: &MatchNotificationTable) -> Result<usize>
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

    fn delete_match_notification(&self, m: &MatchNotificationTable) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("delete from match_notification where id = :id")?;
        stmt.execute_named(named_params!{":id": m.id})
    }

    fn create_match_from_notification(&self, m: &MatchNotificationTable) -> Result<usize>
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

    fn create_match_notification(&self, winner: &User, loser: &User, elo_diff: f64, token: Option<String>) -> Result<usize>
    {
        if let Some(token) = token
        {
            if self.user_have_token(winner.id, &token)?
            {
                return self.conn.execute(
                    "insert into match_notification (epoch, winner, loser, elo_diff, winner_elo, loser_elo, winner_accept) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![self.epoch(), winner.id, loser.id, elo_diff, winner.elo + elo_diff, loser.elo - elo_diff, ACCEPT_MATCH],);
            }
            else if self.user_have_token(loser.id, &token)?
            {
                return self.conn.execute(
                    "insert into match_notification (epoch, winner, loser, elo_diff, winner_elo, loser_elo, loser_accept) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![self.epoch(), winner.id, loser.id, elo_diff, winner.elo + elo_diff, loser.elo - elo_diff, ACCEPT_MATCH],);
            }
        }
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser, elo_diff, winner_elo, loser_elo) values (?1, ?2, ?3, ?4, ?5, ?6)",
                params![self.epoch(), winner.id, loser.id, elo_diff, winner.elo + elo_diff, loser.elo - elo_diff],)
    }

    fn user_have_token(&self, user_id: i64, token: &String) -> Result<bool>
    {
        let mut stmt = self.conn.prepare("select count(*) from users where id = :id and uuid = :token")?;
        let mut c = stmt.query_map_named(named_params!{":id": user_id, ":token": token}, |row|
        {
            let c: i64 = row.get(0)?;
            Ok(c)
        })?;

        Ok(c.next().unwrap().unwrap() == 1)
    }

    fn try_change_password(&self, name: String, password: String, new_password: String) -> Result<usize>
    {
        let mut stmt = self.conn.prepare("select id, password_hash from users where name = :name;")?;
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
        let mut stmt = self.conn.prepare("select password_hash, uuid from users where name = :name;")?;
        let info = stmt.query_map_named(named_params!{":name" : name}, |row|
        {
            let passwd: String = row.get(0)?;
            let uuid: String = row.get(1)?;
            Ok((passwd, uuid))
        })?.next();

        if info.is_none()
        {
                return Err(rusqlite::Error::InvalidParameterName("No user with that name exist".to_string()))
        }

        match info.unwrap()
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
        let mut stmt = self.conn.prepare("select id, name, elo, user_role from users;")?;
        let users = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                user_role: row.get(3)?,
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
        self.get_user_without_matches_by("name", "=", name.as_str())
    }

    fn get_user(&self, name: &String) -> Result<User>
    {
        let mut user = self.get_user_without_matches_by("name", "=", name.as_str())?;
        user.match_history = self.get_matches(user.id)?.collect();
        Ok(user)
    }

    fn  get_user_without_matches_by(&self, col: &str, comp: &str, val: &str) -> Result<User>
    {
        let mut stmt = self.conn.prepare(&format!("select id, name, elo, user_role from users where {} {} :val", col, comp))?;
        let mut users = stmt.query_map_named(named_params!{":val": val}, |row|
        {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                elo: row.get(2)?,
                user_role: row.get(3)?,
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


    fn respond_to_match(s: &DataBase, name: &str, id: i64)
    {
        let mut stmt = s.conn
                        .prepare("select uuid from users where name = :name")
                        .expect("Creating query");
        let token:  String = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            let token: String = row.get(0).expect("Getting first row");
            Ok(token)
        }).expect("Executing stmt").next().unwrap().unwrap();

        s.respond_to_match(id, ACCEPT_MATCH, token).expect("Responding true");
    }

    #[test]
    fn test_match_notification_both_accepted()
    {
        let db_file = "temp1.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner, loser, None).expect("Creating match");
        respond_to_match(&s, "Sivert", 1);
        respond_to_match(&s, "Lars", 1);

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

        let (sivert, lars) = (s.get_user_without_matches(&"Sivert".to_string()).unwrap(),
                              s.get_user_without_matches(&"Lars".to_string()).unwrap());
        assert!(sivert.elo > 1500.0);
        assert!(lars.elo < 1500.0);
    }

    fn get_token_from_user(s: &DataBase, name: &String) -> String
    {
        let mut stmt = s.conn.prepare("select uuid from users where name = :name").unwrap();
        let name = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            let name: String = row.get(0).unwrap();
            Ok(name)
        }).unwrap().next().unwrap().unwrap();
        name
    }

    #[test]
    fn test_match_registered_by_none_participant_gets_answered_no()
    {
        let db_file = "tempB.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");
        s.create_user("Bernt".to_string(), "password".to_string()).expect("Creating Bernt");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        let token = Some(get_token_from_user(&s, &"Bernt".to_string()));
        s.register_match(winner.clone(), loser.clone(), token.clone()).expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), token).expect("Creating match");


        let mut stmt = s.conn.prepare("select winner_accept from match_notification where id = 1").unwrap();
        let winner_accept = stmt.query_map(NO_PARAMS, |row|
        {
            let c: u8 = row.get(0).expect("Getting first (and only) row");
            Ok(c)
        }).unwrap().next().unwrap().unwrap();



        let mut stmt = s.conn.prepare("select loser_accept from match_notification where id = 2").unwrap();
        let loser_accept = stmt.query_map(NO_PARAMS, |row|
        {
            let c: u8 = row.get(0).expect("Getting first (and only) row");
            Ok(c)
        }).unwrap().next().unwrap().unwrap();


        std::fs::remove_file(db_file).expect("Removing file tempA");
        assert!(winner_accept == MATCH_NO_ANS);
        assert!(loser_accept == MATCH_NO_ANS);
    }

    #[test]
    fn test_match_registered_by_participant_gets_answered_yes()
    {
        let db_file = "tempA.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();

        let token1 = Some(get_token_from_user(&s, &winner));
        let token2 = Some(get_token_from_user(&s, &loser));
        s.register_match(winner.clone(), loser.clone(), token1).expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), token2).expect("Creating match");


        let mut stmt = s.conn.prepare("select winner_accept from match_notification where id = 1").unwrap();
        let winner_accept = stmt.query_map(NO_PARAMS, |row|
        {
            let c: u8 = row.get(0).expect("Getting first (and only) row");
            Ok(c)
        }).unwrap().next().unwrap().unwrap();



        let mut stmt = s.conn.prepare("select loser_accept from match_notification where id = 2").unwrap();
        let loser_accept = stmt.query_map(NO_PARAMS, |row|
        {
            let c: u8 = row.get(0).expect("Getting first (and only) row");
            Ok(c)
        }).unwrap().next().unwrap().unwrap();


        std::fs::remove_file(db_file).expect("Removing file tempA");
        assert!(winner_accept == ACCEPT_MATCH);
        assert!(loser_accept == ACCEPT_MATCH);
    }

    #[test]
    fn test_can_respond_to_match()
    {
        let db_file = "temp2.db";
        let s = DataBase::new(db_file);
        s.create_user("Sivert".to_string(), "password".to_string()).expect("Creating Sivert");
        s.create_user("Lars".to_string(), "password".to_string()).expect("Creating Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner, loser, None).expect("Creating match");
        respond_to_match(&s, "Sivert", 1);


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

        let winner =  "Sivert".to_string();
        let loser = "Lars".to_string();

        s.register_match(winner, loser, None).expect("Creating match");

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

        let user = s.get_user_without_matches_by("name", "=", "Sivert");
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
        s.change_password(name.clone(), password.clone(), new.clone()).expect("Changing password");

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

    fn get_match_noti_winner_elo(s: &Connection) -> Vec<f64>
    {
        let mut stmt =  s.prepare("select winner_elo from match_notification;").unwrap();
        let matches = stmt.query_map(NO_PARAMS, |row|
        {
            let win_elo: f64 = row.get(0).unwrap();
            Ok(win_elo)
        }).unwrap();

        let mut vec: Vec<f64> = Vec::new();
        for w in matches
        {
            if let Ok(elo) = w
            {
                vec.push(elo);
            }
        }
        vec
    }

    #[test]
    fn test_match_registration_elos_better()
    {
        let db_file = "tempD.db";
        let s = DataBase::new(db_file);

        let siv = "Sivert".to_string();
        let lars = "Lars".to_string();

        s.create_user(siv.clone(), "password".to_string()).expect("Creating Sivert");
        s.create_user(lars.clone(), "password".to_string()).expect("Creating Lars");

        let token_siv = Some(get_token_from_user(&s, &siv));


        use std::{thread, time};
        let five_millis = time::Duration::from_millis(5);


        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);
        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);

        let noti_winner_elos = get_match_noti_winner_elo(&s.conn);

        std::fs::remove_file(db_file).expect("Removing file temp");

        assert_ne!(noti_winner_elos[0], noti_winner_elos[1]);
    }

    #[test]
    fn test_rolls_back_correctly()
    {
        let db_file = "tempC.db";
        let s = DataBase::new(db_file);

        let siv = "Sivert".to_string();
        let lars = "Lars".to_string();
        let bernt = "Bernt".to_string();

        s.create_user(siv.clone(), "password".to_string()).expect("Creating Sivert");
        s.create_user(lars.clone(), "password".to_string()).expect("Creating Lars");
        s.create_user(bernt.clone(), "password".to_string()).expect("Creating Bernt");

        let token_siv = Some(get_token_from_user(&s, &siv));


        use std::{thread, time};
        let five_millis = time::Duration::from_millis(5);


        s.register_match(lars.clone(), siv.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);
        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);
        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);
        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        thread::sleep(five_millis);
        s.register_match(bernt.clone(), siv.clone(), token_siv.clone()).expect("Creating match");


        respond_to_match(&s, lars.as_str(), 2);
        respond_to_match(&s, bernt.as_str(), 5);
        respond_to_match(&s, lars.as_str(), 3);
        respond_to_match(&s, lars.as_str(), 1);
        respond_to_match(&s, lars.as_str(), 4);
        
        /*
         * Match Order:
         * W - L id  // winner - loser
         * L - S 1
         * S - L 2
         * S - L 3
         * S - L 4 
         * B - S 5
         *
         * But we will accept them in the order:
         * S - L 2
         * B - S 5
         * S - L 3
         * L - S 1
         * S - L 4
         * 1484, 1484, 1516   
         * 1516, 1484, 1516  
         *
         * Before roll_back the ELo's should be:
         * Sivert: 1512.016815723276
         * Lars  : 1471.2468774832018
         * Bernt : 1516.736306793522
         *
         * But in actuality they should be:
         * Sivert: 1514.2851406137202
         * Lars  : 1468.2570986091923
         * Bernt : 1517.4577607770875
         * A _little_ different, BUT! More correct, rollback should fix these
         */

        let old_matches = s.get_history().unwrap();
        s.roll_back().expect("Rolling back");
        let new_matches = s.get_history().unwrap();
        

        let (siv_user, lars_user, bernt_user) = (s.get_user_without_matches(&siv).unwrap(), 
                                                 s.get_user_without_matches(&lars).unwrap(),
                                                 s.get_user_without_matches(&bernt).unwrap());
        std::fs::remove_file(db_file).expect("Removing file tempC");

        assert_eq!(siv_user.elo, 1514.2851406137202);
        assert_eq!(lars_user.elo, 1468.2570986091923);
        assert_eq!(bernt_user.elo, 1517.4577607770875);
    }

    #[test]
    fn test_case_sensitive_user_search()
    {
        let db_file = "tempE.db";
        let s = DataBase::new(db_file);

        let Siv = "Sivert".to_string();
        let siv = "sivert".to_string();

        s.create_user(Siv.clone(), "password".to_string()).expect("Creating Sivert");
        s.create_user(siv.clone(), "password".to_string()).expect("Creating sivert");

        let user = s.get_user(&siv.clone()).unwrap();
        //std::fs::remove_file(db_file).expect("Removing file tempE");
        assert_eq!(user.name, siv);
    }
}
