use crate::server::DataBase;
use crate::server::{ServerResult};
use rusqlite::{params, NO_PARAMS};
use crate::season::Season;
use chrono::prelude::*;
use crate::user::USER_ROLE_SOFT_INACTIVE;


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
}


// ~ end season functions
impl DataBase
{
    fn _end_season(&self) -> ServerResult<()>
    {
        // If there was a season before, could be that no seasons have occured
        if let Some(season) = self.archive_season_and_get_prev()?
        {
            for (i, user) in self.get_users()?.into_iter().take(4).enumerate()
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
            params![season, badge_index, pid],)?;
        Ok(())
    }

    fn archive_season_and_get_prev(&self) -> ServerResult<Option<Season>>
    {
        let mut stmt = self.conn.prepare(
            "select id, start_epoch from seasons order by id")?;
        let mut seasons = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(Season {
                id: row.get(0)?,
                start_epoch: row.get(1)?,
            })
        })?;

        let season = seasons.next();
        if season.is_none()
        {
            // This is the first ever season, and no awards should be awarded
            return Ok(None);
        }

        let old_season = season.unwrap().unwrap();
        Ok(Some(old_season))
    }
}


// ~ start new season functions
impl DataBase
{


    fn _start_new_season(&self) -> ServerResult<()>
    {
        let season_number = self.create_new_season()?;
        self.archive_match_history(season_number)?;
        self.reset_elos()?;
        self.set_users_soft_inactive()?;
        Ok(())
    }

    fn set_users_soft_inactive(&self) -> ServerResult<()>
    {
        self.conn.execute(
            &format!("update users set user_role = {}", USER_ROLE_SOFT_INACTIVE),
            NO_PARAMS)?;

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
            NO_PARAMS)?;

        self.conn.execute(
            &format!("create table matches as select * from matches{} where 0", season_number),
            NO_PARAMS)?;

        Ok(())
    }

    fn create_new_season(&self) -> ServerResult<i64>
    {
        let old_season = self.get_latest_season()?.unwrap_or_else(||
        {
            // First ever season!!
            let now = Utc::now();
            Season {
                id: 0,
                start_epoch: now.timestamp_millis(),
            }
        });

        self.conn.execute(
            "insert into seasons (start_epoch) values (?1)",
            params![old_season.start_epoch])?;
        
        Ok(old_season.id)
    }

    fn get_latest_season(&self) -> ServerResult<Option<Season>>
    {
        let mut stmt = self.conn.prepare(
            "select id, start_epoch from seasons order by id desc")?;
        let mut seasons = stmt.query_map(NO_PARAMS, |row|
        {
            Ok(Season {
                id: row.get(0)?,
                start_epoch: row.get(1)?,
            })
        })?;

        let season = seasons.next();
        if season.is_none()
        {
            /*let now = Utc::now();
            let m = now.month();
            let year = now.year();

            let num_days = 
            if m == 12 
            {
                NaiveDate::from_ymd(year + 1, 1, 1)
            } 
            else
            {
                NaiveDate::from_ymd(year, m + 1, 1)
            }.signed_duration_since(NaiveDate::from_ymd(year, m, 1))
            .num_days();

            // This is just a dummy season to create the first
            let mock_season = Season { 
                id: -1, 
                start_epoch: now.timestamp_millis() - (num_days * 24 * 60 * 60 * 1000), 
                end_epoch: now.timestamp_millis()
            };
            return Ok(mock_season);*/
            return Ok(None);
        }

        // Should be okay to unwrap here, there _has_ to be as seanson :thinking:
        let season = season.unwrap().unwrap();
        Ok(Some(season))
    }
}