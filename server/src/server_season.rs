use chrono::prelude::*;
use rusqlite::{named_params, params, NO_PARAMS};

use crate::{
    GET_OR_CREATE_DB_VAR,
    badge::NUM_SEASON_PRIZES,
    season::Season,
    server::{DataBase, ServerError, ServerResult, N_SEASON_ID},
    user::USER_ROLE_SOFT_INACTIVE,
};

impl DataBase
{
    pub fn end_season(&self) -> ServerResult<()>
    {
        self._end_season()
    }

    pub fn start_new_season(&self, inactive: bool) -> ServerResult<()>
    {
        self._start_new_season(inactive)
    }

    pub fn get_season_length(&self) -> ServerResult<i32>
    {
        GET_OR_CREATE_DB_VAR!(&self.conn, N_SEASON_ID, 1)
    }

    pub fn set_season_length(&self, token: String, new_val: i32) -> ServerResult<()>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }
        // update variables set value = {} where id = {} !!!
        self.conn.execute("update variables set value = (?1) where id = (?2)", params![
            new_val,
            N_SEASON_ID
        ])?;
        Ok(())
    }
}


// ~ end season functions
impl DataBase
{
    fn _end_season(&self) -> ServerResult<()>
    {
        // Only award badges if there were a season
        if let Some(season) = self.get_latest_season()?
        {
            for (i, user) in self.get_users()?.into_iter().take(NUM_SEASON_PRIZES).enumerate()
            {
                self.award_badge(i as i64, season.id, user.id)?;
            }
        }

        Ok(())
    }

    fn award_badge(&self, badge_index: i64, season: i64, pid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into badges (season_id, badge_index, pid) values (?1, ?2, ?3)",
            params![season, badge_index, pid],
        )?;
        Ok(())
    }
}


// ~ start new season functions
impl DataBase
{
    fn _start_new_season(&self, inactive: bool) -> ServerResult<()>
    {
        let season_number = self.create_new_season()?;
        self.archive_match_history(season_number)?;
        self.reset_elos()?;
        if inactive
        {
            self.set_users_soft_inactive()?;
        }
        Ok(())
    }

    fn set_users_soft_inactive(&self) -> ServerResult<()>
    {
        self.conn.execute(
            &format!("update users set user_role = user_role | {}", USER_ROLE_SOFT_INACTIVE),
            NO_PARAMS,
        )?;

        Ok(())
    }

    fn reset_elos(&self) -> ServerResult<()>
    {
        self.conn.execute("update users set elo = 1500.0", NO_PARAMS)?;
        Ok(())
    }

    fn archive_match_history(&self, season_number: i64) -> ServerResult<()>
    {
        self.conn.execute(
            &format!("alter table matches rename to matches{}", season_number),
            NO_PARAMS,
        )?;

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
            NO_PARAMS,
        )?;

        Ok(())
    }

    fn create_new_season(&self) -> ServerResult<i64>
    {
        let old_season = self.get_latest_season()?.unwrap_or_else(|| {
            // First ever season!!
            Season {
                id: 0, start_epoch: -1
            }
        });

        self.conn.execute("insert into seasons (start_epoch) values (?1)", params![
            Utc::now().timestamp_millis()
        ])?;

        Ok(old_season.id)
    }
}

// ~ Common functions :thinking:
impl DataBase
{
    pub fn get_latest_season(&self) -> ServerResult<Option<Season>>
    {
        let mut stmt = self.conn.prepare("select id, start_epoch from seasons order by id desc")?;
        let mut seasons = stmt.query_map(NO_PARAMS, |row| {
            Ok(Season {
                id: row.get(0)?, start_epoch: row.get(1)?
            })
        })?;

        Ok(seasons.next().map(|s| s.unwrap()))
    }
}
