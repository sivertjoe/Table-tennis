use std::fmt;

#[derive(Debug)]
pub struct Match
{
    pub winner: String,
    pub loser: String,
}


impl fmt::Display for Match
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "Winner: {}, Loser: {}", self.winner, self.loser)
    }
}

