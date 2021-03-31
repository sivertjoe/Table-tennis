use std::sync::{Arc, Mutex};

use chrono::prelude::*;
use server_core::constants::DATABASE_FILE;

use super::{season::Season, server::DataBase};

fn get_ars(data: &Arc<Mutex<DataBase>>) -> (bool, i64, u32)
{
    let s = data.lock().expect("Getting mutex");

    // Assume this means that this is the first season
    let season =
        s.get_latest_season().unwrap().unwrap_or_else(|| Season {
            id:          0,
            start_epoch: Utc::now().timestamp_millis(),
        });
    let start_month = Utc.timestamp_millis(season.start_epoch).month0();
    let running = s.get_is_season().expect("Getting running season");
    let season_length = s.get_season_length().unwrap();
    (running, season_length, start_month)
}

fn start_new_season(data: &Arc<Mutex<DataBase>>)
{
    let s = data.lock().expect("Getting mutex");
    s.end_season().expect("Endig season");
    s.start_new_season().expect("starting new season");
}

fn backup(data: &Arc<Mutex<DataBase>>)
{
    let s = data.lock().expect("Getting mutex");
    std::fs::copy(DATABASE_FILE, format!("db/backup/{}.db", Utc::now().format("%d_%m_%y_%H%M%S")))
        .expect("Backing DB up");
    drop(s);
}

pub fn spawn_season_checker(data: Arc<Mutex<DataBase>>)
{
    std::thread::spawn(move || {
        loop
        {
            let now = Utc::now();
            let next_3 = now.date().succ().and_hms(3, 0, 0);
            let duration = next_3.signed_duration_since(now).to_std().unwrap();
            std::thread::sleep(duration);
            backup(&data);

            let (is_season, len, start) = get_ars(&data);
            if is_season && Utc::now().month0() == (start + len as u32) % 12
            {
                start_new_season(&data);
            }
        }
    });
}
