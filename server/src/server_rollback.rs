use crate::server::DataBase;
use std::collections::HashMap;
use crate::r#match::Match;
use rusqlite::{Connection, NO_PARAMS, named_params};
use elo::EloRank;
use crate::server::ServerResult;

impl DataBase
{
    pub fn roll_back(&self, time: i64) -> ServerResult<()>
    {
        self.conn.execute("BEGIN TRANSACTION;", NO_PARAMS)?;
        if let Err(e) = self._roll_back(time)
        {
            self.conn.execute("ROLLBACK;", NO_PARAMS)?;
            return Err(e);
        }

        self.conn.execute("COMMIT;", NO_PARAMS)?;
        Ok(())
    }

    fn _roll_back(&self, time: i64) -> ServerResult<()>
    {
        let elo = EloRank { k: 32 };
        
                            // Match, id
        let mut modified: Vec<(Match, i64)> = Vec::new();

                         // Username, elo
        let mut map: HashMap<String, f64> = HashMap::new();

        let flag = time < 0;
        let time = time.abs();
        let default_score = |m: &Match, is_winner: bool| -> f64
        { 
            if flag { 1500.0 } 
            else 
            { 
                if is_winner 
                { 
                    m.winner_elo - m.elo_diff 
                }
                else 
                { 
                    m.loser_elo + m.elo_diff
                }
            }
        };

        let initial_elo = |name: &String, matches: &Vec<(Match, i64)>| ->  f64
        {
            if flag { 1500.0 }
            else { get_initial_elo(name, matches) }
        };

        let matches = get_all_matches_before(&self.conn, time)?;
        map.insert(matches[0].0.winner.clone(), initial_elo(&matches[0].0.winner, &matches));
        map.insert(matches[0].0.loser.clone(), initial_elo(&matches[0].0.loser, &matches));

        for (m, id) in matches
        {
            let winner_name = m.winner.clone();
            let loser_name = m.loser.clone();
            let winner_elo = *map.entry(winner_name.clone()).or_insert(default_score(&m, true));
            let loser_elo = *map.entry(loser_name.clone()).or_insert(default_score(&m, false));

            let (new_winner_elo, new_loser_elo) = elo.calculate(winner_elo, loser_elo);

            map.insert(winner_name, new_winner_elo);
            map.insert(loser_name, new_loser_elo);
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

        Ok(())
    }

}

fn get_initial_elo(name: &String, matches: &Vec<(Match, i64)>) -> f64
{
    match matches.iter()
        .skip(1)
        .find(|(m, _)| &m.winner == name || &m.loser == name)
    {	
        Some((m, _)) => if &m.winner == name
        {
            m.winner_elo - m.elo_diff
        }
        else
        {
            m.loser_elo + m.elo_diff
        },

        None => if &matches[0].0.winner == name
        {	    
            matches[0].0.winner_elo - matches[0].0.elo_diff
        }
        else
        {
            matches[0].0.loser_elo + matches[0].0.elo_diff
        },
    }
}

fn update_match(s: &Connection, m: (Match, i64)) -> ServerResult<()>
{
    let id = m.1;
    let m = m.0;
    let mut stmt = s.prepare("update matches  
                                set winner_elo = :w_elo,
                                    loser_elo = :l_elo,
                                    elo_diff = :diff
                                where id = :id")?;
    stmt.execute_named(named_params!{":w_elo": m.winner_elo, ":l_elo": m.loser_elo, ":diff": m.elo_diff, ":id": id})?;
    Ok(())
}

fn update_elo(s: &Connection, name: String, elo: f64) -> ServerResult<()>
{
    let mut stmt = s.prepare("update users  set elo = :elo WHERE name like :name")?;
    stmt.execute_named(named_params!{":elo": elo, ":name": name})?;
    Ok(())
}

fn create_match(mut m: Match, winner_new_elo: f64, loser_new_elo: f64, elo_diff: f64) -> Match
{
    m.winner_elo = winner_new_elo;
    m.loser_elo = loser_new_elo;
    m.elo_diff = elo_diff;
    m
}

fn get_all_matches_before(s: &Connection, time: i64) -> ServerResult<Vec<(Match, i64)>>
{
    let zin = "select a.name, b.name, m.id, m.elo_diff, m.winner_elo, m.loser_elo, m.epoch
            from matches as m
            inner join users as a on a.id = m.winner
            inner join users as b on b.id = m.loser
            where epoch >= :epoch
            order by epoch;";
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
    Ok(vec)
}
