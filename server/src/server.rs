use std::str::FromStr;
use chrono::prelude::*;
use uuid::Uuid;
use rusqlite::{Connection, NO_PARAMS, params, named_params};
use crate::user::{User, EditUserAction, USER_ROLE_REGULAR, USER_ROLE_SUPERUSER, USER_ROLE_INACTIVE};
use elo::EloRank;
use crate::r#match::Match;
use crate::notification::{MatchNotificationTable, MatchNotification, NewUserNotification, NewUserNotificationAns};

use std::convert::From;

pub struct DataBase
{
    pub conn: Connection
}
#[allow(dead_code)]
const MATCH_NO_ANS: u8 = 0;
const ACCEPT_MATCH: u8 = 1;
#[allow(dead_code)]
const DECLINE_MATCH: u8 = 2;

pub type ServerResult<T> = rusqlite::Result<T, ServerError>;

#[repr(u8)]
#[derive(Debug)]
pub enum ServerError
{
    UserNotExist,
    UsernameTaken,
    PasswordNotMatch,
    Unauthorized,
    WaitingForAdmin,
    Critical(String),
    Rusqlite(rusqlite::Error),
}

impl From<rusqlite::Error> for ServerError
{
    fn from(error: rusqlite::Error) -> Self
    {
        Self::Rusqlite(error)
    }
}

impl From<ServerError> for rusqlite::Error
{
    fn from(error: ServerError) -> Self
    {
        match error
        {
            ServerError::Rusqlite(e) => e,
            _ => unreachable!() 
        }
    }
}

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
                )", USER_ROLE_REGULAR),
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
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
                NO_PARAMS,).expect("Creating match_notification table");

        conn.execute(
            "create table if not exists new_user_notification (
                id              integer primary key autoincrement,
                name            VARCHAR(20) not null unique,
                password_hash   varchar(64) not null
            )",
            NO_PARAMS,).expect("Creating new_user_notification");
        DataBase {
            conn: conn
        }
    }

    pub fn get_is_admin(&self, token: String) -> ServerResult<bool>
    {
        let user = self.get_user_without_matches_by("uuid", "=", &token)?;
        // TODO: Implement & for u8
        Ok(user.user_role & USER_ROLE_SUPERUSER == USER_ROLE_SUPERUSER )
    }

    pub fn login(&self, name: String, password: String) -> ServerResult<String> // String = Uuid
    {
        self.try_login(name, password)
    }

    pub fn respond_to_match(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        self.try_respond_to_notification(id, ans, token)
    }

    pub fn create_user(&self, new_user: String, password: String) -> ServerResult<()>
    {
        self.create_new_user_notification(new_user, password)
    }

    pub fn edit_users(&self, users: Vec<String>, action: String, token: String) -> ServerResult<usize>
    {
        self._edit_users(users, action, token)
    }

    pub fn get_profile(&self, user: String) -> ServerResult<User>
    {
        self.get_user(&user)
    }

    pub fn get_all_users(&self, token: String) -> ServerResult<Vec<User>>
    {
        Ok(self._get_all_users(token)?)
    }

    pub fn get_users(&self) -> ServerResult<Vec<User>>
    {
        Ok(self.get_users()?)
    }

    pub fn register_match(&self, winner_name: String, loser_name: String, token: String) -> ServerResult<()>
    {
        if self.get_user_without_matches_by("uuid", "=", &token).is_err()
        {
            return Err(ServerError::UserNotExist);
        };

        let (winner, loser) = (self.get_user_without_matches(&winner_name)?,
                               self.get_user_without_matches(&loser_name)?);

        self.create_match_notification(&winner, &loser, token)
    }

    pub fn get_history(&self) -> ServerResult<Vec<Match>>
    {
        self.get_all_matches()
    }

    pub fn change_password(&self, name: String, password: String, new_password: String) -> ServerResult<()>
    {
        self.try_change_password(name, password, new_password)
    }

    pub fn get_notifications(&self, user_token: String) -> ServerResult<Vec<MatchNotification>>
    {
        self.try_get_notifications(user_token)
    }

    pub fn get_new_user_notifications(&self, token: String) -> ServerResult<Vec<NewUserNotification>>
    {
        self.try_get_new_user_notifications(token)
    }

    pub fn respond_to_new_user(&self, not: NewUserNotificationAns) -> ServerResult<()>
    {
        if !self.get_is_admin(not.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        // No match is being accepted, but the ans values are the same Xdd
        if not.ans == ACCEPT_MATCH
        {
            self.create_user_from_notification(not.id)?;
        }
        self.delete_new_user_notification(not.id)
    }
}


// Only private  functions here ~!
impl DataBase
{
    fn create_new_user_notification(&self, name: String, password: String) -> ServerResult<()>
    {
        if !self.check_unique_name(&name)?
        {
            return Err(ServerError::UsernameTaken);
        }

        self.conn.execute(
            "insert into new_user_notification (name, password_hash) values (?1, ?2)",
            params![name, self.hash(&password)],
            )?;
        Ok(())
    }

    fn make_user_admin(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let role = user.user_role | USER_ROLE_SUPERUSER;
        self.conn.execute(
            &format!("update users
                set user_role = {}
                where name = \"{}\"", role, name),
                NO_PARAMS,)?;
        Ok(0)
    }

    fn make_user_regular(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let role = user.user_role & !USER_ROLE_SUPERUSER;
        self.conn.execute(
            &format!("update users
                set user_role = {}
                where name = \"{}\"", role, name),
                NO_PARAMS,)?;
        Ok(0)
    }

    fn make_user_active(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let role = user.user_role & !USER_ROLE_INACTIVE;
        let mut stmt = self.conn.prepare("update users set user_role = :role where name = :name")?;
        stmt.execute_named(named_params!{":role": role, ":name": name})?;
        Ok(0)
    }

    fn make_user_inactive(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let role = user.user_role | USER_ROLE_INACTIVE;
        let mut stmt = self.conn.prepare("update users set user_role = :role where name = :name")?;
        stmt.execute_named(named_params!{":role": role, ":name": name})?;
        Ok(0)
    }

    fn _edit_users(&self, users: Vec<String>, action: String, token: String) -> ServerResult<usize>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

        let action = match EditUserAction::from_str(&action)
        {
            Ok(a) => a,
            _ => return Err(ServerError::Critical("Invalid action".into())),
        };

        let func: Box<dyn Fn(String) -> ServerResult<usize>> = match action
        {
            EditUserAction::MakeUserActive => Box::new(|name: String| self.make_user_active(name)),
            EditUserAction::MakeUserRegular => Box::new(|name: String| self.make_user_regular(name)),
            EditUserAction::MakeUserInactive => Box::new(|name: String| self.make_user_inactive(name)),
            EditUserAction::MakeUserSuperuser => Box::new(|name: String| self.make_user_admin(name)),
        };

        let mut errors = Vec::<String>::new();
        for name in users
        {
            let res = func(name);
            if res.is_err()
            {
                errors.push(format!("{}", res.unwrap_err()));
            }
        }

        if errors.len() > 0
        {
            return Err(ServerError::Critical(format!("The following errors occured: {:?}", errors)));
        }

        Ok(0)
    }

    fn check_unique_name(&self, name: &String) -> ServerResult<bool>
    {
        let mut stmt = self.conn.prepare("select count(*) from users where name = :name")?;
        let count = stmt.query_map_named(named_params!{":name": name}, |row|
        {
            let c: i64 = row.get(0)?;
            Ok(c)
        })?.next().unwrap().unwrap();
        Ok(count == 0)
    }

    fn delete_new_user_notification(&self, id: i64) -> ServerResult<usize>
    {
        let mut stmt = self.conn.prepare("delete from new_user_notification where id = :id")?;
        stmt.execute_named(named_params!{":id": id})?;
        Ok(0)
    }

    fn create_user_from_notification(&self, id: i64) -> ServerResult<String>
    {
        let mut stmt = self.conn.prepare("select name, password_hash from new_user_notification where id = :id")?;
        let user = stmt.query_map_named(named_params!{":id": id}, |row|
        {
            let name: String = row.get(0)?;
            let password_hash: String = row.get(1)?;
            Ok((name, password_hash))
        })?.next();

        if user.is_none()
        {
            return Err(ServerError::Critical("err".into())); // Should not be possible to reach this
        }

        let user = user.unwrap().unwrap();
        self.create_user_with_password_hash(user.0, user.1)
    }

    fn create_user_with_password_hash(&self, new_user: String, password_hash: String) -> ServerResult<String>
    {
        let uuid = format!("{}", Uuid::new_v4());
        self.conn.execute(
            "insert into users (name, password_hash, uuid) values (?1, ?2, ?3)",
            params![new_user, password_hash, uuid],
            )?;
        Ok(uuid)
    }

    // Get all notifications for the users, exclude already
    // answered ones
    fn try_get_notifications(&self, token: String) -> ServerResult<Vec<MatchNotification>>
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
    fn try_get_new_user_notifications(&self, token: String) -> ServerResult<Vec<NewUserNotification>>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

        let mut stmt = self.conn.prepare("select id, name from new_user_notification")?;

        let notifications = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(
                NewUserNotification {
                    id: row.get(0)?,
                    name: row.get(1)?
                })
        })?;

        let mut vec: Vec<NewUserNotification> = Vec::new();
        for n in notifications
        {
            if let Ok(mn) = n
            {
                vec.push(mn);
            };
        }
        Ok(vec)
    }
    fn try_respond_to_notification(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let mut stmt = self.conn.prepare(
            "select id, winner_accept, loser_accept, epoch, winner, loser
            from match_notification where id = :id")?;
        let mut match_notification = stmt.query_map_named(named_params!{":id": id}, |row|
        {
            Ok(
                MatchNotificationTable {
                    id: row.get(0)?,
                    winner_accept: row.get(1)?,
                    loser_accept: row.get(2)?,
                    epoch: row.get(3)?,
                    winner: row.get(4)?,
                    loser: row.get(5)?
                })
        })?.next().expect("Unwrapping element").expect("Unwrapping Option");

        if user.id != match_notification.winner && user.id != match_notification.loser
        {
            return Err(ServerError::Unauthorized);
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

    fn handle_notification_answer(&self, user: &User, ans: u8, match_notification: &MatchNotificationTable) -> ServerResult<()>
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
            if user.id == match_notification.winner
            {

                let loser = &self.get_user_without_matches_by("id", "=", &match_notification.loser.to_string())?;
                self.create_match_from_notification(&match_notification, user, &loser)?;
            }
            else
            {
                let winner = &self.get_user_without_matches_by("id", "=", &match_notification.winner.to_string())?;
                self.create_match_from_notification(&match_notification, &winner, user)?;
            };

            self.delete_match_notification(&match_notification)?;
            return Ok(());
        }
        else
        {
            let col = if user.id == match_notification.winner { "winner_accept" } else { "loser_accept" };
            self.update_match_notification_answer(col, ans, match_notification.id)
        }
    }

    fn update_match_notification_answer(&self, col: &str, ans: u8, id: i64) -> ServerResult<()>
    {
        let mut stmt = self.conn.prepare(&format!("update match_notification set {} = :ans where id = :id", col))?;
        stmt.execute_named(named_params!{":id": id, ":ans": ans})?;
        Ok(())
    }

    fn delete_match_notification(&self, m: &MatchNotificationTable) -> ServerResult<()>
    {
        let mut stmt = self.conn.prepare("delete from match_notification where id = :id")?;
        stmt.execute_named(named_params!{":id": m.id})?;
        Ok(())
    }

    fn create_match_from_notification(&self, m: &MatchNotificationTable, winner: &User, loser: &User) -> ServerResult<()>
    {
        let elo = EloRank { k: 32 };
        let (new_winner_elo, new_loser_elo) = elo.calculate(winner.elo, loser.elo);


        self.conn.execute(
            "insert into matches (epoch, winner, loser, elo_diff, winner_elo, loser_elo)
            values (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                m.epoch,
                m.winner,
                m.loser,
                new_winner_elo - winner.elo,
                new_winner_elo,
                new_loser_elo],)?;


        if self.need_to_roll_back(m.epoch)?
        {
           self.roll_back(m.epoch)?;
           return Ok(());
        }

        self.update_elo(winner.id, new_winner_elo)?;
        self.update_elo(loser.id, new_loser_elo)?;
        Ok(())
    }

    fn need_to_roll_back(&self, epoch: i64) -> ServerResult<bool>
    {
        let mut stmt = self.conn.prepare("select count(*) from matches
                               where epoch > :epoch;")?;

        let count = stmt.query_map_named(named_params!{":epoch": epoch}, |row|
        {
            let n: i64 = row.get(0)?;
            Ok(n)
        })?.next().unwrap().unwrap();

        Ok(count > 0)
    }

    fn create_match_notification(&self, winner: &User, loser: &User, token: String) -> ServerResult<()>
    {
        if self.user_have_token(winner.id, &token)?
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser, winner_accept) values (?1, ?2, ?3, ?4)",
                params![self.epoch(), winner.id, loser.id, ACCEPT_MATCH],)?;
        }
        else if self.user_have_token(loser.id, &token)?
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser,  loser_accept) values (?1, ?2, ?3, ?4)",
                params![self.epoch(), winner.id, loser.id,  ACCEPT_MATCH],)?;
        }
        else
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser) values (?1, ?2, ?3)",
                params![self.epoch(), winner.id, loser.id],)?;
        }
        Ok(())
    }

    fn user_have_token(&self, user_id: i64, token: &String) -> ServerResult<bool>
    {
        let mut stmt = self.conn.prepare("select count(*) from users where id = :id and uuid = :token")?;
        let mut c = stmt.query_map_named(named_params!{":id": user_id, ":token": token}, |row|
        {
            let c: i64 = row.get(0)?;
            Ok(c)
        })?;

        Ok(c.next().unwrap().unwrap() == 1)
    }

    fn try_change_password(&self, name: String, password: String, new_password: String) -> ServerResult<()>
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

                return Err(ServerError::PasswordNotMatch);
            },
            _ => Err(ServerError::UserNotExist)
        }
    }

    fn update_password(&self, id: i64, new_password: String) -> ServerResult<()>
    {
        let hash = self.hash(&new_password);
        let mut stmt = self.conn.prepare("update users  set password_hash = :hash WHERE id = :id")?;
        stmt.execute_named(named_params!{":hash": hash, ":id": id})?;
        Ok(())
    }

    fn try_login(&self, name: String, password: String) -> ServerResult<String>
    {
        let mut stmt = self.conn.prepare("select password_hash, uuid, user_role from users where name = :name;")?;
        let info = stmt.query_map_named(named_params!{":name" : name}, |row|
        {
            let passwd: String = row.get(0)?;
            let uuid: String = row.get(1)?;
            let role: u8 = row.get(2)?;
            Ok((passwd, uuid, role))
        })?.next();

        if info.is_none()
        {
            let mut stmt = self.conn.prepare("select count(*) from new_user_notification where name = :name")?;
            let c = stmt.query_map_named(named_params!{":name": &name}, |row|
            {
                let c: i64 = row.get(0)?;
                Ok(c)
            })?.next().unwrap().unwrap();

            if c == 1
            {
                return Err(ServerError::WaitingForAdmin);
            }

            return Err(ServerError::UserNotExist);
        }

        match info.unwrap()
        {
            Ok((p, u, r)) =>
            {
                if r & USER_ROLE_INACTIVE == USER_ROLE_INACTIVE
                {
                    return Err(rusqlite::Error::InvalidParameterName("User is inactive".to_string()))
                }

                if self.hash(&password) == p
                {
                    return Ok(u);
                }

                return Err(ServerError::PasswordNotMatch)
            },
            _ => Err(ServerError::UserNotExist)
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

    fn get_all_matches(&self) -> ServerResult<Vec<Match>>
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

    fn update_elo(&self, id: i64, elo: f64) -> ServerResult<()>
    {
        let mut stmt = self.conn.prepare("update users  set elo = :elo WHERE id = :id")?;
        stmt.execute_named(named_params!{":elo": elo, ":id": id})?;
        Ok(())
    }

    fn get_matches(&self, id: i64) -> ServerResult<Vec<Match>>
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
        Ok(vec)
    }


    fn _get_all_users(&self, token: String) -> ServerResult<Vec<User>>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

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
        Ok(vec)
    }

    fn get_active_users(&self) -> ServerResult<Vec<User>>
    {
        let mut stmt = self.conn.prepare("select id, name, elo, user_role from users where user_role & :inactive != :inactive;")?;
        let users = stmt.query_map_named(named_params!{":inactive": USER_ROLE_INACTIVE}, |row|
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
        Ok(vec)
    }

    fn get_user_without_matches(&self, name: &String) -> ServerResult<User>
    {
        self.get_user_without_matches_by("name", "=", name.as_str())
    }

    fn get_user(&self, name: &String) -> ServerResult<User>
    {
        let mut user = self.get_user_without_matches_by("name", "=", name.as_str())?;
        user.match_history = self.get_matches(user.id)?;
        Ok(user)
    }

    fn  get_user_without_matches_by(&self, col: &str, comp: &str, val: &str) -> ServerResult<User>
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
            Some(Err(_)) => Err(ServerError::Critical("???")), // What does this even mean 🤷‍♀️
            None => Err(ServerError::UserNotExist)
        }
    }

    #[allow(dead_code)]
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

    /*#[test]
    fn test_register_user_creats_notification()
    {
    }

    #[test]
    fn test_register_user_can_accept_user()
    {
    }
    */

    fn create_user(s: &DataBase, name: &str) -> String
    {
        let uuid = format!("{}", Uuid::new_v4());
        s.conn.execute(
            "insert into users (name, password_hash, uuid) values (?1, ?2, ?3)",
            params![name, s.hash(&"password".to_string()), uuid],
            ).unwrap();
        uuid
    }

    #[test]
    fn test_match_notification_both_accepted()
    {
        let db_file = "temp1.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner, loser, uuid).expect("Creating match");
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

    #[test]
    fn test_match_registered_by_none_participant_gets_answered_no()
    {
        let db_file = "tempB.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");
        create_user(&s, "Lars");
        let uuid = create_user(&s, "Bernt");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner.clone(), loser.clone(), uuid.clone()).expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), uuid).expect("Creating match");


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
        let winner_uuid = create_user(&s, "Sivert");
        let loser_uuid = create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();

        s.register_match(winner.clone(), loser.clone(), winner_uuid).expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), loser_uuid).expect("Creating match");


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
        let uuid = create_user(&s, "Sivert");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner, loser, uuid).expect("Creating match");
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
        let uuid = create_user(&s, "Sivert");
        create_user(&s, "Lars");

        let winner =  "Sivert".to_string();
        let loser = "Lars".to_string();

        s.register_match(winner, loser, uuid).expect("Creating match");

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
        create_user(&s, "Sivertt");
        create_user(&s, "Sivert");

        let user = s.get_user_without_matches_by("name", "=", "Sivert");
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(user.is_ok() && user.unwrap().name == "Sivert");
    }

    #[test]
    fn test_can_get_user_by_id()
    {
        let db_file = "temp5.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");

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
        create_user(&s, &name);

        let uuid = s.login(name, password);
        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(uuid.is_ok(), uuid.unwrap().len() == 36);
    }

    #[test]
    fn test_login_cant_log_in_with_wrong_password()
    {
        let db_file = "temp7.db";
        let s = DataBase::new(db_file);
        let name = "Sivert".to_string();
        create_user(&s, &name);

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
        create_user(&s, &name);
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

    #[test]
    fn test_rolls_back_correctly()
    {
        let db_file = "tempC.db";
        let s = DataBase::new(db_file);

        let siv = "Sivert".to_string();
        let lars = "Lars".to_string();
        let bernt = "Bernt".to_string();

        let token_siv = create_user(&s, siv.as_str());
        create_user(&s, lars.as_str());
        create_user(&s, bernt.as_str());

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
         * If no rollback happend the result should be
         * Sivert: 1512.016815723276
         * Lars  : 1471.2468774832018
         * Bernt : 1516.736306793522
         *
         * But they should actually be
         * Sivert: 1514.2851406137202
         * Lars  : 1468.2570986091923
         * Bernt : 1517.4577607770875
         * A _little_ different, BUT! More correct, rollback should fix these
         */
        let (siv_user, lars_user, bernt_user) = (s.get_user_without_matches(&siv).unwrap(), 
                                                 s.get_user_without_matches(&lars).unwrap(),
                                                 s.get_user_without_matches(&bernt).unwrap());

        std::fs::remove_file(db_file).expect("Removing file temp");
        assert_eq!(siv_user.elo, 1514.2851406137202);
        assert_eq!(lars_user.elo, 1468.2570986091923);
        assert_eq!(bernt_user.elo, 1517.4577607770875);
    }

    #[test]
    fn test_case_sensitive_user_search()
    {
        let db_file = "tempE.db";
        let s = DataBase::new(db_file);

        let _siv = "Sivert".to_string();
        let siv = "sivert".to_string();

        create_user(&s, _siv.as_str());
        create_user(&s, siv.as_str());

        let user = s.get_user(&siv.clone()).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempE");
        assert_eq!(user.name, siv);
    }
}
