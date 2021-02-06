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
    use crate::user::{
        USER_ROLE_SOFT_INACTIVE,
        USER_ROLE_SUPERUSER,
    };
    use crate::badge::*;

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

    #[test]
    fn test_match_notification_cleared_after_season_start_and_end()
    {
        let db_file = "tempDCCC.db";
        let s = DataBase::new(db_file);

        let siv = "Sivert".to_string();
        let lars = "Lars".to_string();

        let token_siv = create_user(&s, siv.as_str());
        create_user(&s, lars.as_str());

        s.start_new_season().unwrap();
        s.register_match(lars.clone(), siv.clone(), token_siv.clone()).expect("Creating match");
        let notification_count_before = get_table_size(&s, "match_notification");
        s.end_season().unwrap();
        let notification_count_after = get_table_size(&s, "match_notification");

        s.register_match(lars.clone(), siv.clone(), token_siv.clone()).expect("Creating match");
        let notification_count_before2 = get_table_size(&s, "match_notification");
        s.start_new_season().unwrap();
        let notification_count_after2 = get_table_size(&s, "match_notification");

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(notification_count_before == 1);
        assert!(notification_count_after == 0);
        assert!(notification_count_before2 == 1);
        assert!(notification_count_after2 == 0);
    }

    #[test]
    fn test_offseason_games()
    {
        let db_file = "tempCCC.db";
        let s = DataBase::new(db_file);

        let siv = "Sivert".to_string();
        let lars = "Lars".to_string();
        let bernt = "Bernt".to_string();

        let token_siv = create_user(&s, siv.as_str());
        create_user(&s, lars.as_str());
        create_user(&s, bernt.as_str());

        s.start_new_season().unwrap();
        s.register_match(lars.clone(), siv.clone(), token_siv.clone()).expect("Creating match");
        respond_to_match(&s, lars.as_str(), 1);
        s.end_season().unwrap();

        let badge_len = s.get_profile(lars.clone()).unwrap().badges.len();
        let old_matches_len = get_table_size(&s, "old_matches");
        let matches_len = get_table_size(&s, "matches");
        let offseason_matches = get_table_size(&s, "offseason_matches");

        // We are now in offseason
        s.register_match(siv.clone(), lars.clone(), token_siv.clone()).expect("Creating match");
        respond_to_match(&s, lars.as_str(), 2);

        s.start_new_season().unwrap();

        // Check that they were archived correctly
        let old_matches_len2 = get_table_size(&s, "old_matches");
        let matches_len2 = get_table_size(&s, "matches");
        let offseason_matches2 = get_table_size(&s, "offseason_matches");
        let badge_len2 = s.get_profile(lars.clone()).unwrap().badges.len();

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(badge_len, 1);
        assert_eq!(old_matches_len, 1);
        assert_eq!(matches_len, 0);
        assert_eq!(offseason_matches, 0);
        assert_eq!(old_matches_len2, 1);
        assert_eq!(matches_len2, 0);
        assert_eq!(offseason_matches2, 1);
        assert_eq!(badge_len2, 1);
    }

    #[test]
    fn test_creating_multiple_seasons()
    {
        let db_file = "tempMM.db";
        let s = DataBase::new(db_file);

        s.start_new_season().unwrap();
        s.end_season().unwrap();

        s.start_new_season().unwrap();
        s.end_season().unwrap();

        s.start_new_season().unwrap();
        s.end_season().unwrap();

        let mut stmt = s.conn.prepare("select count(*) from seasons").unwrap();
        let count = stmt
            .query_map(NO_PARAMS, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_start_new_season_creates_season_table_and_resets_users_and_history()
    {
        let db_file = "tempM.db";
        let s = DataBase::new(db_file);

        let mark = "Markus".to_string();
        let siv = "Sivert".to_string();
        let uuid_m = create_user(&s, mark.as_str());
        create_user(&s, siv.as_str());
        make_user_admin(&s, siv.clone()).expect("Making user admin");

        s.register_match(mark.clone(), siv.clone(), uuid_m).unwrap();
        respond_to_match(&s, siv.as_str(), 1);

        let (s_elo_old, m_elo_old) =
            (s.get_profile(siv.clone()).unwrap().elo, s.get_profile(mark.clone()).unwrap().elo);

        s.start_new_season().unwrap();
        s.end_season().unwrap();

        let (s_elo_new, m_elo_new) =
            (s.get_profile(siv.clone()).unwrap().elo, s.get_profile(mark.clone()).unwrap().elo);

        let mut stmt = s.conn.prepare("select count(*) from seasons").unwrap();
        let count = stmt
            .query_map(NO_PARAMS, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();

        let mut stmt = s.conn.prepare("select count(*) from matches").unwrap();
        let match_history_count = stmt
            .query_map(NO_PARAMS, |row| {
                let c: i64 = row.get(0)?;
                Ok(c)
            })
            .unwrap()
            .next()
            .unwrap()
            .unwrap();

        let users = s.get_users().unwrap();
        let siv = s.get_profile(siv).expect("Getting user Sivert");
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(count, 1);
        assert_eq!(match_history_count, 0);
        assert_eq!(users.len(), 0);
        assert!(s_elo_old < 1500.0);
        assert!(m_elo_old > 1500.0);
        assert!(s_elo_new == 1500.0);
        assert!(m_elo_new == 1500.0);
        assert_eq!(siv.user_role, USER_ROLE_SOFT_INACTIVE | USER_ROLE_SUPERUSER);
    }

    #[test]
    fn test_ending_season_yields_awards()
    {
        let db_file = "tempK.db";
        let s = DataBase::new(db_file);

        create_user(&s, "Sivert");
        create_user(&s, "Lars");
        create_user(&s, "Ella");

        s.start_new_season().expect("Staring first season");
        s.end_season().expect("ending first season");

        let users1 = s.get_users().expect("Getting users");
        // Do this scheme to check that a new season gave _new_ awards
        s.start_new_season().expect("Starting season");
        s.end_season().expect("Ending season");

        let users2 = s.get_users().expect("Getting users");


        std::fs::remove_file(db_file).expect("Removing file tempH");

        users1.into_iter().zip(users2.into_iter()).enumerate().for_each(|(i, (u1, u2))| {
            assert_eq!(u1.badges[0].name, BADGES[i]);
            assert_eq!(u2.badges[0].season, 1);
            assert_eq!(u2.badges[1].season, 2);
        });
    }

    #[test]
    fn test_ending_season_when_no_exist_yields_no_awards()
    {
        let db_file = "tempL.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");

        s.end_season().expect("ending season");

        let users = s.get_users().expect("Getting users");
        std::fs::remove_file(db_file).expect("Removing file tempH");
        users.into_iter().for_each(|u| assert!(u.badges.len() == 0));
    }
}
