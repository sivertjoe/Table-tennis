use std::io::prelude::*;

use rusqlite::params;
use serde_derive::{Deserialize, Serialize};
use server_core::{
    constants::*,
    types::{FromSql, *},
};
use server_macro::Sql;

use crate::{
    _params,
    server::{DataBase, ParamsType},
};

#[cfg_attr(test, derive(Debug, PartialOrd, Ord, Eq, PartialEq))]
pub enum TournamentType
{
    SingleElimination,
    DoubleElimination,
}

impl From<u8> for TournamentType
{
    fn from(n: u8) -> Self
    {
        match n
        {
            0 => TournamentType::SingleElimination,
            1 => TournamentType::DoubleElimination,
            _ => unreachable!(),
        }
    }
}

#[derive(Deserialize)]
pub struct GetTournamentOptions
{
    pub query: Option<String>,
}

#[derive(Sql)]
pub struct Image
{
    pub id:   i64,
    pub name: String,
}

#[derive(Sql)]
pub struct TournamentBadge
{
    pub id:    i64,
    pub image: i64,
    pub pid:   i64,
    pub tid:   i64,
}

#[derive(Deserialize)]
pub struct JoinTournament
{
    pub token: String,
    pub tid:   i64,
}

#[derive(Deserialize)]
pub struct CreateTournament
{
    organizer_token: String,
    name:            String,
    image:           String,
    player_count:    i64,
    ttype:           String,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq))]
pub struct RegisterTournamentMatch
{
    organizer_token: String,
    winner:          String,
    loser:           String,
    tournament_game: i64,
}

#[repr(u8)]
#[cfg_attr(test, derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq))]
pub enum TournamentState
{
    Created,
    InProgress,
    Done,
}

impl From<u8> for TournamentState
{
    fn from(n: u8) -> Self
    {
        use TournamentState::*;
        match n
        {
            0 => Created,
            1 => InProgress,
            2 => Done,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialOrd, Ord, Eq, PartialEq))]
enum TournamentInfoState
{
    Games(Vec<TournamentGameInfo>),
    Players(Vec<String>),
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialOrd, Ord, Eq, PartialEq))]
struct TournamentGameInfo
{
    id:      i64,
    player1: String,
    player2: String,
    bucket:  i64,
}

#[derive(Sql)]
struct TournamentWinner
{
    id:         i64,
    player:     i64,
    tournament: i64,
}


#[derive(Serialize)]
pub struct TournamentInfo
{
    tournament: SendTournament,
    data:       TournamentInfoState,
}

#[derive(Sql)]
pub struct Tournament
{
    pub id:           i64,
    pub name:         String,
    pub prize:        i64,
    pub state:        u8,
    pub ttype:        u8,
    pub player_count: i64,
    pub organizer:    i64,
}

#[derive(Serialize)]
pub struct SendTournament
{
    id:             i64,
    name:           String,
    prize:          String,
    player_count:   i64,
    state:          u8,
    ttype:          u8,
    organizer_name: String,
    winner:         String,
}

#[derive(Sql)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TournamentList
{
    pub id:         i64,
    pub tournament: i64,
    pub player:     i64,
}

#[derive(Sql)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TournamentGame
{
    pub id:         i64,
    pub tournament: i64,
    pub player1:    i64,
    pub player2:    i64,
    pub bucket:     i64,
}

// This struct is the result of a game
#[derive(Sql)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TournamentMatch
{
    pub id:     i64,
    pub game:   i64,
    pub winner: i64,
    pub loser:  i64,
}

impl TournamentGame
{
    fn empty(tid: i64, bucket: i64) -> Self
    {
        TournamentGame {
            id:         -1, // Will be initialized by sqlite
            tournament: tid,
            player1:    0,
            player2:    0,
            bucket:     bucket,
        }
    }

    fn players(tid: i64, bucket: i64, p1: i64, p2: i64) -> Self
    {
        TournamentGame {
            id:         -1, // Will be initialized by sqlite
            tournament: tid,
            player1:    p1,
            player2:    p2,
            bucket:     bucket,
        }
    }

    fn get_single(&self) -> i64
    {
        if self.player1 != 0 { self.player1 } else { self.player2 }
    }

    fn is_single(&self) -> bool
    {
        (self.player1 == 0 && self.player2 != 0) || (self.player1 != 0 && self.player2 == 0)
    }

    fn insert_player(&mut self, player: i64)
    {
        if self.player1 == 0
        {
            self.player1 = player;
        }
        else
        {
            self.player2 = player;
        }
    }
}

struct LoserBracketGenerator(i64);

impl LoserBracketGenerator
{
    fn new(p: i64) -> Self
    {
        LoserBracketGenerator(p)
    }
}

impl Iterator for LoserBracketGenerator
{
    type Item = Vec<i64>;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.0 == 1
        {
            None
        }
        else
        {
            let p = self.0;
            self.0 >>= 1;
            Some(((p / 2 - 1)..(p - 1)).collect::<Self::Item>())
        }
    }
}

const DEFAULT_PICTURE: &str = "tournament_badges/default.png";

impl DataBase
{
    pub fn create_tournament(&self, info: CreateTournament) -> ServerResult<()>
    {
        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments order by id desc limit 1", None)
            .map(|t| t.id + 1)
            .unwrap_or(1);

        // Use default picture
        let prize = if info.image == ""
        {
            self.get_default_prize()?
        }
        else
        {
            self.create_image_prize(info.image, tournament)?
        };

        let organizer_pid =
            self.get_user_without_matches_by("uuid", "=", &info.organizer_token)?.id;

        let ttype = match info.ttype.as_str()
        {
            "singleElimination" => TournamentType::SingleElimination,
            "doubleElimination" => TournamentType::DoubleElimination,
            _ => return Err(ServerError::Tournament(TournamentError::InvalidTtype)),
        };
        self._create_tournament(organizer_pid, info.name, prize, info.player_count, ttype)?;
        Ok(())
    }

    fn get_default_prize(&self) -> ServerResult<i64>
    {
        if let Ok(image) = self
            .sql_one::<Image, _>("select * from images where name = ?1", _params![DEFAULT_PICTURE])
        {
            Ok(image.id)
        }
        else
        {
            self.conn
                .execute("insert into images (name) values (?1)", params!(DEFAULT_PICTURE))?;
            Ok(self
                .sql_one::<Image, _>("select * from images where name = ?1", _params![
                    DEFAULT_PICTURE
                ])
                .unwrap()
                .id)
        }
    }

    fn create_image_prize(&self, image: String, tournament: i64) -> ServerResult<i64>
    {
        let image_name = format!("{}/{}.png", TOURNAMENT_BADGES_PATH, tournament);
        let mut file = std::fs::File::create(&format!("{}/{}", ASSETS_PATH, &image_name))
            .expect("creating file");

        let bin: Vec<&str> = image.as_str().splitn(2, ",").collect();
        let bin = base64::decode(bin[1]).unwrap();
        file.write_all(&bin).expect("Writing into file");

        self.create_image_prize_table(image_name)
    }

    fn create_image_prize_table(&self, image_name: String) -> ServerResult<i64>
    {
        self.conn
            .execute("insert into images (name) values (?1)", params![image_name])?;
        self.sql_one::<Image, _>("select * from images order by id desc limit 1", None)
            .map(|t| t.id)
    }

    fn _create_tournament(
        &self,
        pid: i64,
        name: String,
        prize: i64,
        player_count: i64,
        ttype: TournamentType,
    ) -> ServerResult<()>
    {
        if player_count < 4 || player_count > 64
        {
            return Err(ServerError::Tournament(TournamentError::WrongTournamentCount));
        }

        self.conn.execute(
            "insert into tournaments (name, prize, state, ttype, organizer, player_count) values \
             (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, prize, TournamentState::Created as i64, ttype as i64, pid, player_count],
        )?;
        Ok(())
    }

    pub fn leave_tournament(&self, token: String, tid: i64) -> ServerResult<()>
    {
        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid])?;
        if tournament.state != TournamentState::Created as u8
        {
            return Err(ServerError::Tournament(TournamentError::WrongState));
        }
        let pid = self.get_user_without_matches_by("uuid", "=", &token)?.id;

        self.conn.execute(
            "delete from tournament_lists where tournament = ?1 and player = ?2",
            params![tid, pid],
        )?;
        Ok(())
    }

    pub fn join_tournament(&self, token: String, tid: i64) -> ServerResult<bool>
    {
        let user = self.get_user_without_matches_by("uuid", "=", token.as_str())?;
        let list = self.sql_many::<TournamentList, _>(
            "select * from tournament_lists where tournament = ?1",
            _params![tid],
        )?;

        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid])?;
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

    fn get_all_tournament_games(&self, tid: i64) -> ServerResult<Vec<TournamentGame>>
    {
        self.sql_many::<TournamentGame, _>(
            "select * from tournament_games where tournament = ?1",
            _params![tid],
        )
    }

    fn get_all_single_tournament_games(&self, tid: i64) -> ServerResult<Vec<TournamentGame>>
    {
        self.sql_many::<TournamentGame, _>(
            "select * from tournament_games where tournament = ?1 and bucket >= 0",
            _params![tid],
        )
    }

    pub fn register_tournament_match(
        &self,
        register_game: RegisterTournamentMatch,
    ) -> ServerResult<()>
    {
        let game = self.sql_one::<TournamentGame, _>(
            "select * from tournament_games where id = ?1",
            _params![register_game.tournament_game],
        )?;

        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![
                game.tournament
            ])?;
        let organizer_id = self
            .get_user_without_matches_by("uuid", "=", &register_game.organizer_token)?
            .id;

        if tournament.state != TournamentState::InProgress as u8
        {
            return Err(ServerError::Tournament(TournamentError::WrongState));
        }

        if organizer_id != tournament.organizer
        {
            return Err(ServerError::Tournament(TournamentError::NotOrganizer));
        }

        if game.player1 == 0 || game.player2 == 0
        {
            return Err(ServerError::Tournament(TournamentError::InvalidGame));
        }

        if self
            .sql_one::<TournamentMatch, _>(
                "select * from tournament_matches where game = ?1",
                _params![game.id],
            )
            .is_ok()
        {
            return Err(ServerError::Tournament(TournamentError::GameAlreadyPlayed));
        }
        match tournament.ttype.into()
        {
            TournamentType::SingleElimination =>
            {
                self.handle_single_elimination_match(&game, &register_game, &tournament)?
            },
            TournamentType::DoubleElimination =>
            {
                self.handle_double_elimination_match(&game, &register_game, &tournament)?
            },
        }
        Ok(())
    }

    fn handle_double_elimination_match(
        &self,
        game: &TournamentGame,
        register_game: &RegisterTournamentMatch,
        tournament: &Tournament,
    ) -> ServerResult<()>
    {
        let mut games = self.get_all_single_tournament_games(game.tournament)?;
        let winner_id = self.get_user_without_matches(&register_game.winner)?.id;
        let loser_id = self.get_user_without_matches(&register_game.loser)?.id;

        self.create_match_from_game(winner_id, loser_id, game.id)?;

        let biggest_power_of_two =
            ((tournament.player_count as f64).ln() / 2.0_f64.ln()).ceil() as u32;
        let power = 2_i64.pow(biggest_power_of_two);

        match game.bucket
        {
            // Winners bracket final game
            0 =>
            {
                self.conn.execute(
                    "update tournament_games set player1 = ?1 where bucket = ?2 and tournament = \
                     ?3",
                    params![winner_id, power, tournament.id],
                )?;
                self.send_loser_to_losers_bracket(loser_id, &game, tournament.id);
            },
            // Losers bracket final game
            -1 =>
            {
                self.conn.execute(
                    "update tournament_games set player2 = ?1 where bucket = ?2 and tournament = \
                     ?3",
                    params![winner_id, power, tournament.id],
                )?;
            },
            // First final
            n if n == power =>
            {
                // Tournament is over
                if winner_id == game.player1
                {
                    self.finish_tournament(&tournament, winner_id)?;
                }
                // Loser won, go to game #2
                else
                {
                    self._create_tournament_game(loser_id, winner_id, power + 1, tournament.id)?;
                }
            },
            // Second final
            N if N == (power + 1) =>
            {
                self.finish_tournament(&tournament, winner_id)?;
            },
            // Normal game
            _ =>
            {
                if game.bucket > 0
                // winners bracket match
                {
                    let parent = self.advance_player(&mut games, game.bucket as usize, winner_id);
                    let game_index = games.iter().position(|g| g.bucket == parent as i64).unwrap();
                    self.update_bucket(&games[game_index])?;
                    self.send_loser_to_losers_bracket(loser_id, &game, tournament.id);
                }
                else
                // loser bracket match
                {
                    let loser_bracket_parent =
                        self.loser_bracket_parent(game.bucket, tournament.player_count);

                    let mut lgame = self.sql_one::<TournamentGame, _>(
                        "select * from tournament_games where bucket = ?1 and tournament = ?2",
                        _params![loser_bracket_parent, tournament.id],
                    )?;

                    if lgame.player1 == 0
                    {
                        lgame.player1 = winner_id;
                    }
                    else
                    {
                        lgame.player2 = winner_id;
                    }
                    self.update_bucket(&lgame)?;
                }
            },
        };

        Ok(())
    }

    fn loser_bracket_parent(&self, bucket: i64, player_count: i64) -> i64
    {
        let bucket = bucket.abs();
        let biggest_power_of_two = (((bucket + 2) as f64).ln() / 2.0_f64.ln()).ceil() as u32;
        let power = 2_i64.pow(biggest_power_of_two);

        let bracket_size = power / 4;
        let x = bracket_size - 1;

        let parent = |n: i64| (((n - 1) as f64) / 2.0).ceil() as i64;

        if power - 2 - bracket_size > bucket
        {
            -(parent(bucket - x) + x)
        }
        else
        {
            -(bucket - bracket_size)
        }
    }

    fn send_loser_to_losers_bracket(
        &self,
        id: i64,
        game: &TournamentGame,
        tid: i64,
    ) -> ServerResult<()>
    {
        // OK, this guy just lost match in bucket #n, send him to loser bracket where
        // player1 = #n - 1,
        // right?
        let less = -(game.bucket + 1);
        let mut loser_bracket = self.sql_one::<TournamentGame, _>(
            "select * from tournament_games where (player1 = ?1 or player2 = ?1) and tournament = \
             ?2",
            _params![less, tid],
        )?;

        if loser_bracket.player1 == less
        {
            loser_bracket.player1 = id;
        }
        else
        {
            loser_bracket.player2 = id;
        }
        self.update_bucket(&loser_bracket)
    }

    fn finish_tournament(&self, tournament: &Tournament, winner_id: i64) -> ServerResult<()>
    {
        self.create_tournament_winner(tournament.id, winner_id)?;
        self.update_tournament_state(tournament.id, TournamentState::Done)?;
        self.award_winner_with_prize(tournament.prize, winner_id, tournament.id)?;
        Ok(())
    }

    fn handle_single_elimination_match(
        &self,
        game: &TournamentGame,
        register_game: &RegisterTournamentMatch,
        tournament: &Tournament,
    ) -> ServerResult<()>
    {
        let mut games = self.get_all_tournament_games(game.tournament)?;
        let winner_id = self.get_user_without_matches(&register_game.winner)?.id;
        let loser_id = self.get_user_without_matches(&register_game.loser)?.id;

        self.create_match_from_game(winner_id, loser_id, game.id)?;
        // This was the last game, award some stuff
        if game.bucket == 0
        {
            // @TODO: Check if double elim
            self.finish_tournament(tournament, winner_id)?;
        }
        else
        {
            let game_index = games.iter().position(|g| g.bucket == game.bucket).unwrap();
            let parent = self.advance_player(&mut games, game_index, winner_id);
            self.update_bucket(&games[parent])?;
        }
        Ok(())
    }

    fn create_match_from_game(
        &self,
        winner_id: i64,
        loser_id: i64,
        game_id: i64,
    ) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_matches (game, winner, loser) values (?1, ?2, ?3)",
            params![game_id, winner_id, loser_id],
        )?;
        Ok(())
    }

    fn award_winner_with_prize(&self, prize: i64, pid: i64, tid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_badges (image, pid, tid) values (?1, ?2, ?3)",
            params![prize, pid, tid],
        )?;
        Ok(())
    }

    fn create_tournament_winner(&self, tid: i64, pid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_winners (tournament, player) values (?1, ?2)",
            params![tid, pid],
        )?;
        Ok(())
    }

    pub fn generate_matchups(&self, mut people: Vec<i64>) -> Vec<i64>
    {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        people.shuffle(&mut rng);
        people
    }

    fn advance_game(&self, games: &mut Vec<TournamentGame>, i: usize)
    {
        let parent = (i - 1) / 2;
        let player = games[i].get_single();

        let parent_player =
            if i & 1 == 1 { &mut games[parent].player1 } else { &mut games[parent].player2 };

        *parent_player = player;
        games[i].player1 = 0;
        games[i].player2 = 0;
    }

    fn advance_player(&self, games: &mut Vec<TournamentGame>, i: usize, winner: i64) -> usize
    {
        let parent = (i - 1) / 2;
        let mut game = games.iter_mut().find(|g| g.bucket == parent as i64).unwrap();

        let parent_player = if i & 1 == 1 { &mut game.player1 } else { &mut game.player2 };

        *parent_player = winner;
        parent
    }

    fn generate_buckets(&self, tournament: &Tournament, people: &Vec<i64>) -> Vec<TournamentGame>
    {
        let biggest_power_of_two = ((people.len() as f32).ln() / 2.0_f32.ln()).ceil() as u32;
        let power = 2_usize.pow(biggest_power_of_two);

        let mut games: Vec<TournamentGame> =
            (0..power - 1).map(|i| TournamentGame::empty(tournament.id, i as i64)).collect();

        for (player, i) in people.iter().zip(((power / 2) - 1..power - 1).cycle())
        {
            games[i].insert_player(*player);
        }

        for i in power / 2 - 1..power - 1
        {
            if games[i].is_single()
            {
                self.advance_game(&mut games, i);
            }
        }
        games
    }

    fn update_bucket(&self, game: &TournamentGame) -> ServerResult<()>
    {
        self.conn.execute(
            "update tournament_games set player1 = ?1, player2 = ?2 where id = ?3",
            params![game.player1, game.player2, game.id],
        )?;
        Ok(())
    }

    /*
     * @NOTE:
     * This is a little scuffed, for the TournamentGames, if the value is below 0
     * it means that this is the position of the loser in the corresponding
     * positive number bucket. Meaning, the loser of bucket #n will be
     * placed in the tournament_game where player1 = -#n. BUT, since 0 is a
     * valid bucket AND used for denoting empty game, I need to do
     * player1 = - (#n + 1). Kind of scuffed, but it works..
     * Use pos to get the actual number
     *
     * Forward games, are games that have been forwarded
     */
    fn create_losers_bracket(
        &self,
        player_count: i64,
        tid: i64,
        forward_games: Vec<i64>,
    ) -> ServerResult<()>
    {
        let player_count = player_count;
        let biggest_power_of_two = ((player_count as f32).ln() / 2.0_f32.ln()).ceil() as u32;
        let power = 2_usize.pow(biggest_power_of_two);

        let mut matches: Vec<TournamentGame> = Vec::new();
        let mut iter = LoserBracketGenerator::new(power as i64).into_iter();

        // Helper functions for later
        let pos = |i: i64| -(i + 1);
        let get_pair = |pair: &[i64]| -> (i64, i64) {
            match pair
            {
                &[a, b] => (a, b),
                _ => unreachable!(),
            }
        };
        let create_empty = |len: usize, matches: &mut Vec<TournamentGame>, bucket: &mut i64| {
            for i in 0..len
            {
                matches.push(TournamentGame::empty(tid, *bucket));
                *bucket += 1;
            }
        };
        let create_normal = |set: Vec<i64>, matches: &mut Vec<TournamentGame>, bucket: &mut i64| {
            for p in set
            {
                matches.push(TournamentGame::players(tid, *bucket, pos(p), 0));
                *bucket += 1;
            }
        };

        // These two sections don't really follow the pattern, so just deal with them
        // first
        let first_section = iter.next().unwrap();
        let second_section = iter.next().unwrap();

        let mut bucket = -(power as i64 - 2);
        for pair in first_section.chunks(2)
        {
            let (b1, b2) = get_pair(pair);
            matches.push(TournamentGame::players(tid, bucket, pos(b1), pos(b2)));
            bucket += 1;
        }

        create_normal(second_section, &mut matches, &mut bucket);

        // Check to see if we can forward anything
        // IF we need to forward something, its always from bot to top in terms of
        // brackets
        println!("{:?}", &forward_games);
        for (i, forward) in forward_games.into_iter().enumerate()
        {
            // The match we're forwarding to
            let parent = matches[i].bucket + (power as i64 / 4);
        }


        // Now that the hard coded guys left, we can finally start the pattern
        for section in iter
        {
            create_empty(section.len(), &mut matches, &mut bucket);
            create_normal(section, &mut matches, &mut bucket);
        }

        for game in matches
        {
            self._create_tournament_game(game.player1, game.player2, game.bucket, game.tournament)?;
        }

        Ok(())
    }

    pub fn generate_tournament(&self, tournament: Tournament, people: Vec<i64>)
        -> ServerResult<()>
    {
        let games = self.generate_buckets(&tournament, &self.generate_matchups(people));
        if tournament.ttype == TournamentType::DoubleElimination as u8
        {
            let biggest_power_of_two =
                ((tournament.player_count as f64).ln() / 2.0_f64.ln()).ceil() as u32;

            let power = 2_i64.pow(biggest_power_of_two);


            // Games that were forwarded
            let forwards: Vec<i64> = games
                .iter()
                .take(power as usize / 2)
                .filter_map(|g| g.is_single().then(|| g.bucket))
                .collect();

            self.create_losers_bracket(tournament.player_count, tournament.id, forwards)?;


            // We will denote the final final match with n, where n is the highest power of
            // two E.g 16, 32, 8, etc.

            // For now, I dont wanna generate the second final, only generate it
            // if it's needed
            self._create_tournament_game(0, 0, power, tournament.id)?;
        }

        for bucket in games
        {
            self._create_tournament_game(
                bucket.player1,
                bucket.player2,
                bucket.bucket,
                bucket.tournament,
            )?;
        }

        Ok(())
    }

    fn _create_tournament_game(&self, u1: i64, u2: i64, bucket: i64, tid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_games (player1, player2, bucket, tournament) values (?1, ?2, \
             ?3, ?4)",
            params![u1, u2, bucket, tid],
        )?;
        Ok(())
    }

    fn add_player_to_tournament(&self, tid: i64, pid: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "insert into tournament_lists (tournament, player) values (?1, ?2)",
            params![tid, pid],
        )?;
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
        self.conn.execute("update tournaments set state = ?1 where id = ?2", params![
            state as u8,
            tid
        ])?;
        Ok(())
    }

    fn convert(&self, tg: TournamentGame) -> TournamentGameInfo
    {
        let h = |id: i64| -> String {
            if id > 0
            {
                match self.get_user_without_matches_by("id", "=", &id.to_string())
                {
                    Ok(u) => u.name,
                    Err(_) => String::from(""),
                }
            }
            else
            {
                String::from("")
            }
        };

        TournamentGameInfo {
            player1: h(tg.player1),
            player2: h(tg.player2),
            id:      tg.id,
            bucket:  tg.bucket,
        }
    }

    fn get_image_name(&self, image: i64) -> ServerResult<String>
    {
        self.sql_one::<Image, _>("select * from images where id = ?1", _params![image])
            .map(|i| i.name)
    }

    fn convert_tournament(
        &self,
        tournament: Tournament,
        tw: Option<String>,
    ) -> ServerResult<SendTournament>
    {
        Ok(SendTournament {
            name:           tournament.name,
            prize:          self.get_image_name(tournament.prize)?,
            player_count:   tournament.player_count,
            state:          tournament.state,
            ttype:          tournament.ttype,
            id:             tournament.id,
            organizer_name: self
                .get_user_without_matches_by("id", "=", &tournament.organizer.to_string())?
                .name,
            winner:         tw.unwrap_or(String::from("")),
        })
    }

    fn map_tournament_info(&self, t: Tournament) -> TournamentInfo
    {
        /* @Optimization:
         *  Get all the users necessary, E.g select * from ... where id in (<all
         * users ids>)  and pass all users to the map function here and
         *  self.conver(tg, users), this will (probabably) be more
         *  efficient
         *
         *  Another Optimization:
         *  create a get_user_without_matches_by_id function, this way we won't
         * allocate the string  for the id all the time, this is such a
         * wasted allocation
         */
        if t.state == TournamentState::Created as u8
        {
            let players: Vec<String> = self
                .sql_many::<TournamentList, _>(
                    "select * from tournament_lists where tournament = ?1",
                    _params![t.id],
                )
                .unwrap()
                .into_iter()
                .map(|t| {
                    self.get_user_without_matches_by("id", "=", &t.player.to_string()).unwrap().name
                })
                .collect();
            TournamentInfo {
                tournament: self.convert_tournament(t, None).unwrap(),
                data:       TournamentInfoState::Players(players),
            }
        }
        else
        {
            let players: Vec<TournamentGameInfo> = self
                .sql_many::<TournamentGame, _>(
                    "select * from tournament_games where tournament = ?1 order by bucket desc",
                    _params![t.id],
                )
                .unwrap()
                .into_iter()
                .map(|tg| self.convert(tg))
                .collect();

            let mut tournament_winner = None;
            if t.state == TournamentState::Done as u8
            {
                if let Ok(winner) = self.sql_one::<TournamentWinner, _>(
                    "select * from tournament_winners where tournament = ?1",
                    _params![t.id],
                )
                {
                    let winner = self
                        .get_user_without_matches_by("id", "=", &winner.player.to_string())
                        .unwrap();
                    tournament_winner = Some(winner.name);
                }
            }

            TournamentInfo {
                tournament: self.convert_tournament(t, tournament_winner).unwrap(),
                data:       TournamentInfoState::Games(players),
            }
        }
    }

    fn filter_tournaments(&self, t: &Tournament, info: &GetTournamentOptions) -> bool
    {
        if let Some(s) = &info.query
        {
            match s.as_str()
            {
                "old" => t.state == TournamentState::Done as u8,
                "active" => t.state != TournamentState::Done as u8,
                _ => true,
            }
        }
        else
        {
            true
        }
    }

    pub fn get_tournaments(&self, info: GetTournamentOptions) -> ServerResult<Vec<TournamentInfo>>
    {
        let tournaments = self.sql_many::<Tournament, _>("select * from tournaments", None)?;
        let t_infos = tournaments
            .into_iter()
            .filter(|t| self.filter_tournaments(t, &info))
            .map(|t| self.map_tournament_info(t))
            .collect::<Vec<TournamentInfo>>();
        Ok(t_infos)
    }

    fn get_is_organizer(&self, token: String, tid: i64) -> ServerResult<bool>
    {
        let pid = self.get_user_without_matches_by("uuid", "=", &token)?.id;
        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid])?;
        Ok(pid == tournament.organizer)
    }

    pub fn delete_tournament(&self, token: String, tid: i64) -> ServerResult<()>
    {
        if !self.get_is_organizer(token, tid)?
        {
            return Err(ServerError::Unauthorized);
        }

        self._delete_tournament(tid)
    }

    fn _delete_tournament(&self, tid: i64) -> ServerResult<()>
    {
        let tournament = self
            .sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid])?;

        let delete_tournament = |tid: i64| -> ServerResult<()> {
            self.conn.execute("delete from tournaments where id = ?1", params![tid])?;
            Ok(())
        };

        // Can't delete finished tournament maybe?
        if tournament.state == TournamentState::Done as u8
        {
            return Err(ServerError::Tournament(TournamentError::WrongState));
        }
        else if tournament.state == TournamentState::Created as u8
        {
            self.delete_tourament_list(tid)?;
        }
        else
        {
            let games = self.sql_many::<TournamentGame, _>(
                "select * from tournament_games where tournament = ?1",
                _params![tid],
            )?;
            for game in games
            {
                self.conn
                    .execute("delete from tournament_matches where game = ?1", params![game.id])?;
            }
            self.conn
                .execute("delete from tournament_games where tournament = ?1", params![tid])?;
        }
        delete_tournament(tid)?;
        Ok(())
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

        let id_1 =
            s._create_tournament(1, "epic".to_string(), 3, 4, TournamentType::SingleElimination);
        let id_2 =
            s._create_tournament(1, "epic_2".to_string(), 3, 8, TournamentType::SingleElimination);
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(id_1.is_ok());
        assert!(id_2.is_ok());
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


        s._create_tournament(1, "Epic".to_string(), 3, 4, TournamentType::SingleElimination)
            .unwrap();
        let res = s.join_tournament(token, 1);

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


        s._create_tournament(1, "Epic".to_string(), 3, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token.clone(), 1);
        let res = s.join_tournament(token, 1);

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


        let tid = s
            ._create_tournament(1, "Epic".to_string(), 3, 4, TournamentType::SingleElimination)
            .unwrap();
        let rs = s.join_tournament(token_s, 1);
        let rb = s.join_tournament(token_b, 1);
        let rm = s.join_tournament(token_m, 1);
        let re = s.join_tournament(token_e, 1);

        let tournament = s.sql_one::<Tournament, _>("select * from tournaments", None).unwrap();
        let games =
            s.sql_many::<TournamentGame, _>("select * from tournament_games", None).unwrap();
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


        let tid = s
            ._create_tournament(1, "Epic".to_string(), 3, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token_s, 1);
        let _ = s.join_tournament(token_b, 1);
        let _ = s.join_tournament(token_m, 1);
        let re = s.join_tournament(token_e, 1);
        let rl = s.join_tournament(token_l, 1);

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

        let vec4: Vec<i64> = (1..=4).collect();
        let vec8: Vec<i64> = (1..=8).collect();
        let vec16: Vec<i64> = (1..=16).collect();
        let vec = vec![vec4, vec8, vec16];
        let tournament = Tournament {
            id:           0,
            state:        0,
            ttype:        0,
            player_count: 0,
            name:         String::new(),
            prize:        0,
            organizer:    0,
        };

        let vec_ok = |vec| {
            s.generate_buckets(&tournament, &vec)
                .into_iter()
                .rev()
                .take(vec.len() / 2)
                .all(|g| g.player1 != 0 && g.player2 != 0)
        };

        assert!(vec.into_iter().all(vec_ok));
    }

    #[test]
    fn test_generate_bucket_not_of_power_of_two()
    {
        let db_file = "tempT11.db";
        let s = DataBase::new(db_file);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        let tournament = Tournament {
            id:           0,
            state:        0,
            ttype:        0,
            player_count: 0,
            name:         String::new(),
            prize:        0,
            organizer:    0,
        };
        let tid = tournament.id;



        // First check vec5
        let vec5: Vec<i64> = (1..=5).collect();
        let vec5_ans = vec![
            //final
            TournamentGame::players(tid, 0, 0, 0),
            // Semis
            TournamentGame::players(tid, 1, 0, 2),
            TournamentGame::players(tid, 2, 3, 4),
            // playoffs
            TournamentGame::players(tid, 3, 1, 5),
            TournamentGame::players(tid, 4, 0, 0),
            TournamentGame::players(tid, 5, 0, 0),
            TournamentGame::players(tid, 6, 0, 0),
        ];
        let gen5 = s.generate_buckets(&tournament, &vec5);
        assert_eq!(gen5, vec5_ans);

        // Check vec13
        let vec13: Vec<i64> = (1..=13).collect();
        let vec13_ans = vec![
            TournamentGame::players(tid, 0, 0, 0),
            TournamentGame::players(tid, 1, 0, 0),
            TournamentGame::players(tid, 2, 0, 0),
            TournamentGame::players(tid, 3, 0, 0),
            TournamentGame::players(tid, 4, 0, 0),
            TournamentGame::players(tid, 5, 0, 6),
            TournamentGame::players(tid, 6, 7, 8),
            TournamentGame::players(tid, 7, 1, 9),
            TournamentGame::players(tid, 8, 2, 10),
            TournamentGame::players(tid, 9, 3, 11),
            TournamentGame::players(tid, 10, 4, 12),
            TournamentGame::players(tid, 11, 5, 13),
            TournamentGame::players(tid, 12, 0, 0),
            TournamentGame::players(tid, 13, 0, 0),
            TournamentGame::players(tid, 14, 0, 0),
        ];

        let gen13 = s.generate_buckets(&tournament, &vec13);
        assert_eq!(gen13, vec13_ans);

        // Check vec6
        let vec6: Vec<i64> = (1..=6).collect();
        let vec6_ans = vec![
            //final
            TournamentGame::players(tid, 0, 0, 0),
            // Semis
            TournamentGame::players(tid, 1, 0, 0),
            TournamentGame::players(tid, 2, 3, 4),
            // playoffs
            TournamentGame::players(tid, 3, 1, 5),
            TournamentGame::players(tid, 4, 2, 6),
            TournamentGame::players(tid, 5, 0, 0),
            TournamentGame::players(tid, 6, 0, 0),
        ];
        let gen6 = s.generate_buckets(&tournament, &vec6);
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


        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        create_tournament_image(&s);
        let _ = s.join_tournament(token_s.clone(), 1);
        let _ = s.join_tournament(token_b.clone(), 1);
        let _ = s.join_tournament(token_m, 1);
        let _ = s.join_tournament(token_e, 1);

        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token_s, 2);
        let _ = s.join_tournament(token_b, 2);

        let info = GetTournamentOptions {
            query: None
        };
        let tournaments = s.get_tournaments(info);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(tournaments.is_ok());
        let mut tournaments = tournaments.unwrap();
        let first = tournaments.remove(0);
        let last = tournaments.remove(0);


        let assert_func = |t: TournamentInfo| {
            match t.data
            {
                TournamentInfoState::Games(vec) =>
                {
                    assert_eq!(t.tournament.name, "Epic");
                    assert_eq!(vec.len(), 3);
                },
                TournamentInfoState::Players(vec) =>
                {
                    assert_eq!(t.tournament.name, "Epic");
                    assert_eq!(vec.len(), 2);
                },
                _ => unreachable!(),
            };
        };
        assert_func(first);
        assert_func(last);
    }

    fn reg_tournament_match_from_tournament_game(
        s: &DataBase,
        game: &TournamentGame,
        token: String,
    ) -> RegisterTournamentMatch
    {
        let winner_name = s
            .get_user_without_matches_by("id", "=", &game.player1.to_string())
            .expect(&format!("{:?}", game))
            .name;
        let loser_name = s
            .get_user_without_matches_by("id", "=", &game.player2.to_string())
            .unwrap()
            .name;

        RegisterTournamentMatch {
            tournament_game: game.id,
            winner:          winner_name,
            loser:           loser_name,
            organizer_token: token,
        }
    }
    #[test]
    fn test_can_register_tournament_match()
    {
        let db_file = "tempT13.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        s._create_tournament(1, "Epic".to_string(), 3, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token_s.clone(), 1);
        let _ = s.join_tournament(token_b.clone(), 1);
        let _ = s.join_tournament(token_m, 1);
        let _ = s.join_tournament(token_e, 1);

        let games = s.get_all_tournament_games(1).unwrap();
        let game_one = &games[1];


        let register_game =
            reg_tournament_match_from_tournament_game(&s, &games[1], token_s.clone());


        let winner_id = game_one.player1;

        let res = s.register_tournament_match(register_game);
        let new_games = s.get_all_tournament_games(1).unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res.is_ok());
        let mut game = TournamentGame::players(1, 0, winner_id, 0);
        game.id = 1;
        assert_eq!(new_games[0], game);
    }

    #[test]
    fn test_can_finish_tournament()
    {
        let db_file = "tempT14.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        create_tournament_image(&s);
        let _ = s.join_tournament(token_s.clone(), 1);
        let _ = s.join_tournament(token_b.clone(), 1);
        let _ = s.join_tournament(token_m, 1);
        let _ = s.join_tournament(token_e, 1);

        let games = s.get_all_tournament_games(1).unwrap();

        let first_game = reg_tournament_match_from_tournament_game(&s, &games[1], token_s.clone());

        let second_game = reg_tournament_match_from_tournament_game(&s, &games[2], token_s.clone());

        let winner = first_game.winner.clone();

        let _final = RegisterTournamentMatch {
            tournament_game: games[0].id,
            winner:          first_game.winner.clone(),
            loser:           second_game.winner.clone(),
            organizer_token: token_s.clone(),
        };

        let res1 = s.register_tournament_match(first_game);
        let res2 = s.register_tournament_match(second_game);
        let res3 = s.register_tournament_match(_final);


        let winner = s.get_user(&winner);
        let info = GetTournamentOptions {
            query: None
        };
        let tournaments = s.get_tournaments(info);

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res1.is_ok());
        assert!(res2.is_ok());
        assert!(res3.is_ok());
        assert!(tournaments.is_ok());

        assert!(winner.is_ok());
        let tournament = &tournaments.unwrap()[0];
        let winner = winner.unwrap();

        assert_eq!(winner.badges.len(), 1);
        assert_eq!(tournament.tournament.state, TournamentState::Done as u8);
    }

    #[test]
    fn test_cannot_register_same_game_twice()
    {
        let db_file = "tempT15.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token_s.clone(), 1);
        let _ = s.join_tournament(token_b.clone(), 1);
        let _ = s.join_tournament(token_m, 1);
        let _ = s.join_tournament(token_e, 1);

        let games = s.get_all_tournament_games(1).unwrap();

        let first_game = reg_tournament_match_from_tournament_game(&s, &games[1], token_s.clone());

        let res1 = s.register_tournament_match(first_game.clone());
        let res2 = s.register_tournament_match(first_game);

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(res1.is_ok());
        assert!(
            res2.is_err()
                && res2.unwrap_err() == ServerError::Tournament(TournamentError::GameAlreadyPlayed)
        );
    }

    #[test]
    fn test_cannot_register_invalid_game()
    {
        let db_file = "tempT16.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");


        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        let _ = s.join_tournament(token_s.clone(), 1);
        let _ = s.join_tournament(token_b.clone(), 1);
        let _ = s.join_tournament(token_m, 1);
        let _ = s.join_tournament(token_e, 1);

        let games = s.get_all_tournament_games(1).unwrap();

        let invalid = RegisterTournamentMatch {
            tournament_game: games[0].id,
            winner:          String::from(""),
            loser:           String::from(""),
            organizer_token: token_s.clone(),
        };

        let res1 = s.register_tournament_match(invalid.clone());

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(
            res1.is_err()
                && res1.unwrap_err() == ServerError::Tournament(TournamentError::InvalidGame)
        );
    }
    #[test]
    fn can_delete_tournament()
    {
        let db_file = "tempT17.db";
        let s = DataBase::new(db_file);

        let token_s = create_user(&s, "Sivert");
        let token_b = create_user(&s, "Bernt");
        let token_m = create_user(&s, "Markus");
        let token_e = create_user(&s, "Ella");

        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::SingleElimination)
            .unwrap();
        s.join_tournament(token_s.clone(), 1);
        s.join_tournament(token_b.clone(), 1);
        s.join_tournament(token_m, 1);
        s.join_tournament(token_e, 1);
        let tid = 1;

        let games = s.get_all_tournament_games(1).unwrap();
        let first_game = reg_tournament_match_from_tournament_game(&s, &games[1], token_s.clone());

        s.register_tournament_match(first_game.clone());

        s._delete_tournament(1).unwrap();

        let games = s
            .sql_many::<TournamentGame, _>(
                "select * from tournament_games where tournament = ?1",
                _params![tid],
            )
            .unwrap();

        let mut matches = Vec::new();
        for game in &games
        {
            if let Ok(ma) = s.sql_one::<TournamentMatch, _>(
                "select * from tournament_matches where id = ?1",
                _params![game.id],
            )
            {
                matches.push(ma);
            }
        }
        let tournament =
            s.sql_one::<Tournament, _>("select * from tournaments where id = ?1", _params![tid]);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(tournament.is_err());
        assert_eq!(games.len(), 0);
        assert_eq!(matches.len(), 0);
    }


    #[test]
    fn create_losers_bracket_correct_length()
    {
        let db_file = "tempT18";
        let s = DataBase::new(db_file);
        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::DoubleElimination)
            .unwrap();

        s.create_losers_bracket(8, 1, Vec::new()).unwrap();

        let games = s
            .sql_many::<TournamentGame, _>("select * from tournament_games where bucket < 0", None)
            .unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert_eq!(games.len(), 6);
    }

    #[test]
    fn can_create_double_elimination_tournament()
    {
        let db_file = "tempT19";
        let s = DataBase::new(db_file);
        s._create_tournament(1, "Epic".to_string(), 1, 4, TournamentType::DoubleElimination)
            .unwrap();

        s.create_losers_bracket(8, 1, Vec::new()).unwrap();
        let tournament = s.sql_one::<Tournament, _>("select * from tournaments", None).unwrap();
        std::fs::remove_file(db_file).expect("Removing file tempH");
        let ttype: TournamentType = tournament.ttype.into();
        assert_eq!(ttype, TournamentType::DoubleElimination);
    }

    #[test]
    fn can_run_double_elimination_tournament() -> ServerResult<()>
    {
        let run_tournament = |s: &DataBase, player_count: i64, id: i64| {
            let run_game = |i: isize, token: &String| {
                let game = s
                    .get_all_tournament_games(id)
                    .unwrap()
                    .into_iter()
                    .find(|g| g.bucket == i as i64)
                    .expect(&format!("Grabbing bucket {}", i));

                let register_game =
                    reg_tournament_match_from_tournament_game(&s, &game, token.clone());

                let res1 = s.register_tournament_match(register_game);
                res1
            };
            let users: Vec<String> = if player_count == 4
            {
                (1..=player_count).map(|n| create_user(s, &n.to_string())).collect()
            }
            else
            {
                let get_token = |n: i64| -> ServerResult<String> {
                    use rusqlite::NO_PARAMS;

                    use crate::SQL_TUPLE;
                    let res: Vec<_> =
                        SQL_TUPLE!(s, &format!("select uuid from users where id = {}", n), String)
                            .unwrap();
                    Ok(res.first().unwrap().0.clone())
                };
                (1..=(player_count / 2))
                    .map(|n| get_token(n).expect("Getting token"))
                    .chain(
                        ((player_count / 2) + 1..=player_count)
                            .map(|n| create_user(s, &n.to_string())),
                    )
                    .collect()
            };


            let token = users.first().clone().unwrap();
            let create_tournament = CreateTournament {
                organizer_token: token.clone(),
                name:            player_count.to_string(),
                image:           "".to_string(),
                player_count:    player_count,
                ttype:           "doubleElimination".to_string(),
            };

            s.create_tournament(create_tournament).expect("Creating tournament");

            users.iter().for_each(|token| {
                s.join_tournament(token.clone(), id).expect("joining tournament");
            });

            let games: Vec<isize> = (0..=player_count - 2)
                .rev()
                .map(|i| i as isize)
                .chain((1..=player_count - 2).rev().map(|i| -(i as isize)))
                .chain(vec![player_count as isize]) // final
                .collect();

            games.into_iter().map(|g| run_game(g, &token)).all(|r| r.is_ok())
        };
        let db_file = "tempT20.db";
        let s = DataBase::new(db_file);


        for (id, p) in (2..=6).enumerate()
        {
            let pow = 2_i64.pow(p as u32);
            assert_eq!(run_tournament(&s, pow, (id + 1) as i64), true);
        }

        std::fs::remove_file(db_file).expect("Removing file tempH");
        Ok(())
    }


    #[test]
    fn loser_bracket_parent_function()
    {
        let db_file = "tempT21.db";
        let db = DataBase::new(db_file);

        let test_size = |n| // n: e.g 32
        {
            if n == 4
            {
                return db.loser_bracket_parent(2, 4) == -1;
            }
            let stop = (n / 2) - 2;
            let start = stop - (n / 8) + 1;

            let mut vec = Vec::new();
            let mut s = (n / 2 - 1);
            for i in start..=stop
            {
                for _ in 0..2
                {
                    vec.push(db.loser_bracket_parent(s, n) == -i);
                    s += 1;
                }
            }

            for i in (stop + 1)..(stop + 1) + n / 4
            {
                vec.push(db.loser_bracket_parent(s, n) == -i);
                s += 1;
            }
            vec.into_iter().all(|b| b)
        };
        std::fs::remove_file(db_file).expect("Removing file tempH");

        for i in 2..=6
        {
            let pow = 2_i64.pow(i as u32);
            assert!(test_size(pow));
        }
    }

    #[test]
    fn can_forward_games_in_double_elimination()
    {
        let db_file = "tempT22.db";
        let s = DataBase::new(db_file);
        let users = (1..16).map(|n| create_user(&s, &n.to_string())).collect::<Vec<String>>();

        let token = users.first().clone().unwrap();
        let create_tournament = CreateTournament {
            organizer_token: token.clone(),
            name:            "Epic".to_string(),
            image:           "".to_string(),
            player_count:    15,
            ttype:           "doubleElimination".to_string(),
        };

        s.create_tournament(create_tournament).expect("Creating tournament");

        users.iter().for_each(|token| {
            s.join_tournament(token.clone(), 1).expect("joining tournament");
        });



        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(false);
    }
}
