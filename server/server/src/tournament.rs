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

#[derive(Debug, Sql, PartialEq)]
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

    fn players(tid: i64, bucket: i64, p1: i64, p2: i64) -> Self
    {
        TournamentGame {
            id: -1, // Will be initialized by sqlite
            tournament: tid,
            player1: p1,
            player2: p2,
            bucket: bucket
        }
    }

    fn is_empty(&self) -> bool
    {
        self.player1 == -1 && self.player2 == -1
    }

    fn is_single(&self) -> bool
    {
        (self.player1 == -1 && self.player2 != -1) ||
            (self.player1 != -1 && self.player2 == -1)
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

// Helper functions when creating the matchups
fn parent_index(i: usize) -> usize
{
    (i - 1) / 2
}

fn get_neighbour(i: usize) -> usize
{
    if i & 1 == 1
    {
        i + 1
    }
    else
    {
        i - 1
    }

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
                .rev()
                .find(|g| g.player1 == -1 && g.player2 == -1)
    }

    fn swap_game_matchups(&self, vec: &mut Vec<TournamentGame>, index1: usize, index2: usize)
    {
        let temp = (vec[index1].player1, vec[index1].player2);

        vec[index1].player1 = vec[index2].player1;
        vec[index1].player2 = vec[index2].player2;

        vec[index2].player1 = temp.0;
        vec[index2].player2 = temp.1;
    }

    fn forward_player(&self, vec: &mut Vec<TournamentGame>, index: usize)
    {
        let parent = parent_index(index);
        let player = if vec[index].player1 == -1 { vec[index].player2 } else { vec[index].player1 };
        vec[index].player1 = -1;
        vec[index].player2 = -1;

        if vec[parent].player1 == -1
        {
            vec[parent].player1 = player;
        }
        else
        {
            vec[parent].player2 = player;
        }
    }
    fn advance_game(&self, vec: &mut Vec<TournamentGame>, index: usize)
    {
        if index == 0
        {
            return;
        }

        let parent = parent_index(index);
        if vec[parent].is_empty()
        {
            self.swap_game_matchups(vec, index, parent);
        }
        else
        {
            self.forward_player(vec, index);
        }
    }

    fn spread_game(&self, games: &mut Vec<TournamentGame>, index: usize)
    {
        games[index + 1].player2 = games[index].player2;
        games[index].player2 = -1;
    }

    fn find_first_index_of_single_bucket(&self, games: &Vec<TournamentGame>) -> Option<usize>
    {
        games.iter()
            .enumerate()
            .rev()
            .find(|(_i, g)| g.is_single())
            .map(|(i, _t)| i)
    }

    fn fill_buckets(&self, games: &mut Vec<TournamentGame>, people: &Vec<i64>)
    {
        for users in people.chunks(2)
        {
            if users.len() != 2
            {
                break;
            }
            let (u1, u2) = (&users[0], &users[1]);
            let bucket = self.find_first_empty_bucke(games).expect("Could not find empty bucket");
            bucket.player1 = *u1;
            bucket.player2 = *u2;
        }

        if (people.len() & 1) == 1
        {
            let bucket = self.find_first_empty_bucke(games).expect("Could not find empty bucket");
            let last = people.last().expect("No last, ???");
            bucket.player1 = *last;
        }

    }
    fn generate_buckets(&self, tournament: &Tournament, people: &Vec<i64>) -> Vec<TournamentGame>
    {
        let biggest_power_of_two = ((people.len() as f32).ln() / 2.0_f32.ln()).ceil() as u32;
        let n_buckets = 2_usize.pow(biggest_power_of_two) - 1;

        let mut games = (0..n_buckets).map(|i| TournamentGame::empty(tournament.id, i as i64)).collect();
        self.fill_buckets(&mut games, people);

        if people.len() == n_buckets + 1 as usize
        {
            // Greate! we're done
            return games;
        }

        // Ok, now we have generate the tournament structure, problem is that it's kind of shit,
        // for example, suppose a tournament of 5, it would look like this:
        //  (x denote player, o empty spot)
        //
        // x                x
        //  --|         |--
        // x                o
        //    o         o
        //     -- o o --
        //    o         o
        // x                o
        //  --|         |--
        // x                o
        //
        // Basically a four man tournament + 1 guy automatically in the final. This is boring, a
        // better structure would be something like this:
        //
        // x                x
        //  --|         |--
        // x                x
        //    o         o
        //     -- o o --
        //    o         o
        // x                o
        //  --|         |--
        // o                o
        //
        // This way, one guy is not automatically in the final.

        // If it's even it's fine
        if let Some(index) = self.find_first_index_of_single_bucket(&games)
        {
            let neighbour = get_neighbour(index);

            let first_index = n_buckets - 1;
            self.swap_game_matchups(&mut games, first_index, index);
            if games[neighbour].is_empty()
            {
                let next = index + 1;
                self.swap_game_matchups(&mut games, next, neighbour);
                self.spread_game(&mut games, index);
            }
        }

        // Forward players that 'skips' a round
        let lim = ((n_buckets + 2) / 2) - 1;
        for i in (lim..n_buckets).rev()
        {
            let parent = parent_index(i);
            let neighbour = get_neighbour(i);
            if games[i].is_single() || (games[neighbour].is_empty() && !games[i].is_empty() && games[parent].is_empty())
            {
                self.advance_game(&mut games, i);
            }
        }

        games

    }

    pub fn generate_tournament(&self, tournament: Tournament, people: Vec<i64>) -> ServerResult<()>
    {
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
        std::fs::remove_file(db_file).expect("Removing file tempH");

        let vec4: Vec<i64> = (0..4).collect();
        let vec8: Vec<i64> = (0..8).collect();
        let vec16: Vec<i64> = (0..16).collect();
        let vec = vec![vec4, vec8, vec16];
        let tournament = Tournament { id: 0, state: 0, player_count: 0, name: String::new(), prize: 0, organizer: 0 };

        let vec_ok = |vec| s.generate_buckets(&tournament, &vec).into_iter().rev().take(vec.len() / 2).all(|g| g.player1 != -1 && g.player2 != -1);

        assert!(vec.into_iter().all(vec_ok));
    }

    #[test]
    fn test_gemerate_bucket_not_of_power_of_two()
    {
        let db_file = "tempT11.db";
        let s = DataBase::new(db_file);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        let tournament = Tournament { id: 0, state: 0, player_count: 0, name: String::new(), prize: 0, organizer: 0 };
        let tid = tournament.id;



        // First check vec5
        let vec5: Vec<i64> = (0..5).collect();
        let vec5_ans = vec![
            //final
            TournamentGame::players(tid, 0, -1, -1),
            // Semis
            TournamentGame::players(tid, 1, 0, -1),
            TournamentGame::players(tid, 2, 4, 1),

            // playoffs
            //
            TournamentGame::players(tid, 3, 2, 3),
            TournamentGame::players(tid, 4, -1, -1),
            TournamentGame::players(tid, 5, -1, -1),
            TournamentGame::players(tid, 6, -1, -1),
        ];
        let gen5 = s.generate_buckets(&tournament, &vec5);
        assert_eq!(gen5, vec5_ans);

        // Check vec13
        let vec13: Vec<i64> = (0..13).collect();
        let vec13_ans = vec![
            TournamentGame::players(tid, 0, -1, -1),
            TournamentGame::players(tid, 1, -1, -1),
            TournamentGame::players(tid, 2, -1, -1),
            TournamentGame::players(tid, 3, 0, -1),
            TournamentGame::players(tid, 4, -1, 1),
            TournamentGame::players(tid, 5, -1, -1),
            TournamentGame::players(tid, 6, 12, -1),
            TournamentGame::players(tid, 7, 10, 11),
            TournamentGame::players(tid, 8, -1, -1),
            TournamentGame::players(tid, 9, -1, -1),
            TournamentGame::players(tid, 10, 8, 9),
            TournamentGame::players(tid, 11, 6, 7),
            TournamentGame::players(tid, 12, 4, 5),
            TournamentGame::players(tid, 13, 2, 3),
            TournamentGame::players(tid, 14, -1, -1),
        ];

        let gen13 = s.generate_buckets(&tournament, &vec13);

        /*vec13_ans.iter().zip(gen13.iter()).for_each(|(v, g)|
        {
            println!("predict: {:?}, generate: {:?}", (v.bucket, v.player1, v.player2), (g.bucket, g.player1, g.player2));
        });*/
        assert_eq!(gen13, vec13_ans);


        // Check vec6
        let vec6: Vec<i64> = (0..6).collect();
        let vec6_ans = vec![
            //final
            TournamentGame::players(tid, 0, -1, -1),
            // Semis
            TournamentGame::players(tid, 1, 4, 5),
            TournamentGame::players(tid, 2, -1, -1),

            // playoffs
            //
            TournamentGame::players(tid, 3, -1, -1),
            TournamentGame::players(tid, 4, -1, -1),
            TournamentGame::players(tid, 5, 2, 3),
            TournamentGame::players(tid, 6, 0, 1),
        ];
        let gen6 = s.generate_buckets(&tournament, &vec6);
        vec6_ans.iter().zip(gen6.iter()).for_each(|(v, g)|
        {
            println!("predict: {:?}, generate: {:?}", (v.bucket, v.player1, v.player2), (g.bucket, g.player1, g.player2));
        });
        assert_eq!(gen6, vec6_ans);
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
