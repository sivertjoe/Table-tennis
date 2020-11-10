use crate::server::DataBase;
use std::collections::HashMap;
use chrono::prelude::*;

impl DataBase
{
    pub fn roll_back(&self)
    {
        let time_now = Utc::now().timestamp_millis();
        let one_day = 86400000;

        println!("{}", time_now - one_day);
    }
}
