use crate::server::DataBase;
use std::collections::HashMap;
use chrono::prelude::*;
use crate::r#match::Match;
use rusqlite::{Connection, Result, named_params};
use elo::EloRank;


impl DataBase
{
    pub fn roll_back(&self) -> Result<usize>
    {
        let elo = EloRank { k: 32 };
        let time_now = Utc::now().timestamp_millis();
        let one_day = time_now - 86400000;
        
                            // Match, id
        let mut modified: Vec<(Match, i64)> = Vec::new();

                         // Username, elo
        let mut map: HashMap<String, f64> = HashMap::new();

        for (m, id) in get_all_matches_before(&self.conn, one_day)?
        {
            let winner_elo = *map.entry(m.winner.clone()).or_insert(m.winner_elo - m.elo_diff);
            let loser_elo = *map.entry(m.loser.clone()).or_insert(m.loser_elo + m.elo_diff);

            let (new_winner_elo, new_loser_elo) = elo.calculate(winner_elo, loser_elo);

            map.insert(m.winner.clone(), new_winner_elo);
            map.insert(m.loser.clone(), new_loser_elo);
            modified.push(
                (create_match(m, new_winner_elo, new_loser_elo, new_winner_elo - winner_elo), 
                 id)
            );
        }

        for (name, elo) in map
        {
            update_elo(&self.conn, name, elo)?;
        }

        for m in modified
        {
            update_match(&self.conn, m)?;
        }

        Ok(0)
    }

}

fn update_match(s: &Connection, m: (Match, i64)) -> Result<usize>
{
    let id = m.1;
    let m = m.0;
    let mut stmt = s.prepare("update matches  
                                set winner_elo = :w_elo,
                                    loser_elo = :l_elo,
                                    elo_diff = :diff
                                where id = :id")?;
    stmt.execute_named(named_params!{":w_elo": m.winner_elo, ":l_elo": m.loser_elo, ":diff": m.elo_diff, ":id": id})
}

fn update_elo(s: &Connection, name: String, elo: f64) -> Result<usize>
{
    let mut stmt = s.prepare("update users  set elo = :elo WHERE name like :name")?;
    stmt.execute_named(named_params!{":elo": elo, ":name": name})
}

fn create_match(mut m: Match, winner_new_elo: f64, loser_new_elo: f64, elo_diff: f64) -> Match
{
    m.winner_elo = winner_new_elo;
    m.loser_elo = loser_new_elo;
    m.elo_diff = elo_diff;
    m
}

fn get_all_matches_before(s: &Connection, time: i64) -> Result<Vec<(Match, i64)>>
{
    let zin = "select a.name, b.name, m.id, m.elo_diff, m.winner_elo, m.loser_elo, m.epoch
            from matches as m
            inner join users as a on a.id = m.winner
            inner join users as b on b.id = m.loser
            where epoch >= :epoch";
    let mut stmt = s.prepare(zin)?;
    let matches = stmt.query_map_named(named_params!{":epoch" : time}, |row|
    {
        let id: i64 = row.get(2)?;
        Ok((Match {
            winner: row.get(0)?,
            loser: row.get(1)?,
            elo_diff: row.get(3)?,
            winner_elo: row.get(4)?,
            loser_elo: row.get(5)?,
            epoch: row.get(6)?,
        }, id))
    })?;

    let mut vec = Vec::new();
    for m in matches
    {
        if let Ok((u, id)) = m
        {
            vec.push((u, id));
        };
    }
    vec.sort_by(|a, b| a.0.epoch.partial_cmp(&b.0.epoch).unwrap());
    Ok(vec)
}