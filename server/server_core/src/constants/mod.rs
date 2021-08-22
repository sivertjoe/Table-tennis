pub const NUM_SEASON_PRIZES: usize = 3;
pub static BADGES: &'static [&'static str] =
    &["first_place.png", "second_place.png", "third_place.png", "default_tournament_win.png"];

#[allow(dead_code)]
pub const MATCH_NO_ANS: u8 = 0;
pub const ACCEPT_REQUEST: u8 = 1;
#[allow(dead_code)]
const DECLINE_REQUEST: u8 = 2;

pub const STOP_SEASON: i64 = -1;
pub const START_SEASON: i64 = -2;
pub const CANCEL_SEASON: i64 = -3;

pub const N_SEASON_ID: u32 = 1;
pub const IS_SEASON_ID: u32 = 2;
pub const REQUIRE_CONFIRMATION_ID: u32 = 3;

pub const USER_ROLE_REGULAR: u8 = 0;
pub const USER_ROLE_SUPERUSER: u8 = 1 << 1;
pub const USER_ROLE_INACTIVE: u8 = 1 << 2;
pub const USER_ROLE_SOFT_INACTIVE: u8 = 1 << 4;

pub const DATABASE_FILE: &'static str = "db.db";

pub const TOURNAMENT_BADGES_PATH: &'static str = "tournament_badges";

pub const INVALID_TOKEN: i64 = 0;
pub const VALID_TOKEN: i64 = 1;
