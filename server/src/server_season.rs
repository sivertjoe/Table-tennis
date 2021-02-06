use chrono::prelude::*;
use rusqlite::{named_params, params, NO_PARAMS};

use crate::{
    GET_OR_CREATE_DB_VAR,
    badge::NUM_SEASON_PRIZES,
    season::Season,
    server::{DataBase, ServerError, ServerResult},
    user::USER_ROLE_SOFT_INACTIVE,
};

pub const N_SEASON_ID: u32 = 1;
pub const IS_SEASON_ID: u32 = 2;

impl DataBase
{
    pub fn end_season(&self) -> ServerResult<()>
    {
        self._end_season()
    }

    pub fn start_new_season(&self) -> ServerResult<()>
    {
        self._start_new_season()
    }

    pub fn get_season_length(&self) -> ServerResult<i64>
    {
        GET_OR_CREATE_DB_VAR!(&self.conn, N_SEASON_ID, 1)
    }

    pub fn set_season_length(&self, token: String, new_val: i64) -> ServerResult<()>
    {
        if !self.get_is_admin(token)?
        {
            return Err(ServerError::Unauthorized);
        }

        self._set_season_length(new_val)?;
        Ok(())
    }

    pub fn get_is_season(&self) -> ServerResult<bool>
    {
        GET_OR_CREATE_DB_VAR!(&self.conn, IS_SEASON_ID, 1)
            .map(|num| num == 1)
    }

    pub fn set_is_season(&self, val: bool) -> ServerResult<()>
    {
        let new_val = if val { 1 } else { 0 };
        self.conn.execute("update variables set value = (?1) where id = (?2)", params![new_val, IS_SEASON_ID])?;
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
            if self.get_is_season()?
            {
                for (i, user) in self.get_users()?.into_iter().take(NUM_SEASON_PRIZES).enumerate()
                {
                    self.award_badge(i as i64, season.id, user.id)?;
                }
                self.archive_match_history(season.id)?;
                self.clear_matches()?;
                self.clear_notifications()?;
                self.reset_elos()?;
                self.set_users_soft_inactive()?;
                self.set_is_season(false)?;
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

    fn archive_match_history(&self, season_number: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into old_matches (epoch, elo_diff, winner_elo, loser_elo, winner, loser, season)
             select epoch, elo_diff, winner_elo, loser_elo, winner, loser, seasons.id
             from matches, seasons where seasons.id = (?1)",
             params![season_number])?;
        Ok(())
    }

}


// ~ start new season functions
impl DataBase
{
    fn _start_new_season(&self) -> ServerResult<()>
    {
        if !self.get_is_season()? || self.get_latest_season()?.is_none()
        {
            self.create_new_season()?;
            self.archive_offseason()?;
            self.set_users_soft_inactive()?;
            self.reset_elos()?;
            self.clear_notifications()?;
            self.clear_matches()?;
            self.set_is_season(true)?;
        }
        Ok(())
    }

    fn create_new_season(&self) -> ServerResult<()>
    {
        self.conn.execute("insert into seasons (start_epoch) values (?1)", params![
            Utc::now().timestamp_millis()
        ])?;
        Ok(())
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

    pub fn archive_offseason(&self) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into offseason_matches (epoch, elo_diff, winner_elo, loser_elo, winner, loser)
             select epoch, elo_diff, winner_elo, loser_elo, winner, loser from matches",
             NO_PARAMS,)?;
        Ok(())
    }

    pub fn clear_matches(&self) -> ServerResult<()>
    {
        self.conn.execute(
            "delete from matches",
            NO_PARAMS,
        )?;
        Ok(())
    }

    pub fn reset_elos(&self) -> ServerResult<()>
    {
        self.conn.execute("update users set elo = 1500.0", NO_PARAMS)?;
        Ok(())
    }

    pub fn clear_notifications(&self) -> ServerResult<()>
    {
        self.conn.execute("delete from match_notification", NO_PARAMS)?;
        Ok(())
    }

    pub fn set_users_soft_inactive(&self) -> ServerResult<()>
    {
        self.conn.execute(
            &format!("update users set user_role = user_role | {}", USER_ROLE_SOFT_INACTIVE),
            NO_PARAMS,
        )?;

        Ok(())
    }

    pub fn _set_season_length(&self, new_val: i64) -> ServerResult<()>
    {
        self.conn.execute("update variables set value = (?1) where id = (?2)", params![new_val, N_SEASON_ID])?; 
        Ok(())
    }
}


#[cfg(test)]
mod test
{
    use super::*;
    use crate::test_util::*;
    #[test]
    fn test_get_set_is_season()
    {
        let db_file = "tempGGG.db";
        let s = DataBase::new(db_file);
        let default = s.get_is_season().unwrap(); // Create it

        s.set_is_season(false).unwrap();
        let next = s.get_is_season().unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(default, true);
        assert_eq!(next, false);
    }

    #[test]
    fn test_get_set_season_length()
    {
        let db_file = "tempDDD.db";
        let s = DataBase::new(db_file);
        let default = s.get_season_length().unwrap(); // Create it

        s._set_season_length(2).unwrap();
        let next = s.get_season_length().unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(default, 1);
        assert_eq!(next, 2);
    }
}
