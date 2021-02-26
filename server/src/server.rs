use std::{collections::HashMap, convert::From, str::FromStr};

use chrono::prelude::*;
use elo::EloRank;
use lazy_static::lazy_static;
use rusqlite::{named_params, params, Connection, NO_PARAMS};
use uuid::Uuid;

use crate::{
    badge::*,
    r#match::{DeleteMatchInfo, EditMatchInfo, Match, NewEditMatchInfo},
    notification::{
        AdminNotification, AdminNotificationAns, MatchNotification, MatchNotificationTable,
    },
    server_season::{IS_SEASON_ID, N_SEASON_ID, REQUIRE_CONFIRMATION_ID},
    user::{
        EditUserAction, StatsUsers, User, USER_ROLE_INACTIVE, USER_ROLE_REGULAR,
        USER_ROLE_SOFT_INACTIVE, USER_ROLE_SUPERUSER,
    },
};

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

// Map the name to the id _and_ the default value
lazy_static! {
    static ref HASHMAP: HashMap<&'static str, (i64, i64)> = {
        let mut m = HashMap::new();
        m.insert("is_season", (IS_SEASON_ID as i64, 1));
        m.insert("season_length", (N_SEASON_ID as i64, 1));
        m.insert("user_conf", (REQUIRE_CONFIRMATION_ID as i64, 0));
        m
    };
}


pub struct DataBase
{
    pub conn: Connection,
}
#[allow(dead_code)]
const MATCH_NO_ANS: u8 = 0;
pub const ACCEPT_REQUEST: u8 = 1;
#[allow(dead_code)]
const DECLINE_REQUEST: u8 = 2;

pub const STOP_SEASON: i64 = -1;
pub const START_SEASON: i64 = -2;


pub type ServerResult<T> = rusqlite::Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError
{
    Rusqlite(rusqlite::Error),
    Critical(String),
    UserNotExist,
    UsernameTaken,
    WrongUsernameOrPassword,
    PasswordNotMatch,
    Unauthorized,
    WaitingForAdmin,
    InactiveUser,
    ResetPasswordDuplicate,
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
            _ => unreachable!(),
        }
    }
}

impl DataBase
{
    pub fn new(file: &str) -> Self
    {
        DataBase::init(file)
    }

    pub fn create_superuser(&self, name: String) -> ServerResult<()>
    {
        let password = self.hash(&"password".to_string());
        self.create_user_with_password_hash(name.clone(), password)?;
        self.make_user_admin(name)?;
        Ok(())
    }

    pub fn get_is_admin(&self, token: String) -> ServerResult<bool>
    {
        let user = self.get_user_without_matches_by("uuid", "=", &token)?;
        Ok(user.user_role & USER_ROLE_SUPERUSER == USER_ROLE_SUPERUSER)
    }

    pub fn admin_rollback(&self, token: String) -> ServerResult<()>
    {
        if self.get_is_admin(token)?
        {
            self.roll_back(-1)?;
        }
        Ok(())
    }

    pub fn delete_match(&self, info: DeleteMatchInfo) -> ServerResult<()>
    {
        if !self.get_is_admin(info.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        self.try_delete_match(info)?;
        self.roll_back(-1)?;
        Ok(())
    }

    pub fn edit_match(&self, info: NewEditMatchInfo) -> ServerResult<()>
    {
        if !self.get_is_admin(info.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        self.update_match(info)?;
        self.roll_back(-1)?;
        Ok(())
    }

    pub fn login(&self, name: String, password: String) -> ServerResult<String> // String = Uuid
    {
        self.try_login(name, password)
    }

    pub fn respond_to_match(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        self.try_respond_to_notification(id, ans, token)
    }

    pub fn get_variable(&self, variable: String) -> ServerResult<i64>
    {
        let (id, default) = match HASHMAP.get(variable.as_str())
        {
            Some(t) => t,
            None => return Err(ServerError::Critical(format!("No varialbe named {}", variable))),
        };

        let res: Result<i64, rusqlite::Error> = GET_OR_CREATE_DB_VAR!(&self.conn, *id, *default);
        match res
        {
            Ok(r) => Ok(r),
            Err(e) => Err(ServerError::Rusqlite(e)),
        }
    }

    pub fn set_variable(&self, token: String, varialbe: String, val: i64) -> ServerResult<()>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

        let id = match HASHMAP.get(varialbe.as_str())
        {
            Some((id, _)) => id,
            None => return Err(ServerError::Critical(format!("No variable named: {}", varialbe))),
        };

        self.conn
            .execute("replace into variables(id, value) values (?1, ?2);", params![id, val])?;
        Ok(())
    }

    pub fn create_user(&self, new_user: String, password: String) -> ServerResult<String>
    {
        if self.get_variable("user_conf".to_string())? == 1
        {
            self.create_new_user_notification(new_user, password)?;
            Ok(" Now you have to wait for an admin to accept..!".to_string())
        }
        else
        {
            if !self.check_unique_name(&new_user)?
            {
                return Err(ServerError::UsernameTaken);
            }
            let password_hash = self.hash(&password);
            self.create_user_with_password_hash(new_user, password_hash)?;
            Ok("".to_string())
        }
    }

    pub fn edit_users(
        &self,
        users: Vec<String>,
        action: String,
        token: String,
    ) -> ServerResult<usize>
    {
        self._edit_users(users, action, token)
    }

    pub fn get_profile(&self, user: String) -> ServerResult<User>
    {
        self.get_user(&user)
    }

    pub fn get_all_users(&self, token: String) -> ServerResult<Vec<User>>
    {
        self._get_all_users(token)
    }

    pub fn get_non_inactive_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_all_non_inactive_users()
    }

    pub fn get_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_active_users()
    }

    pub fn get_multiple_users(&self, users: Vec<i64>) -> ServerResult<Vec<User>>
    {
        let list = format!("{:?}", users).as_str().replace("[", "(").replace("]", ")");
        let sql = format!("select id, name, elo, user_role from users where id in {}", list);
        let mut stmt = self.conn.prepare(&sql)?;
        let users = stmt.query_map(NO_PARAMS, |row| {
            let id: i64 = row.get(0)?;
            Ok(User {
                id:            id,
                name:          row.get(1)?,
                elo:           row.get(2)?,
                user_role:     row.get(3)?,
                match_history: self.get_matches(id)?,
                badges:        Vec::new(),
            })
        })?;

        let mut vec = Vec::new();
        for user in users
        {
            if let Ok(u) = user
            {
                vec.push(u);
            }
        }
        Ok(vec)
    }

    pub fn register_match(
        &self,
        winner_name: String,
        loser_name: String,
        token: String,
    ) -> ServerResult<()>
    {
        if self.get_user_without_matches_by("uuid", "=", &token).is_err()
        {
            return Err(ServerError::UserNotExist);
        };

        let (winner, loser) = (
            self.get_user_without_matches(&winner_name)?,
            self.get_user_without_matches(&loser_name)?,
        );

        self.create_match_notification(&winner, &loser, token)
    }

    pub fn get_history(&self) -> ServerResult<Vec<Match>>
    {
        self.get_all_matches()
    }

    pub fn get_stats(
        &self,
        info: StatsUsers,
    ) -> ServerResult<(Vec<i64>, HashMap<String, Vec<Match>>)>
    {
        self._get_stats(info)
    }

    pub fn get_edit_match_history(&self) -> ServerResult<Vec<EditMatchInfo>>
    {
        self.get_all_edit_matches()
    }

    pub fn change_password(
        &self,
        name: String,
        password: String,
        new_password: String,
    ) -> ServerResult<()>
    {
        self.try_change_password(name, password, new_password)
    }

    pub fn request_reset_password(&self, name: String) -> ServerResult<()>
    {
        self._request_reset_password(name)
    }

    pub fn get_notifications(&self, token: String) -> ServerResult<Vec<MatchNotification>>
    {
        self.try_get_notifications(token)
    }

    pub fn get_admin_notifications(
        &self,
        token: String,
    ) -> ServerResult<HashMap<String, Vec<AdminNotification>>>
    {
        self._get_admin_notifications(token)
    }

    pub fn respond_to_new_user(&self, not: AdminNotificationAns) -> ServerResult<()>
    {
        if !self.get_is_admin(not.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        // No match is being accepted, but the ans values are the same Xdd
        if not.ans == ACCEPT_REQUEST
        {
            self.create_user_from_notification(not.id)?;
        }
        self.delete_new_user_notification(not.id)?;
        Ok(())
    }

    pub fn respond_to_reset_password(&self, info: AdminNotificationAns) -> ServerResult<()>
    {
        if !self.get_is_admin(info.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        if info.ans == ACCEPT_REQUEST
        {
            self.reset_password(info.id)?;
        }
        self.delete_reset_password_notification(info.id)?;
        Ok(())
    }

    pub fn request_new_name(&self, token: String, new_name: String) -> ServerResult<()>
    {
        if !self.check_unique_name(&new_name)?
        {
            return Err(ServerError::UsernameTaken);
        }
        let user = self.get_user_without_matches_by("uuid", "=", &token)?;
        self.conn.execute(
            "insert into new_name_notification (user, new_name) values (?1, ?2)",
            params![user.id, new_name],
        )?;

        Ok(())
    }

    pub fn set_new_name(&self, id: i64, new_name: String) -> ServerResult<()>
    {
        self.conn
            .execute("update users set name = (?1) where id = (?2)", params![new_name, id])?;
        Ok(())
    }

    pub fn respond_to_new_name_notification(&self, not: AdminNotificationAns) -> ServerResult<()>
    {
        if !self.get_is_admin(not.token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        if not.ans == ACCEPT_REQUEST
        {
            let (id, new_name): (i64, String) = self.conn.query_row(
                "select user, new_name from new_name_notification where id = (?1)",
                params![not.id],
                |row| {
                    let id: i64 = row.get(0)?;
                    let name: String = row.get(1)?;
                    Ok((id, name))
                },
            )?;
            self.set_new_name(id, new_name)?;
        }
        self.conn
            .execute("delete from new_name_notification where id = (?1)", params![not.id])?;
        Ok(())
    }
}


// Only private  functions here ~!
impl DataBase
{
    fn try_delete_match(&self, info: DeleteMatchInfo) -> ServerResult<()>
    {
        self.conn
            .execute(&format!("delete from matches where id = {}", info.id), NO_PARAMS)?;
        Ok(())
    }

    fn update_match(&self, info: NewEditMatchInfo) -> ServerResult<()>
    {
        let winner = self.get_user_without_matches(&info.winner)?.id;
        let loser = self.get_user_without_matches(&info.loser)?.id;
        self.conn.execute(
            &format!(
                "update matches
                set winner = {}, loser = {}, epoch = {}
                where id = {}",
                winner, loser, info.epoch, info.id
            ),
            NO_PARAMS,
        )?;
        Ok(())
    }

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
            &format!(
                "update users
                set user_role = {}
                where name = \"{}\"",
                role, name
            ),
            NO_PARAMS,
        )?;
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
            &format!(
                "update users
                set user_role = {}
                where name = \"{}\"",
                role, name
            ),
            NO_PARAMS,
        )?;
        Ok(0)
    }

    fn make_user_active(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        if user.user_role & (USER_ROLE_INACTIVE | USER_ROLE_SOFT_INACTIVE) > 0
        {
            let role = user.user_role & !USER_ROLE_INACTIVE & !USER_ROLE_SOFT_INACTIVE;
            let mut stmt =
                self.conn.prepare("update users set user_role = :role where name = :name")?;
            stmt.execute_named(named_params! {":role": role, ":name": name})?;
        }
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
        let mut stmt =
            self.conn.prepare("update users set user_role = :role where name = :name")?;
        stmt.execute_named(named_params! {":role": role, ":name": name})?;
        Ok(0)
    }

    fn make_user_soft_inactive(&self, name: String) -> ServerResult<usize>
    {
        let user: User = match self.get_user_without_matches(&name)
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let role = user.user_role | USER_ROLE_SOFT_INACTIVE;
        let mut stmt =
            self.conn.prepare("update users set user_role = :role where name = :name")?;
        stmt.execute_named(named_params! {":role": role, ":name": name})?;
        Ok(0)
    }

    fn _edit_users(&self, users: Vec<String>, action: String, token: String)
        -> ServerResult<usize>
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
            EditUserAction::MakeUserRegular =>
            {
                Box::new(|name: String| self.make_user_regular(name))
            },
            EditUserAction::MakeUserInactive =>
            {
                Box::new(|name: String| self.make_user_inactive(name))
            },
            EditUserAction::MakeUserSoftInactive =>
            {
                Box::new(|name: String| self.make_user_soft_inactive(name))
            },
            EditUserAction::MakeUserSuperuser =>
            {
                Box::new(|name: String| self.make_user_admin(name))
            },
        };

        let mut errors = Vec::<String>::new();
        for name in users
        {
            let res = func(name);
            if res.is_err()
            {
                errors.push(format!("{:?}", res.unwrap_err()));
            }
        }

        if errors.len() > 0
        {
            return Err(ServerError::Critical(format!(
                "The following errors occured: {:?}",
                errors
            )));
        }

        Ok(0)
    }

    fn check_unique_name(&self, name: &String) -> ServerResult<bool>
    {
        let mut stmt = self.conn.prepare(
            "select
             (select count(*) from users where name = :name)+
             (select count(*) from new_user_notification where name = :name);
        ",
        )?;

        let count = stmt
            .query_map_named(named_params! {":name": name}, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })?
            .next()
            .unwrap()
            .unwrap();
        Ok(count == 0)
    }

    fn delete_new_user_notification(&self, id: i64) -> ServerResult<usize>
    {
        let mut stmt = self.conn.prepare("delete from new_user_notification where id = :id")?;
        stmt.execute_named(named_params! {":id": id})?;
        Ok(0)
    }

    fn delete_reset_password_notification(&self, id: i64) -> ServerResult<usize>
    {
        let mut stmt =
            self.conn.prepare("delete from reset_password_notification where id = :id")?;
        stmt.execute_named(named_params! {":id": id})?;
        Ok(0)
    }

    fn create_user_from_notification(&self, id: i64) -> ServerResult<String>
    {
        let mut stmt = self
            .conn
            .prepare("select name, password_hash from new_user_notification where id = :id")?;
        let user = stmt
            .query_map_named(named_params! {":id": id}, |row| {
                let name: String = row.get(0)?;
                let password_hash: String = row.get(1)?;
                Ok((name, password_hash))
            })?
            .next();

        if user.is_none()
        {
            return Err(ServerError::Critical("err".into())); // Should not be possible to reach this
        }

        let user = user.unwrap().unwrap();
        self.create_user_with_password_hash(user.0, user.1)
    }

    fn create_user_with_password_hash(
        &self,
        new_user: String,
        password_hash: String,
    ) -> ServerResult<String>
    {
        let uuid = format!("{}", Uuid::new_v4());
        self.conn.execute(
            "insert into users (name, password_hash, uuid, user_role) values (?1, ?2, ?3, ?4)",
            params![new_user, password_hash, uuid, USER_ROLE_SOFT_INACTIVE | USER_ROLE_REGULAR],
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
            where winner = :id and winner_accept = 0
            union
            select id, winner, loser, epoch from match_notification
            where loser = :id and loser_accept = 0
            ",
        )?;

        let notifications = stmt.query_map_named(named_params! {":id": user.id}, |row| {
            let winner_id: i64 = row.get(1)?;
            let loser_id: i64 = row.get(2)?;

            let winner_name: String =
                self.get_user_without_matches_by("id", "=", &winner_id.to_string())?.name;
            let loser_name: String =
                self.get_user_without_matches_by("id", "=", &loser_id.to_string())?.name;
            Ok(MatchNotification {
                id:     row.get(0)?,
                winner: winner_name,
                loser:  loser_name,
                epoch:  row.get(3)?,
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

    fn _get_admin_notifications(
        &self,
        token: String,
    ) -> ServerResult<HashMap<String, Vec<AdminNotification>>>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

        let new_user = self.try_get_new_user_notifications()?;
        let reset_password = self.try_get_reset_password_notifications()?;
        let rename = self.try_get_rename_notifications()?;

        let mut map = HashMap::new();
        map.insert("new_users".to_string(), new_user);
        map.insert("reset_password".to_string(), reset_password);
        map.insert("rename".to_string(), rename);
        Ok(map)
    }

    fn try_get_new_user_notifications(&self) -> ServerResult<Vec<AdminNotification>>
    {
        let mut stmt = self.conn.prepare("select id, name from new_user_notification")?;
        let notifications = stmt.query_map(NO_PARAMS, |row| {
            Ok(AdminNotification {
                id: row.get(0)?, name: row.get(1)?
            })
        })?;

        let mut vec: Vec<AdminNotification> = Vec::new();
        for n in notifications
        {
            if let Ok(mn) = n
            {
                vec.push(mn);
            };
        }
        Ok(vec)
    }

    fn try_get_rename_notifications(&self) -> ServerResult<Vec<AdminNotification>>
    {
        let mut stmt = self.conn.prepare("select * from new_name_notification as n")?;
        let notifications = stmt.query_map(NO_PARAMS, |row| {
            let user_id: i64 = row.get(1)?;
            let user_name = self.get_user_without_matches_by("id", "=", &user_id.to_string())?.name;
            let new_name: String = row.get(2)?;
            let _final = format!("{} -> {}", user_name, new_name);

            Ok(AdminNotification {
                id: row.get(0)?, name: _final
            })
        })?;

        let mut vec: Vec<AdminNotification> = Vec::new();
        for n in notifications
        {
            if let Ok(mn) = n
            {
                vec.push(mn);
            };
        }
        Ok(vec)
    }

    fn try_get_reset_password_notifications(&self) -> ServerResult<Vec<AdminNotification>>
    {
        let mut stmt = self.conn.prepare(
            "select n.id, name from reset_password_notification as n
             join users as a on a.id = n.user",
        )?;
        let notifications = stmt.query_map(NO_PARAMS, |row| {
            Ok(AdminNotification {
                id: row.get(0)?, name: row.get(1)?
            })
        })?;

        let mut vec: Vec<AdminNotification> = Vec::new();
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
            from match_notification where id = :id",
        )?;
        let mut match_notification = stmt.query_map_named(named_params! {":id": id}, |row| {
            Ok(MatchNotificationTable {
                id:            row.get(0)?,
                winner_accept: row.get(1)?,
                loser_accept:  row.get(2)?,
                epoch:         row.get(3)?,
                winner:        row.get(4)?,
                loser:         row.get(5)?,
            })
        })?;

        let mut match_notification = match match_notification.next()
        {
            Some(mn) => mn.unwrap(),
            None => return Err(ServerError::Critical(String::from("Notification does not exist"))),
        };

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

    fn handle_notification_answer(
        &self,
        user: &User,
        ans: u8,
        match_notification: &MatchNotificationTable,
    ) -> ServerResult<()>
    {
        if match_notification.loser_accept == ACCEPT_REQUEST
            && match_notification.winner_accept == ACCEPT_REQUEST
        {
            if user.id == match_notification.winner
            {
                let loser = &self.get_user_without_matches_by(
                    "id",
                    "=",
                    &match_notification.loser.to_string(),
                )?;
                self.create_match_from_notification(&match_notification, user, &loser)?;
            }
            else
            {
                let winner = &self.get_user_without_matches_by(
                    "id",
                    "=",
                    &match_notification.winner.to_string(),
                )?;
                self.create_match_from_notification(&match_notification, &winner, user)?;
            };

            self.delete_match_notification(&match_notification)?;
            return Ok(());
        }
        else
        {
            let col =
                if user.id == match_notification.winner { "winner_accept" } else { "loser_accept" };
            self.update_match_notification_answer(col, ans, match_notification.id)
        }
    }

    fn update_match_notification_answer(&self, col: &str, ans: u8, id: i64) -> ServerResult<()>
    {
        let mut stmt = self
            .conn
            .prepare(&format!("update match_notification set {} = :ans where id = :id", col))?;
        stmt.execute_named(named_params! {":id": id, ":ans": ans})?;
        Ok(())
    }

    fn delete_match_notification(&self, m: &MatchNotificationTable) -> ServerResult<()>
    {
        let mut stmt = self.conn.prepare("delete from match_notification where id = :id")?;
        stmt.execute_named(named_params! {":id": m.id})?;
        Ok(())
    }

    fn create_match_from_notification(
        &self,
        m: &MatchNotificationTable,
        winner: &User,
        loser: &User,
    ) -> ServerResult<()>
    {
        let elo = EloRank {
            k: 32
        };
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
                new_loser_elo
            ],
        )?;


        if self.need_to_roll_back(m.epoch)?
        {
            self.roll_back(m.epoch)?;
            return Ok(());
        }

        self.update_elo(winner.id, new_winner_elo)?;
        self.update_elo(loser.id, new_loser_elo)?;
        self.make_user_active(winner.name.clone())?;
        self.make_user_active(loser.name.clone())?;
        Ok(())
    }

    fn need_to_roll_back(&self, epoch: i64) -> ServerResult<bool>
    {
        let mut stmt = self.conn.prepare(
            "select count(*) from matches
             where epoch > :epoch;",
        )?;

        let count = stmt
            .query_map_named(named_params! {":epoch": epoch}, |row| {
                let n: i64 = row.get(0)?;
                Ok(n)
            })?
            .next()
            .unwrap()
            .unwrap();

        Ok(count > 0)
    }

    fn create_match_notification(
        &self,
        winner: &User,
        loser: &User,
        token: String,
    ) -> ServerResult<()>
    {
        if self.user_have_token(winner.id, &token)?
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser, winner_accept) values (?1, \
                 ?2, ?3, ?4)",
                params![self.epoch(), winner.id, loser.id, ACCEPT_REQUEST],
            )?;
        }
        else if self.user_have_token(loser.id, &token)?
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser,  loser_accept) values (?1, \
                 ?2, ?3, ?4)",
                params![self.epoch(), winner.id, loser.id, ACCEPT_REQUEST],
            )?;
        }
        else
        {
            self.conn.execute(
                "insert into match_notification (epoch, winner, loser) values (?1, ?2, ?3)",
                params![self.epoch(), winner.id, loser.id],
            )?;
        }
        Ok(())
    }

    fn user_have_token(&self, user_id: i64, token: &String) -> ServerResult<bool>
    {
        let mut stmt = self
            .conn
            .prepare("select count(*) from users where id = :id and uuid = :token")?;
        let mut c =
            stmt.query_map_named(named_params! {":id": user_id, ":token": token}, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })?;

        Ok(c.next().unwrap().unwrap() == 1)
    }

    fn try_change_password(
        &self,
        name: String,
        password: String,
        new_password: String,
    ) -> ServerResult<()>
    {
        let mut stmt =
            self.conn.prepare("select id, password_hash from users where name = :name;")?;
        let info = stmt
            .query_map_named(named_params! {":name" : name}, |row| {
                let id: i64 = row.get(0)?;
                let passwd: String = row.get(1)?;
                Ok((id, passwd))
            })?
            .next()
            .expect("getting user");

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
            _ => Err(ServerError::UserNotExist),
        }
    }

    fn update_password(&self, id: i64, new_password: String) -> ServerResult<()>
    {
        let hash = self.hash(&new_password);
        let mut stmt =
            self.conn.prepare("update users  set password_hash = :hash WHERE id = :id")?;
        stmt.execute_named(named_params! {":hash": hash, ":id": id})?;
        Ok(())
    }

    fn reset_password(&self, id: i64) -> ServerResult<()>
    {
        let mut stmt = self
            .conn
            .prepare("select user from reset_password_notification where id = :id;")?;
        let user_id = stmt
            .query_map_named(named_params! {":id" : id}, |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            })?
            .next();
        if user_id.is_none()
        {
            return Err(ServerError::Critical("Something went wrong".into())); // Should not be possible to reach this
        }

        let user_id = user_id.unwrap().unwrap();
        let hash = self.hash(&"@uit".to_string());
        stmt = self.conn.prepare("update users set password_hash = :hash where id = :id")?;
        stmt.execute_named(named_params! {":hash": hash, ":id": user_id})?;
        Ok(())
    }

    fn _request_reset_password(&self, user: String) -> ServerResult<()>
    {
        let user_id = self.get_user_without_matches(&user)?.id;
        let mut stmt = self
            .conn
            .prepare("select user from reset_password_notification where user = :id;")?;
        let notification = stmt
            .query_map_named(named_params! {":id" : user_id}, |row| {
                let id: i64 = row.get(0)?;
                Ok(id)
            })?
            .next();
        if !notification.is_none()
        {
            return Err(ServerError::ResetPasswordDuplicate);
        }

        stmt = self
            .conn
            .prepare("insert into reset_password_notification (user) values (:id)")?;
        stmt.execute_named(named_params! {":id": user_id})?;
        Ok(())
    }

    fn try_login(&self, name: String, password: String) -> ServerResult<String>
    {
        let mut stmt = self
            .conn
            .prepare("select password_hash, uuid, user_role from users where name = :name;")?;
        let info = stmt
            .query_map_named(named_params! {":name" : name}, |row| {
                let passwd: String = row.get(0)?;
                let uuid: String = row.get(1)?;
                let role: u8 = row.get(2)?;
                Ok((passwd, uuid, role))
            })?
            .next();

        if info.is_none()
        {
            let mut stmt = self
                .conn
                .prepare("select count(*) from new_user_notification where name = :name")?;
            let c = stmt
                .query_map_named(named_params! {":name": &name}, |row| {
                    let c: i64 = row.get(0)?;
                    Ok(c)
                })?
                .next()
                .unwrap()
                .unwrap();

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
                    return Err(ServerError::InactiveUser);
                }

                if self.hash(&password) == p
                {
                    return Ok(u);
                }

                return Err(ServerError::WrongUsernameOrPassword);
            },
            _ => Err(ServerError::UserNotExist),
        }
    }

    fn hash(&self, word: &String) -> String
    {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(word);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    fn get_all_edit_matches(&self) -> ServerResult<Vec<EditMatchInfo>>
    {
        let mut stmt = self.conn.prepare(
            "select a.name, b.name, epoch, m.id from matches as m
             inner join users as a on a.id = winner
             inner join users as b on b.id = loser
             order by epoch;",
        )?;
        let matches = stmt.query_map(NO_PARAMS, |row| {
            Ok(EditMatchInfo {
                winner: row.get(0)?,
                loser:  row.get(1)?,
                epoch:  row.get(2)?,
                id:     row.get(3)?,
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
        vec.reverse();
        Ok(vec)
    }

    fn get_all_matches(&self) -> ServerResult<Vec<Match>>
    {
        let current_season = self.get_latest_season_number()?;
        let mut stmt = self.conn.prepare(
            "select a.name, b.name, elo_diff, winner_elo, loser_elo, epoch from matches
             inner join users as a on a.id = winner
             inner join users as b on b.id = loser
             order by epoch;",
        )?;
        let matches = stmt.query_map(NO_PARAMS, |row| {
            Ok(Match {
                winner:     row.get(0)?,
                loser:      row.get(1)?,
                elo_diff:   row.get(2)?,
                winner_elo: row.get(3)?,
                loser_elo:  row.get(4)?,
                epoch:      row.get(5)?,
                season:     current_season, // -1 if off-season
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
        vec.reverse();
        Ok(vec)
    }

    fn get_stats_from_table(
        &self,
        table: String,
        info: &StatsUsers,
        user1_id: i64,
        user2_id: i64,
    ) -> ServerResult<Vec<Match>>
    {
        let mut stmt = self.conn.prepare(&format!(
            "select winner, loser, elo_diff, winner_elo, loser_elo, epoch, {}
             where winner = :user1 and loser = :user2
             or winner = :user2 and loser = :user1;",
            table
        ))?;
        let matches =
            stmt.query_map_named(named_params! {":user1": user1_id, ":user2": user2_id}, |row| {
                let res: i64 = row.get(0)?;
                let (winner, loser) = if res == user1_id
                {
                    (info.user1.clone(), info.user2.clone())
                }
                else
                {
                    (info.user2.clone(), info.user1.clone())
                };
                Ok(Match {
                    winner:     winner,
                    loser:      loser,
                    elo_diff:   row.get(2)?,
                    winner_elo: row.get(3)?,
                    loser_elo:  row.get(4)?,
                    epoch:      row.get(5)?,
                    season:     row.get(6)?,
                })
            })?;

        let mut vec = Vec::new();
        for _match in matches
        {
            if let Ok(m) = _match
            {
                vec.push(m);
            };
        }
        Ok(vec)
    }

    fn _get_stats(&self, info: StatsUsers)
        -> ServerResult<(Vec<i64>, HashMap<String, Vec<Match>>)>
    {
        let user1_id = self.get_user_without_matches(&info.user1)?.id;
        let user2_id = self.get_user_without_matches(&info.user2)?.id;

        let current_season = self.get_latest_season_number()?;
        let current = self.get_stats_from_table(
            format!("{} from matches", current_season),
            &info,
            user1_id,
            user2_id,
        )?;
        let rest = self.get_stats_from_table(
            "season from old_matches".to_string(),
            &info,
            user1_id,
            user2_id,
        )?;

        let mut map = HashMap::new();
        map.insert("current".to_string(), current);
        map.insert("rest".to_string(), rest);
        Ok((vec![user1_id, user2_id], map))
    }

    fn update_elo(&self, id: i64, elo: f64) -> ServerResult<()>
    {
        let mut stmt = self.conn.prepare("update users set elo = :elo WHERE id = :id")?;
        stmt.execute_named(named_params! {":elo": elo, ":id": id})?;
        Ok(())
    }

    fn get_matches(&self, id: i64) -> ServerResult<Vec<Match>>
    {
        let current_season = self.get_latest_season_number()?;
        let s = "select a.name, b.name, elo_diff, winner_elo, loser_elo, epoch
                from matches
                inner join users as a on a.id = winner
                inner join users as b on b.id = loser
                where winner = :id or loser = :id";
        let mut stmt = self.conn.prepare(s)?;
        let matches = stmt.query_map_named(named_params! {":id" : id}, |row| {
            Ok(Match {
                winner:     row.get(0)?,
                loser:      row.get(1)?,
                elo_diff:   row.get(2)?,
                winner_elo: row.get(3)?,
                loser_elo:  row.get(4)?,
                epoch:      row.get(5)?,
                season:     current_season,
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
        let users = stmt.query_map(NO_PARAMS, |row| {
            Ok(User {
                id:            row.get(0)?,
                name:          row.get(1)?,
                elo:           row.get(2)?,
                user_role:     row.get(3)?,
                match_history: Vec::new(),
                badges:        Vec::new(),
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

    fn get_badges(&self, pid: i64) -> ServerResult<Vec<Badge>>
    {
        let mut stmt = self
            .conn
            .prepare("select id, season_id, badge_index, pid from badges where pid = :pid")?;
        let badges = stmt.query_map_named(named_params! {":pid": pid}, |row| {
            let index: u32 = row.get(2)?;
            let index = index as usize;

            Ok(Badge {
                id:     row.get(0)?,
                season: row.get(1)?,
                name:   BADGES[index].to_string(),
            })
        })?;

        let mut vec = Vec::new();
        for badge in badges
        {
            if let Ok(b) = badge
            {
                vec.push(b);
            }
        }
        return Ok(vec);
    }

    fn get_users_with_user_role(&self, user_role: u8, val: u8) -> ServerResult<Vec<User>>
    {
        let mut stmt = self.conn.prepare(
            "select id, name, elo, user_role from users
             where user_role & :user_role = :val;",
        )?;
        let users =
            stmt.query_map_named(named_params! {":user_role": user_role, ":val": val}, |row| {
                Ok(User {
                    id:            row.get(0)?,
                    name:          row.get(1)?,
                    elo:           row.get(2)?,
                    user_role:     row.get(3)?,
                    match_history: Vec::new(),
                    badges:        Vec::new(),
                })
            })?;

        let mut vec = Vec::new();
        for user in users
        {
            if let Ok(mut u) = user
            {
                u.badges = self.get_badges(u.id)?;
                vec.push(u);
            };
        }
        vec.sort_by(|a, b| b.elo.partial_cmp(&a.elo).unwrap());
        Ok(vec)
    }

    fn get_all_non_inactive_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_users_with_user_role(USER_ROLE_INACTIVE, 0)
    }

    fn get_active_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_users_with_user_role(USER_ROLE_INACTIVE | USER_ROLE_SOFT_INACTIVE, 0)
    }

    fn get_user_without_matches(&self, name: &String) -> ServerResult<User>
    {
        self.get_user_without_matches_by("name", "=", name.as_str())
    }

    fn get_user(&self, name: &String) -> ServerResult<User>
    {
        let mut user = self.get_user_without_matches_by("name", "=", name.as_str())?;
        user.match_history = self.get_matches(user.id)?;
        user.badges = self.get_badges(user.id)?;
        Ok(user)
    }

    fn get_user_without_matches_by(&self, col: &str, comp: &str, val: &str) -> ServerResult<User>
    {
        let mut stmt = self.conn.prepare(&format!(
            "select id, name, elo, user_role from users where {} {} :val",
            col, comp
        ))?;
        let mut users = stmt.query_map_named(named_params! {":val": val}, |row| {
            Ok(User {
                id:            row.get(0)?,
                name:          row.get(1)?,
                elo:           row.get(2)?,
                user_role:     row.get(3)?,
                match_history: Vec::new(),
                badges:        Vec::new(),
            })
        })?;

        match users.next()
        {
            Some(Ok(user)) => Ok(user),
            Some(Err(_)) => Err(ServerError::Critical("???".into())), /* What does this even */
            // mean 🤷‍♀️
            None => Err(ServerError::UserNotExist),
        }
    }

    #[allow(dead_code)]
    fn get_default_password() -> String
    {
        use std::{
            fs::File,
            io::{prelude::*, BufReader},
        };

        use sha2::{Digest, Sha256};

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
    use rusqlite::NO_PARAMS;

    use super::*;
    use crate::{
        test_util::*,
        user::{
            USER_ROLE_INACTIVE, USER_ROLE_REGULAR, USER_ROLE_SOFT_INACTIVE, USER_ROLE_SUPERUSER,
        },
    };


    #[test]
    fn test_register_user_creates_notification()
    {
        let db_file = "temp0.db";
        let s = DataBase::new(db_file);

        let admin = "admin".to_string();
        let admin_token = create_user(&s, &admin);
        s.make_user_admin(admin.clone()).unwrap();
        s.set_variable(admin_token, "user_conf".to_string(), 1).expect("set variable");

        let markus = "markus".to_string();
        s.create_user(markus.clone(), "password".to_string()).unwrap();

        let mut stmt = s.conn.prepare("select name from new_user_notification;").unwrap();
        let user_notification_name = stmt
            .query_map(NO_PARAMS, |row| {
                let name: String = row.get(0).expect("Getting first (and only) row");
                Ok(name)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();

        std::fs::remove_file(db_file).expect("Removing file temp0");
        assert_eq!(user_notification_name, markus);
    }

    #[test]
    fn test_register_user_can_accept_user()
    {
        let db_file = "temp01.db";
        let s = DataBase::new(db_file);

        let admin = "admin".to_string();
        let admin_token = create_user(&s, &admin);
        s.make_user_admin(admin.clone()).unwrap();
        s.set_variable(admin_token.to_string(), "user_conf".to_string(), 1).unwrap();

        let markus = "markus".to_string();
        s.create_user(markus.clone(), "password".to_string()).unwrap();

        let id = s
            .get_admin_notifications(admin_token.clone())
            .unwrap()
            .get("new_users")
            .unwrap()[0]
            .id;
        let answer =
            AdminNotificationAns {
                id: id, token: admin_token.clone(), ans: ACCEPT_REQUEST
            };
        s.respond_to_new_user(answer).unwrap();

        std::fs::remove_file(db_file).expect("Removing file temp01");
        assert_eq!(
            s.get_admin_notifications(admin_token.clone())
                .unwrap()
                .get("new_users")
                .unwrap()
                .len(),
            0
        );
        assert!(s.get_user_without_matches(&markus).is_ok());
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

        let mut stmt =
            s.conn.prepare("select * from match_notification").expect("creating statement");

        let find = stmt.query_map(NO_PARAMS, |row| {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        assert!(find.expect("unwrapping quey map").next().is_none());

        let mut stmn = s.conn.prepare("select COUNT(*) from matches").expect("creating statement");

        let find = stmn.query_map(NO_PARAMS, |row| {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp2");
        assert!(find.unwrap().next().unwrap().unwrap() == 1);

        let (sivert, lars) = (
            s.get_user_without_matches(&"Sivert".to_string()).unwrap(),
            s.get_user_without_matches(&"Lars".to_string()).unwrap(),
        );
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


        s.register_match(winner.clone(), loser.clone(), uuid.clone())
            .expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), uuid).expect("Creating match");


        let mut stmt = s
            .conn
            .prepare("select winner_accept from match_notification where id = 1")
            .unwrap();
        let winner_accept = stmt
            .query_map(NO_PARAMS, |row| {
                let c: u8 = row.get(0).expect("Getting first (and only) row");
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();



        let mut stmt = s
            .conn
            .prepare("select loser_accept from match_notification where id = 2")
            .unwrap();
        let loser_accept = stmt
            .query_map(NO_PARAMS, |row| {
                let c: u8 = row.get(0).expect("Getting first (and only) row");
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();


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

        s.register_match(winner.clone(), loser.clone(), winner_uuid)
            .expect("Creating match");
        s.register_match(winner.clone(), loser.clone(), loser_uuid)
            .expect("Creating match");


        let mut stmt = s
            .conn
            .prepare("select winner_accept from match_notification where id = 1")
            .unwrap();
        let winner_accept = stmt
            .query_map(NO_PARAMS, |row| {
                let c: u8 = row.get(0).expect("Getting first (and only) row");
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();



        let mut stmt = s
            .conn
            .prepare("select loser_accept from match_notification where id = 2")
            .unwrap();
        let loser_accept = stmt
            .query_map(NO_PARAMS, |row| {
                let c: u8 = row.get(0).expect("Getting first (and only) row");
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();


        std::fs::remove_file(db_file).expect("Removing file tempA");
        assert!(winner_accept == ACCEPT_REQUEST);
        assert!(loser_accept == ACCEPT_REQUEST);
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


        let mut stmt =
            s.conn.prepare("select * from match_notification").expect("creating statement");

        let find = stmt.query_map(NO_PARAMS, |row| {
            let c: i64 = row.get(1).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp2");
        assert!(find.unwrap().next().unwrap().unwrap() == 1);
    }

    #[test]
    fn test_can_register_match()
    {
        let db_file = "temp3.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();

        s.register_match(winner, loser, uuid).expect("Creating match");

        let mut stmn = s
            .conn
            .prepare("select COUNT(*) from match_notification")
            .expect("creating statement");

        let find = stmn.query_map(NO_PARAMS, |row| {
            let c: i64 = row.get(0).expect("getting first row");
            Ok(c)
        });

        std::fs::remove_file(db_file).expect("Removing file temp");
        assert!(find.unwrap().next().unwrap().unwrap() == 1);
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
        let (name, password, new) =
            ("Sivert".to_string(), "password".to_string(), "new".to_string());
        create_user(&s, &name);
        s.change_password(name.clone(), password.clone(), new.clone())
            .expect("Changing password");

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

        s.register_match(lars.clone(), siv.clone(), token_siv.clone())
            .expect("Creating match");
        s.register_match(siv.clone(), lars.clone(), token_siv.clone())
            .expect("Creating match");
        s.register_match(siv.clone(), lars.clone(), token_siv.clone())
            .expect("Creating match");
        s.register_match(siv.clone(), lars.clone(), token_siv.clone())
            .expect("Creating match");
        s.register_match(bernt.clone(), siv.clone(), token_siv.clone())
            .expect("Creating match");


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
        s.roll_back(-1).expect("Rolling back");
        let (siv_user, lars_user, bernt_user) = (
            s.get_user_without_matches(&siv).unwrap(),
            s.get_user_without_matches(&lars).unwrap(),
            s.get_user_without_matches(&bernt).unwrap(),
        );

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

    #[test]
    fn test_make_user_admin_and_regular()
    {
        let db_file = "tempF.db";
        let s = DataBase::new(db_file);

        let mark = "markus".to_string();
        create_user(&s, mark.as_str());

        let user_init = s.get_user_without_matches(&mark).unwrap();

        s.make_user_admin(mark.clone()).unwrap();
        let user_admin = s.get_user_without_matches(&mark).unwrap();

        s.make_user_regular(mark.clone()).unwrap();
        let user_regular = s.get_user_without_matches(&mark).unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempE");
        assert_eq!(user_init.user_role, USER_ROLE_REGULAR | USER_ROLE_SOFT_INACTIVE);
        assert_eq!(user_admin.user_role, USER_ROLE_SUPERUSER | USER_ROLE_SOFT_INACTIVE);
        assert_eq!(user_regular.user_role, USER_ROLE_REGULAR | USER_ROLE_SOFT_INACTIVE);
    }

    #[test]
    fn test_make_user_active_and_inactive()
    {
        let db_file = "tempG.db";
        let s = DataBase::new(db_file);

        let mark = "markus".to_string();
        create_user(&s, mark.as_str());

        let user_init = s.get_user_without_matches(&mark).unwrap();

        s.make_user_inactive(mark.clone()).unwrap();
        let user_inactive = s.get_user_without_matches(&mark).unwrap();

        s.make_user_active(mark.clone()).unwrap();
        let user_active = s.get_user_without_matches(&mark).unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempG");
        assert_eq!(user_init.user_role, USER_ROLE_REGULAR | USER_ROLE_SOFT_INACTIVE);
        assert_eq!(
            user_inactive.user_role,
            USER_ROLE_REGULAR | USER_ROLE_INACTIVE | USER_ROLE_SOFT_INACTIVE
        );
        assert_eq!(user_active.user_role, USER_ROLE_REGULAR);
    }

    #[test]
    fn test_inactive_users_are_not_retrieved_when_calling_get_users()
    {
        let db_file = "tempGG.db";
        let s = DataBase::new(db_file);

        let mark = "markus".to_string();
        let sv = "Sivert".to_string();
        create_user(&s, mark.as_str());
        create_user(&s, sv.as_str());
        create_user(&s, "Bernt");

        let users1 = s.get_users().expect("Getting users");

        s.make_user_active(mark.clone()).unwrap();
        s.make_user_active(sv.clone()).unwrap();
        s.make_user_active("Bernt".to_string()).unwrap();

        let users2 = s.get_users().expect("Getting users");

        s.make_user_inactive(mark.clone()).expect("Making user inactive");
        s.make_user_soft_inactive(sv).expect("Making user soft inactive");

        let users3 = s.get_users().expect("Getting users2");
        std::fs::remove_file(db_file).expect("Removing file tempG");

        assert_eq!(users1.len(), 0);
        assert_eq!(users2.len(), 3);
        assert_eq!(users3.len(), 1);
    }

    #[test]
    fn test_wrong_change_password()
    {
        let db_file = "tempH.db";
        let s = DataBase::new(db_file);

        let markus = "Markus".to_string();
        create_user(&s, &markus);
        let res =
            s.change_password(markus.clone(), "Wrong".to_string(), ":woman_shrug:".to_string());

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(res.is_err());
    }

    #[test]
    fn test_can_edit_matches()
    {
        let db_file = "tempI.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        s.make_user_admin("Sivert".to_string()).expect("Making admin");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner.clone(), loser.clone(), uuid.clone())
            .expect("Creating match");
        respond_to_match(&s, "Lars", 1);


        let info = NewEditMatchInfo {
            token:  uuid,
            winner: loser.clone(),
            loser:  winner.clone(),
            epoch:  15,
            id:     1,
        };

        s.edit_match(info).expect("Editing match");

        let m = &s.get_all_matches().unwrap()[0];
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(m.epoch, 15);
        assert_eq!(m.winner, loser);
        assert_eq!(m.loser, winner);
    }

    #[test]
    fn test_can_delete_match()
    {
        let db_file = "tempJ.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        s.make_user_admin("Sivert".to_string()).expect("Making admin");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner.clone(), loser.clone(), uuid.clone())
            .expect("Creating match");
        respond_to_match(&s, "Lars", 1);


        let info = DeleteMatchInfo {
            token: uuid, id: 1
        };

        s.delete_match(info).expect("delete match");

        let m = &s.get_all_matches().unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn test_can_get_multiple_users()
    {
        let db_file = "tempJ1.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");
        create_user(&s, "Lars");
        create_user(&s, "Bernt");

        let vec = vec![2, 3];
        let users = s.get_multiple_users(vec.clone()).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(users[0].id, vec[0]);
        assert_eq!(users[1].id, vec[1]);
    }

    #[test]
    fn test_user_can_rename()
    {
        let db_file = "tempJ2.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");
        s.set_new_name(1, "Markus".to_string()).unwrap();
        s.get_user(&"Markus".to_string()).unwrap();
        let sivert = s.get_user(&"Sivert".to_string());

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(sivert.is_err());
    }

    #[test]
    fn test_user_can_request_new_name()
    {
        let db_file = "tempJ3.db";
        let s = DataBase::new(db_file);
        let token = create_user(&s, "Sivert");
        s.request_new_name(token, "Markus".to_string()).unwrap();

        let count: i64 = s
            .conn
            .query_row("select count(*) from new_name_notification", NO_PARAMS, |row| row.get(0))
            .unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_admin_can_accept_new_username()
    {
        let db_file = "tempJ4.db";
        let s = DataBase::new(db_file);

        let admin = "admin".to_string();
        let admin_token = create_user(&s, &admin);
        s.make_user_admin(admin.clone()).unwrap();

        s.request_new_name(admin_token.clone(), "not_admin".to_string()).unwrap();
        let not =
            AdminNotificationAns {
                ans: ACCEPT_REQUEST, id: 1, token: admin_token.clone()
            };

        s.respond_to_new_name_notification(not).unwrap();

        let user = s.get_user(&"not_admin".to_string());
        let count: i64 = s
            .conn
            .query_row("select count(*) from new_name_notification", NO_PARAMS, |row| row.get(0))
            .unwrap();

        std::fs::remove_file(db_file).expect("Removing file temp0");
        assert!(user.is_ok());
        assert_eq!(count, 0);
    }
}
