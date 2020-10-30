use std::fmt;
use crate::r#match::Match;
use itertools::join;

pub struct User 
{
    pub id: i64, 
    pub elo: f64,
    pub name: String,
    pub match_history: Vec<Match>
}

fn get_matches_string(matches: &Vec<Match>) -> String
{
    join(matches.iter().map(|m| m.to_string()), "\n\t")
}

fn calculate_kdp(matches: &Vec<Match>, name: &String) -> (u32, u32, f32)
{
    let (mut wins, mut loss) = (0, 0);
    matches
        .iter()
        .for_each(|m|
        {
            if &m.winner == name
            {
                wins += 1;
            }
            else
            {
                loss += 1;
            }
        });

    let per = wins as f32 / (wins + loss) as f32;
    (wins, loss, per)
}

impl fmt::Display for User
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let (w, l, p) = calculate_kdp(&self.match_history, &self.name);
        write!(f, 
               "Username: {}\n\
               Elo: {}\n\
               K/D: {}\n\
               Win % : {}%\n\
               History: \n\t{}\n",
               self.name, 
               self.elo as u32, 
               format!("{}/{}", w, l),
               p,
               get_matches_string(&self.match_history))
    }
}
