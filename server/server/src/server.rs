use std::{collections::HashMap, str::FromStr};

use chrono::prelude::*;
use elo::EloRank;
use lazy_static::lazy_static;
use regex::Regex;
use rusqlite::{named_params, params, Connection, ToSql, NO_PARAMS};
use server_core::{constants::*, types::*};
use uuid::Uuid;

use super::{
    _named_params,
    badge::*,
    r#match::{DeleteMatchInfo, EditMatchInfo, Match, NewEditMatchInfo},
    notification::{
        AdminNotification, MatchNotification, MatchNotificationTable, Notification,
        NotificationAns, NotificationType,
    },
    user::{StatsUsers, User},
    GET_OR_CREATE_DB_VAR, SQL_TUPLE_NAMED,
};


pub enum ParamsType<'a>
{
    Named(&'a [(&'a str, &'a dyn ToSql)]),
    Params(&'a [&'a dyn ToSql]),
}

pub type Params<'a> = Option<ParamsType<'a>>;


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
        let mut info = SQL_TUPLE_NAMED!(
            self,
            "select password_hash, uuid, user_role from users where name = :name;",
            named_params! {":name" : name},
            String,
            String,
            u8
        )?;

        if info.len() == 0
        {
            let count = SQL_TUPLE_NAMED!(
                self,
                "select count(*) from new_user_notification where name = :name",
                named_params! {":name": &name},
                i64
            )?;
            if let Some((c, ..)) = count.get(0)
            {
                if c == &1
                {
                    return Err(ServerError::WaitingForAdmin);
                }
            }

            return Err(ServerError::UserNotExist);
        }

        let (p, u, r) = info.pop().expect("Getting user stuff");

        if r & USER_ROLE_INACTIVE == USER_ROLE_INACTIVE
        {
            return Err(ServerError::InactiveUser);
        }

        if self.hash(&password) == p
        {
            return Ok(u);
        }

        return Err(ServerError::WrongUsernameOrPassword);
    }

    pub fn respond_to_match(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let sql = "select id, winner_accept, loser_accept, epoch, winner, loser
                  from match_notification where id = :id";

        let mut match_notification: MatchNotificationTable =
            self.sql_one(sql, _named_params! {":id": id})?;

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
        let re = Regex::new(r"^[a-zåA-ZæøåÆØÅ0-9_-]*$").unwrap();
        if !re.is_match(&new_user) || new_user.is_empty()
        {
            return Err(ServerError::InvalidUsername);
        }

        if !self.check_unique_name(&new_user)?
        {
            return Err(ServerError::UsernameTaken);
        }

        if self.get_variable("user_conf".to_string())? == 1
        {
            self.create_new_user_notification(new_user, password)?;
            Ok(" Now you have to wait for an admin to accept..!".to_string())
        }
        else
        {
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

    pub fn get_user(&self, name: &String) -> ServerResult<User>
    {
        let mut user = self.get_user_without_matches_by("name", "=", name.as_str())?;
        user.match_history = self.get_matches(user.id)?;
        user.badges = self.get_badges(user.id)?;
        Ok(user)
    }

    pub fn get_all_users(&self, token: String) -> ServerResult<Vec<User>>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }


        let sql = "select * from users order by elo";
        self.sql_many(sql, None)
    }

    pub fn get_non_inactive_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_users_with_user_role(USER_ROLE_INACTIVE, 0)
    }

    pub fn get_users(&self) -> ServerResult<Vec<User>>
    {
        self.get_users_with_user_role(USER_ROLE_INACTIVE | USER_ROLE_SOFT_INACTIVE, 0)
    }

    pub fn get_multiple_users(&self, users: Vec<String>) -> ServerResult<Vec<User>>
    {
        let list = format!("{:?}", users).as_str().replace("[", "(").replace("]", ")");
        let sql = format!("select id, name, elo, user_role from users where name in {}", list);

        let mut users = self.sql_many::<User, _>(sql, None)?;
        for user in &mut users
        {
            user.match_history = self.get_matches(user.id)?;
        }
        Ok(users)
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
        let current_season = self.get_latest_season_number()?;
        let sql = format!(
            "select a.name as winner, b.name as loser, elo_diff, winner_elo, loser_elo, epoch, {} \
             as season from matches
             inner join users as a on a.id = winner
             inner join users as b on b.id = loser
             order by epoch desc;",
            current_season
        );

        self.sql_many(sql, None)
    }

    pub fn get_stats(&self, info: StatsUsers) -> ServerResult<HashMap<String, Vec<Match>>>
    {
        let user1_id = self.get_user_without_matches(&info.user1)?.id;
        let user2_id = self.get_user_without_matches(&info.user2)?.id;

        let current_season = self.get_latest_season_number()?;
        let current = self.get_stats_from_table(
            format!("{} as season from matches", current_season),
            user1_id,
            user2_id,
        )?;
        let rest =
            self.get_stats_from_table("season from old_matches".to_string(), user1_id, user2_id)?;

        let mut map = HashMap::new();
        map.insert("current".to_string(), current);
        map.insert("rest".to_string(), rest);
        Ok(map)
    }

    fn get_stats_from_table(
        &self,
        table: String,
        user1_id: i64,
        user2_id: i64,
    ) -> ServerResult<Vec<Match>>
    {
        let sql = &format!(
            "select u1.name as winner, u2.name as loser, elo_diff, winner_elo, loser_elo, epoch, \
             {} t
             join users u1 on t.winner = u1.id
             join users u2 on t.loser = u2.id
             where t.winner = :user1 and t.loser = :user2
             union
             select u2.name as winner, u1.name as loser, elo_diff, winner_elo, loser_elo, epoch, \
             {} t
             join users u1 on t.loser = u1.id
             join users u2 on t.winner = u2.id
             where t.winner = :user2 and t.loser = :user1;",
            table, table
        );
        self.sql_many(sql, _named_params! {":user1": user1_id, ":user2": user2_id})
    }

    pub fn get_edit_match_history(&self) -> ServerResult<Vec<EditMatchInfo>>
    {
        let sql = "select a.name as winner, b.name as loser, epoch, m.id as id from matches as m
             inner join users as a on a.id = winner
             inner join users as b on b.id = loser
             order by epoch;";

        self.sql_many(sql, None)
    }

    pub fn change_password(
        &self,
        name: String,
        password: String,
        new_password: String,
    ) -> ServerResult<()>
    {
        let sql = "select id, password_hash from users where name = :name;";
        let mut info = SQL_TUPLE_NAMED!(self, sql, named_params! {":name" : name}, i64, String)?;
        if info.len() == 0
        {
            return Err(ServerError::UserNotExist);
        }

        let (id, p) = info.pop().unwrap();
        if self.hash(&password) == p
        {
            return self.update_password(id, new_password);
        }

        return Err(ServerError::PasswordNotMatch);
    }

    pub fn request_reset_password(&self, name: String) -> ServerResult<()>
    {
        let sql = "select user from reset_password_notification where user = :id;";
        let user_id = self.get_user_without_matches(&name)?.id;
        let params = named_params! {":id": user_id};
        let notification = SQL_TUPLE_NAMED!(self, sql, params, i64)?;
        if notification.len() > 0
        {
            return Err(ServerError::ResetPasswordDuplicate);
        }

        self.conn
            .execute_named("insert into reset_password_notification (user) values (:id)", params)?;

        Ok(())
    }

    pub fn get_match_notifications(&self, token: String) -> ServerResult<Vec<MatchNotification>>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let sql = "select * from
            (select m.id, u1.name as winner, u2.name as loser, epoch from match_notification m
            join users u1 on m.winner = u1.id
            join users u2 on m.loser = u2.id
            where m.winner = :id and m.winner_accept = 0
            union
            select m.id, u1.name as winner, u2.name as loser, epoch from match_notification m
            join users u1 on m.winner = u1.id
            join users u2 on m.loser = u2.id
            where m.loser = :id and m.loser_accept = 0)
            order by epoch";

        self.sql_many(sql, _named_params! {":id": user.id})
    }

    pub fn get_admin_notifications(
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

        let mut map = HashMap::new();
        map.insert("new_users".to_string(), new_user);
        map.insert("reset_password".to_string(), reset_password);
        Ok(map)
    }

    pub fn respond_to_new_user(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        if !self.get_is_admin(token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        // No match is being accepted, but the ans values are the same Xdd
        if ans == ACCEPT_REQUEST
        {
            self.create_user_from_notification(id)?;
        }
        self.delete_new_user_notification(id)?;
        Ok(())
    }

    pub fn respond_to_reset_password(&self, id: i64, ans: u8, token: String) -> ServerResult<()>
    {
        if !self.get_is_admin(token.clone())?
        {
            return Err(ServerError::Unauthorized);
        }

        if ans == ACCEPT_REQUEST
        {
            self.reset_password(id)?;
        }
        self.delete_reset_password_notification(id)?;
        Ok(())
    }

    pub fn sql_one<T, S>(&self, s: S, params: Params) -> ServerResult<T>
    where
        S: AsRef<str>,
        T: FromSql,
    {
        let mut stmt = self.conn.prepare(s.as_ref())?;
        let res = match params
        {
            None => stmt.query_map(NO_PARAMS, T::from_sql)?.next(),
            Some(ParamsType::Named(n)) => stmt.query_map_named(n, T::from_sql)?.next(),
            Some(ParamsType::Params(p)) => stmt.query_map(p, T::from_sql)?.next(),
        };
        match res
        {
            Some(Ok(res)) => Ok(res),
            Some(Err(e)) => Err(ServerError::Rusqlite(e)),
            None => Err(ServerError::Critical("Could not find any rows".to_string())),
        }
    }

    pub fn sql_many<T, S>(&self, s: S, params: Params) -> ServerResult<Vec<T>>
    where
        S: AsRef<str>,
        T: FromSql,
    {
        let mut stmt = self.conn.prepare(s.as_ref())?;
        let result = match params
        {
            None => stmt.query_map(NO_PARAMS, T::from_sql)?,
            Some(ParamsType::Named(n)) => stmt.query_map_named(n, T::from_sql)?,
            Some(ParamsType::Params(p)) => stmt.query_map(p, T::from_sql)?,
        };

        let mut vec = Vec::new();
        for elem in result
        {
            if let Ok(r) = elem
            {
                vec.push(r);
            };
        }
        Ok(vec)
    }

    pub fn get_notifications(
        &self,
        t: NotificationType,
        token: String,
    ) -> ServerResult<Notification>
    {
        match t
        {
            NotificationType::Admin =>
            {
                Ok(Notification::Admin(self.get_admin_notifications(token)?))
            },
            NotificationType::Match =>
            {
                Ok(Notification::Match(self.get_match_notifications(token)?))
            },
        }
    }

    pub fn respond_to_notification(&self, not: NotificationAns) -> ServerResult<()>
    {
        match not
        {
            NotificationAns::Match(id, token, ans) => self.respond_to_match(id, ans, token),
            NotificationAns::NewUser(id, token, ans) => self.respond_to_new_user(id, ans, token),
            NotificationAns::ResetPassword(id, token, ans) =>
            {
                self.respond_to_reset_password(id, ans, token)
            },
        }
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
        self.conn.execute(
            "update users set user_role = user_role & (~?1 & ~?2) where name = ?3",
            params![USER_ROLE_SOFT_INACTIVE, USER_ROLE_INACTIVE, name],
        )?;
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
        let sql = "select
             (select count(*) from users where name = :name)+
             (select count(*) from new_user_notification where name = :name);
        ";

        let count = SQL_TUPLE_NAMED!(self, sql, named_params! {":name": name}, i64)?;
        if count.len() == 0
        {
            return Err(ServerError::Critical("Could not count tables".into()));
        }

        Ok(count[0].0 == 0)
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
        let user = SQL_TUPLE_NAMED!(
            self,
            "select name, password_hash from new_user_notification where id = :id",
            named_params! {":id": id},
            String,
            String
        )?;

        if user.len() == 0
        {
            return Err(ServerError::Critical("err".into())); // Should not be possible to reach this
        }
        self.create_user_with_password_hash(user[0].0.clone(), user[0].1.clone())
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

    fn try_get_new_user_notifications(&self) -> ServerResult<Vec<AdminNotification>>
    {
        self.sql_many("select * from new_user_notification", None)
    }

    fn try_get_reset_password_notifications(&self) -> ServerResult<Vec<AdminNotification>>
    {
        let sql = "select n.id, name from reset_password_notification as n
                  join users as a on a.id = n.user";
        self.sql_many(sql, None)
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
            self.make_user_active(winner.name.clone())?;
            self.make_user_active(loser.name.clone())?;
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
        let sql = "select count(*) from matches where epoch > :epoch";
        let count = SQL_TUPLE_NAMED!(self, sql, named_params! {":epoch": epoch}, i64)?;
        Ok(count[0].0 > 0)
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
        let sql = "select count(*) from users where id = :id and uuid = :token";
        let c = SQL_TUPLE_NAMED!(self, sql, named_params! {":id": user_id, ":token": token}, i64)?;
        Ok(c[0].0 == 1)
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
        let sql = "select user from reset_password_notification where id = :id;";
        let user_id = SQL_TUPLE_NAMED!(self, sql, named_params! {":id" : id}, i64)?;
        if user_id.len() == 0
        {
            return Err(ServerError::Critical("Something went wrong".into())); // Should not be possible to reach this
        }
        let user_id = user_id[0].0;
        let hash = self.hash(&"@uit".to_string());
        self.conn.execute_named(
            "update users set password_hash = :hash where id = :id",
            named_params! {":hash": hash, ":id": user_id},
        )?;
        Ok(())
    }

    fn hash(&self, word: &String) -> String
    {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(word);
        let result = hasher.finalize();
        format!("{:x}", result)
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
        let sql = format!(
            "select a.name as winner, b.name as loser, elo_diff, winner_elo, loser_elo, epoch, {} \
             as season
                from matches
                inner join users as a on a.id = winner
                inner join users as b on b.id = loser
                where winner = :id or loser = :id
                order by epoch desc",
            current_season
        );
        self.sql_many(sql, _named_params! {":id" : id})
    }

    //@TODO: Hmm, name is an issue here. Store the names in a table maybe to fix?
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
        let sql = "select id, name, elo, user_role from users
             where user_role & :user_role = :val
             order by elo desc";
        let mut users: Vec<User> =
            self.sql_many(sql, _named_params! {":user_role": user_role, ":val": val})?;
        for user in &mut users
        {
            user.badges = self.get_badges(user.id)?;
        }
        Ok(users)
    }

    fn get_user_without_matches(&self, name: &String) -> ServerResult<User>
    {
        self.get_user_without_matches_by("name", "=", name.as_str())
    }

    fn get_user_without_matches_by(&self, col: &str, comp: &str, val: &str) -> ServerResult<User>
    {
        let sql =
            &format!("select id, name, elo, user_role from users where {} {} :val", col, comp);


        self.sql_one(sql, _named_params! {":val": val}).map_err(|e| match e
        {
            ServerError::Critical(_) => ServerError::UserNotExist,
            _ => e,
        })
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

    pub fn epoch(&self) -> i64
    {
        Utc::now().timestamp_millis()
    }
}


#[cfg(test)]
mod test
{
    use rusqlite::NO_PARAMS;
    use server_core::constants::{
        USER_ROLE_INACTIVE, USER_ROLE_REGULAR, USER_ROLE_SOFT_INACTIVE, USER_ROLE_SUPERUSER,
    };

    use super::*;
    use crate::{test_util::*, SQL_TUPLE};


    #[test]
    fn test_register_user_creates_notification() -> ServerResult<()>
    {
        let db_file = "temp0.db";
        let s = DataBase::new(db_file);

        let admin = "admin".to_string();
        let admin_token = create_user(&s, &admin);
        s.make_user_admin(admin.clone()).unwrap();
        s.set_variable(admin_token, "user_conf".to_string(), 1).expect("set variable");

        let markus = "markus".to_string();
        s.create_user(markus.clone(), "password".to_string()).unwrap();

        let user_notification_name =
            SQL_TUPLE!(s, "select name from new_user_notification;", String).unwrap();

        std::fs::remove_file(db_file).expect("Removing file temp0");
        assert_eq!(user_notification_name[0].0, markus);
        Ok(())
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

        s.respond_to_new_user(id, ACCEPT_REQUEST, admin_token.clone()).unwrap();

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
    fn test_match_notification_both_accepted() -> ServerResult<()>
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

        let find1 = SQL_TUPLE!(s, "select * from match_notification", i64)?;
        let find2 = SQL_TUPLE!(s, "select COUNT(*) from matches", i64)?;

        std::fs::remove_file(db_file).expect("Removing file temp2");

        assert!(find1.len() == 0);
        assert!(find2[0].0 == 1);
        let (sivert, lars) = (
            s.get_user_without_matches(&"Sivert".to_string()).unwrap(),
            s.get_user_without_matches(&"Lars".to_string()).unwrap(),
        );

        assert!(sivert.elo > 1500.0);
        assert!(lars.elo < 1500.0);
        Ok(())
    }

    #[test]
    fn test_match_registered_by_none_participant_gets_answered_no() -> ServerResult<()>
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

        let winner_accept =
            SQL_TUPLE!(s, "select winner_accept from match_notification where id = 1", u8)?;
        let loser_accept =
            SQL_TUPLE!(s, "select loser_accept from match_notification where id = 2", u8)?;

        std::fs::remove_file(db_file).expect("Removing file tempA");
        assert!(winner_accept[0].0 == MATCH_NO_ANS);
        assert!(loser_accept[0].0 == MATCH_NO_ANS);
        Ok(())
    }

    #[test]
    fn test_match_registered_by_participant_gets_answered_yes() -> ServerResult<()>
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

        let winner_accept =
            SQL_TUPLE!(s, "select winner_accept from match_notification where id = 1", u8)?;
        let loser_accept =
            SQL_TUPLE!(s, "select loser_accept from match_notification where id = 2", u8)?;

        std::fs::remove_file(db_file).expect("Removing file tempA");
        assert!(winner_accept[0].0 == ACCEPT_REQUEST);
        assert!(loser_accept[0].0 == ACCEPT_REQUEST);
        Ok(())
    }

    #[test]
    fn test_can_respond_to_match() -> ServerResult<()>
    {
        let db_file = "temp2.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();

        s.register_match(winner, loser, uuid).expect("Creating match");
        respond_to_match(&s, "Sivert", 1);

        let find = SQL_TUPLE!(s, "select * from match_notification", i64)?;

        std::fs::remove_file(db_file).expect("Removing file temp2");
        assert!(find[0].0 == 1);
        Ok(())
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
        assert!(uuid.is_ok());
        assert!(uuid.unwrap().len() == 36);
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
    fn edit_matches_in_correct_order()
    {
        let db_file = "tempI1.db";
        let s = DataBase::new(db_file);
        let uuid = create_user(&s, "Sivert");
        s.make_user_admin("Sivert".to_string()).expect("Making admin");
        create_user(&s, "Lars");

        let winner = "Sivert".to_string();
        let loser = "Lars".to_string();


        s.register_match(winner.clone(), loser.clone(), uuid.clone())
            .expect("Creating match");
        respond_to_match(&s, "Lars", 1);

        s.register_match(loser.clone(), winner.clone(), uuid.clone())
            .expect("Creating match");
        respond_to_match(&s, "Lars", 2);

        let vec = s.get_edit_match_history().unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(vec.get(0).unwrap().winner, loser);
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

        let m = &s.get_history().unwrap()[0];
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

        let m = &s.get_history().unwrap();
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

        let vec = vec!["Lars".to_string(), "Bernt".to_string()];
        let users = s.get_multiple_users(vec.clone()).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        users.into_iter().for_each(|u| assert!(vec.contains(&u.name)));
    }

    #[test]
    fn test_reset_password_creates_notification()
    {
        let db_file = "tempJ2.db";
        let s = DataBase::new(db_file);
        let user = "Sivert".to_string();
        create_user(&s, user.as_str());

        let res = s.request_reset_password(user.clone());
        let res2 = s.request_reset_password(user.clone());

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res.is_ok());
        assert!(res2.is_err());
    }

    #[test]
    fn test_reset_password_can_reset()
    {
        let db_file = "tempJ3.db";
        let s = DataBase::new(db_file);
        let user = "Sivert".to_string();
        let token = create_user(&s, user.as_str());
        s.make_user_admin(user.clone()).unwrap();
        s.request_reset_password(user.clone()).unwrap();

        let not = NotificationAns::ResetPassword(1, token, ACCEPT_REQUEST);
        s.respond_to_notification(not).unwrap();
        let res = s.login(user, "@uit".to_string());

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res.is_ok());
    }

    #[test]
    fn test_get_stats_between_users()
    {
        let db_file = "tempJ4.db";
        let s = DataBase::new(db_file);
        let user1 = "Sivert".to_string();
        let user2 = "Markus".to_string();
        let token1 = create_user(&s, user1.as_str());
        let token2 = create_user(&s, user2.as_str());

        let user1_wins = 3;
        let user2_wins = 1;

        s.start_new_season().unwrap();

        s.register_match(user1.clone(), user2.clone(), token1.clone()).unwrap();
        s.respond_to_match(1, ACCEPT_REQUEST, token2.clone()).unwrap();

        s.end_season().unwrap();
        s.start_new_season().unwrap();


        s.register_match(user1.clone(), user2.clone(), token1.clone()).unwrap();
        s.respond_to_match(2, ACCEPT_REQUEST, token2.clone()).unwrap();

        s.register_match(user1.clone(), user2.clone(), token1.clone()).unwrap();
        s.respond_to_match(3, ACCEPT_REQUEST, token2.clone()).unwrap();

        s.register_match(user2.clone(), user1.clone(), token1.clone()).unwrap();
        s.respond_to_match(4, ACCEPT_REQUEST, token2.clone()).unwrap();

        let info = StatsUsers {
            user1: user1.clone(), user2: user2.clone()
        };

        let stats = s.get_stats(info).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");

        let current = stats.get("current").unwrap();
        let rest = stats.get("rest").unwrap();

        let mut map = HashMap::new();
        for ma in current.into_iter().chain(rest.into_iter())
        {
            *map.entry(&ma.winner).or_insert(0) += 1;
        }
        assert_eq!(*map.get(&user1).unwrap(), user1_wins);
        assert_eq!(*map.get(&user2).unwrap(), user2_wins);
    }
}
