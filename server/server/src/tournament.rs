use rusqlite::params;
use server_core::types::{FromSql, *};
use server_macro::Sql;

use crate::{
    _params,
    server::{DataBase, ParamsType},
};

#[repr(u8)]
pub enum TournamentState
{
    Created,
    InProgress,
    Done,
}
#[derive(Debug, PartialEq)]
enum TournamentInfoState
{
    Started(Vec<(String, String)>),
    Created(Vec<String>)
}

pub struct TournamentInfo
{
    tournament: Tournament,
    data: TournamentInfoState
}

#[derive(Sql)]
pub struct Tournament
{
    id: i64,
    name: String,
    prize: i64,
    state: u8,
    player_count: i64,
    organizer: i64
}

#[derive(Debug, Sql)]
pub struct TournamentList
{
    pub id:        i64,
    pub tournament: i64,
    pub player:    i64,
}

#[derive(Debug, Sql)]
pub struct TournamentGame
{
    pub id:        i64,
    pub tournament: i64,
    pub player1:     i64,
    pub player2:     i64,
    pub bucket: i64
}

impl TournamentGame
{
    fn empty(tid: i64, bucket: i64) -> Self
    {
        TournamentGame {
            id: -1, // Will be initialized by sqlite
            tournament: tid,
            player1: -1,
            player2: -1,
            bucket: bucket
        }
    }
}

#[derive(Debug, Sql)]
pub struct TournamentMatch
{
    pub id:        i64,
    pub winner:     i64,
    pub loser:     i64,
    pub tournament_game: i64
}


impl DataBase
{
    pub fn create_tournament(
        &self,
        pid: i64,
        name: String,
        prize: i64,
        player_count: i64,
    ) -> ServerResult<i64>
    {
        if player_count < 4
        {
            return Err(ServerError::Tournament(TournamentError::WrongTournamentCount));
        }

        self.conn.execute(
            "insert into tournaments (name, prize, state, organizer, player_count) values \
             (?1, ?2, ?3, ?4, ?5)",
            params![name, prize, TournamentState::Created as i64, pid, player_count],
        )?;
        self.sql_one::<Tournament, _>("select * from tournaments order by id desc limit 1", None)
            .map(|t| t.id)
    }

    pub fn join_tournament(&self, token: String, tid: i64) -> ServerResult<bool>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let list = self.sql_many::<TournamentList, _>(
            "select * from tournament_lists where tournament = ?1",
            _params![tid],
        )?;

        let tournament =
            self.sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid])?;
        if tournament.state != TournamentState::Created as u8
        {
            return Err(ServerError::Tournament(TournamentError::WrongState));
        }

        if list.iter().map(|t| t.player).any(|pid| pid == user.id)
        {
            return Err(ServerError::Tournament(TournamentError::AlreadyJoined));
        }


        if list.len() + 1 == tournament.player_count as usize
        {
            let mut players: Vec<i64> = list.into_iter().map(|tl| tl.player).collect();
            players.push(user.id);
            self.generate_tournament(tournament, players)?;
            self.delete_tourament_list(tid)?;
            self.update_tournament_state(tid, TournamentState::InProgress)?;
            Ok(true)
        }
        else
        {
            self.add_player_to_tournament(tid, user.id)?;
            Ok(false)
        }
    }

    pub fn generate_matchups(&self, people: Vec<i64>) -> Vec<i64>
    {
        //@TODO: Get some kind of match up
        people
    }

    fn find_first_empty_bucke<'a>(&self, games: &'a mut Vec<TournamentGame>) -> Option<&'a mut TournamentGame>
    {
            games.iter_mut()
                .find(|g| g.player1 == -1 && g.player2 == -1)
    }
    fn generate_buckets(&self, tournament: &Tournament, people: &Vec<i64>) -> Vec<TournamentGame>
    {
        let biggest_power_of_two = ((people.len() as f32).ln() / 2.0_f32.ln()).ceil() as u32;
        let n_buckets = 2_usize.pow(biggest_power_of_two) - 1;

        let mut games = (0..n_buckets).map(|i| TournamentGame::empty(tournament.id, i as i64)).collect();
        for users in people.chunks(2)
        {
            if users.len() != 2
            {
                break;
            }
            let (u1, u2) = (&users[0], &users[1]);
            let bucket = self.find_first_empty_bucke(&mut games).expect("Could not find empty bucket");
            bucket.player1 = *u1;
            bucket.player2 = *u2;
        }

        if (people.len() & 1) == 1
        {
            let bucket = self.find_first_empty_bucke(&mut games).expect("Could not find empty bucket");
            let last = people.last().expect("No last, ???");
            bucket.player1 = *last;
        }
        games
    }

    pub fn generate_tournament(&self, tournament: Tournament, people: Vec<i64>) -> ServerResult<()>
    {
        //@TODO: How to generate??
        let games = self.generate_buckets(&tournament, &people);

        for bucket in games
        {
            self.create_tournament_game(bucket.player1, bucket.player2, bucket.bucket, bucket.tournament)?;
        }

        Ok(())
    }

    fn create_tournament_game(&self, u1: i64, u2: i64, bucket: i64, tid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_games (player1, player2, bucket, tournament) values (?1, ?2, ?3, ?4)",
            params![u1, u2, bucket, tid])?;
        Ok(())
    }


    fn add_player_to_tournament(&self, tid: i64, pid: i64) -> ServerResult<()>
    {
        self.conn
            .execute("insert into tournament_lists (tournament, player) values (?1, ?2)", params![
                tid, pid
            ])?;
        Ok(())
    }

    fn delete_tourament_list(&self, tid: i64) -> ServerResult<()>
    {
        self.conn
            .execute("delete from tournament_lists where tournament = ?1", params![tid])?;
        Ok(())
    }

    fn update_tournament_state(&self, tid: i64, state: TournamentState) -> ServerResult<()>
    {
        self.conn
            .execute("update tournaments set state = ?1 where id = ?2", params![state as u8, tid])?;
        Ok(())
    }

    pub fn get_tournaments(&self) -> ServerResult<Vec<TournamentInfo>>
    {
        let tournaments = self.sql_many::<Tournament, _>("select * from tournaments", None)?;
        let t_infos: Vec<TournamentInfo> = tournaments.into_iter().map(|t|
        {
            if t.state == TournamentState::Created as u8
            {
                let players: Vec<String> = self.sql_many::<TournamentList, _>("select * from tournament_lists where tournament = ?1", _params![t.id]).unwrap()
                                            .into_iter()
                                            .map(|t| self.get_user_without_matches_by("id", "=", &t.player.to_string()).unwrap().name)
                                            .collect();
                TournamentInfo {
                    tournament: t,
                    data: TournamentInfoState::Created(players)
                }
            }
            else
            {
                let h = |t: TournamentGame|
                {
                    let f = |id: i64|
                        match self.get_user_without_matches_by("id", "=", &id.to_string())
                        {
                            Ok(u) => u.name,
                            Err(_) => String::from("")
                        };
                    (f(t.player1), f(t.player2))
                };

                let players: Vec<(String, String)> =
                    self.sql_many::<TournamentGame, _>("select * from tournament_games where tournament = ?1", _params![t.id]).unwrap()
                        .into_iter()
                        .map(h)
                        .collect();
                TournamentInfo {
                    tournament: t,
                    data: TournamentInfoState::Started(players)
                }
            }
        })
        .collect();
        Ok(t_infos)
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use crate::{server::DataBase, test_util::*};


    #[test]
    fn test_can_create_tornament()
    {
        let db_file = "tempT3.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");

        let id_1 = s.create_tournament(1, "epic".to_string(), 3, 4);
        let id_2 = s.create_tournament(1, "epic_2".to_string(), 3, 8);
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(id_1.is_ok() && id_1.unwrap() == 1);
        assert!(id_2.is_ok() && id_2.unwrap() == 2);
    }

    #[test]
    fn test_can_join_tornament()
    {
        let db_file = "tempT5.db";
        let s = DataBase::new(db_file);
        let token = create_user(&s, "Sivert");
        create_user(&s, "Bernt");
        create_user(&s, "Markus");
        create_user(&s, "Ella");


        let tid = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let res = s.join_tournament(token, tid);

        std::fs::remove_file(db_file).expect("Removing file tempH");
        println!("{:?}", res);

        assert!(res.is_ok() && !res.unwrap());
    }

    #[test]
    fn test_can_not_join_twice()
    {
        let db_file = "tempT6.db";
        let s = DataBase::new(db_file);
        let token = create_user(&s, "Sivert");
        create_user(&s, "Bernt");
        create_user(&s, "Markus");
        create_user(&s, "Ella");


        let tid = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let _ = s.join_tournament(token.clone(), tid);
        let res = s.join_tournament(token, tid);

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res.is_err());
    }

    #[test]
    fn test_join_tornament_gets_started()
    {
        let db_file = "tempT7.db";
        let s = DataBase::new(db_file);
        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        let tid = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let rs = s.join_tournament(token_s, tid);
        let rb = s.join_tournament(token_b, tid);
        let rm = s.join_tournament(token_m, tid);
        let re = s.join_tournament(token_e, tid);

        let tournament = s.sql_one::<Tournament, _>("select * from tournaments", None).unwrap();
        let games = s.sql_many::<TournamentGame, _>("select * from tournament_games", None).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(rs.is_ok() && !rs.unwrap());
        assert!(rb.is_ok() && !rb.unwrap());
        assert!(rm.is_ok() && !rm.unwrap());
        assert!(re.is_ok() && re.unwrap());
        assert_eq!(tournament.state, TournamentState::InProgress as u8);
        assert_eq!(games.len(), 3);
    }

    #[test]
    fn test_cannot_join_started_tornament()
    {
        let db_file = "tempT8.db";
        let s = DataBase::new(db_file);
        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");
        let token_l = create_user(&s, "Lars");


        let tid = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let _ = s.join_tournament(token_s, tid);
        let _ = s.join_tournament(token_b, tid);
        let _ = s.join_tournament(token_m, tid);
        let re = s.join_tournament(token_e, tid);
        let rl = s.join_tournament(token_l, tid);

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(re.is_ok() && re.unwrap());
        assert!(rl.is_err());
    }

    #[test]
    fn test_generate_bucket_power_of_two()
    {
        let db_file = "tempT10.db";
        let s = DataBase::new(db_file);

        let vec4: Vec<i64> = (0..4).collect();
        let vec8: Vec<i64> = (0..8).collect();
        let vec16: Vec<i64> = (0..16).collect();
        let vec = vec![vec4, vec8, vec16];
        let tournament = Tournament { id: 0, state: 0, player_count: 0, name: String::new(), prize: 0, organizer: 0 };

        let vec_ok = |vec| s.generate_buckets(&tournament, &vec).into_iter().take(vec.len() / 2).all(|g| g.player1 != -1 && g.player2 != -1);

        assert!(vec.into_iter().all(vec_ok));
    }

    #[test]
    fn test_gemerate_bucket_not_of_power_of_two()
    {
        let db_file = "tempT11.db";
        let s = DataBase::new(db_file);

        let vec5: Vec<i64> = (0..5).collect();
        let vec12: Vec<i64> = (0..13).collect();
        let vec25: Vec<i64> = (0..25).collect();
        let vec = vec![vec5, vec12, vec25];
        let tournament = Tournament { id: 0, state: 0, player_count: 0, name: String::new(), prize: 0, organizer: 0 };

        let vec_ok = |vec: Vec<_>|
        {
            let take_amount = (vec.len() as f32 / 2.0).ceil() as usize -1;
            let buckets = s.generate_buckets(&tournament, &vec).into_iter();
            let first = buckets.take(take_amount).all(|g| g.player1 != -1 && g.player2 != -1);
            let buckets = s.generate_buckets(&tournament, &vec).into_iter();
            let second = buckets.skip(take_amount).take(1).all(|g| g.player1 != g.player2);

            first && second
        };

        assert!(vec.into_iter().all(vec_ok));
    }

    #[test]
    fn test_can_get_tournaments()
    {
        let db_file = "tempT12.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        let tid = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let _ = s.join_tournament(token_s.clone(), tid);
        let _ = s.join_tournament(token_b.clone(), tid);
        let _ = s.join_tournament(token_m, tid);
        let _ = s.join_tournament(token_e, tid);

        let tid2 = s.create_tournament(1, "Epic".to_string(), 3, 4).unwrap();
        let _ = s.join_tournament(token_s, tid2);
        let _ = s.join_tournament(token_b, tid2);

        let tournaments = s.get_tournaments();
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(tournaments.is_ok());
        let mut tournaments = tournaments.unwrap();
        let first = tournaments.remove(0);
        let last = tournaments.remove(0);


        let assert_func = |t: TournamentInfo|
        {
            if t.tournament.state == TournamentState::InProgress as u8
            {
                match t.data
                {
                    TournamentInfoState::Started(mut vec) =>
                    {
                        let mut dummy = vec![("Ella".to_string(), "Bernt".to_string()), ("Sivert".to_string(), "Markus".to_string())];
                        assert_eq!(t.tournament.id, tid);
                        assert_eq!(dummy.sort(), vec.sort());
                    },
                    _ => unreachable!()
                }
            }
            else
            {
                match t.data
                {
                    TournamentInfoState::Created(vec) =>
                    {
                        assert_eq!(t.tournament.id, tid2);
                        assert_eq!(vec.len(), 2);
                    },
                    _ => unreachable!()
                }
            }
        };
        assert_func(first);
        assert_func(last);
    }
}
