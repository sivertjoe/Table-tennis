use rusqlite::{Connection, NO_PARAMS};

use crate::server::DataBase;

impl DataBase
{
    pub fn init(file: &str) -> Self
    {
        let conn = match Connection::open(file)
        {
            Err(_) => panic!("Could not create connection"),
            Ok(c) => c,
        };

        conn.execute(
            "create table if not exists users (
                id              integer primary key autoincrement,
                name            VARCHAR(20) not null unique,
                elo             float  default 1500.0,
                password_hash   varchar(64) not null,
                uuid            varchar(36) not null,
                user_role       smallint
                )",
            NO_PARAMS,
        )
        .expect("Creating user table");

        conn.execute(
            "create table if not exists matches (
                id              integer primary key autoincrement,
                epoch           bigint not null,
                elo_diff        integer,
                winner_elo      float,
                loser_elo       float,
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
            NO_PARAMS,
        )
        .expect("Creating matches table");

        conn.execute(
            "create table if not exists old_matches (
                id              integer primary key autoincrement,
                epoch           bigint not null,
                elo_diff        integer,
                winner_elo      float,
                loser_elo       float,
                winner          integer,
                loser           integer,
                season           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                foreign key(season) references seasons(id)
                )",
            NO_PARAMS,
        )
        .expect("Creating matches table");

        conn.execute(
            "create table if not exists offseason_matches (
                id              integer primary key autoincrement,
                epoch           bigint not null,
                elo_diff        integer,
                winner_elo      float,
                loser_elo       float,
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
            NO_PARAMS,
        )
        .expect("Creating matches table");

        conn.execute(
            "create table if not exists match_notification (
                id              integer primary key autoincrement,
                winner_accept   smallint default 0,
                loser_accept    smallint default 0,
                epoch           bigint not null,
                winner          integer,
                loser           integer,
                foreign key(winner) references users(id),
                foreign key(loser) references users(id)
                )",
            NO_PARAMS,
        )
        .expect("Creating match_notification table");

        conn.execute(
            "create table if not exists new_user_notification (
                id              integer primary key autoincrement,
                name            VARCHAR(20) not null unique,
                password_hash   varchar(64) not null
            )",
            NO_PARAMS,
        )
        .expect("Creating new_user_notification");

        conn.execute(
            "create table if not exists reset_password_notification (
                id                integer primary key autoincrement,
                user              integer not null unique,
                foreign key(user) references users(id)
            )",
            NO_PARAMS,
        )
        .expect("Creating reset_password_notification");

        conn.execute(
            "create table if not exists badges (
                id              integer primary key autoincrement,
                season_id       integer,
                badge_index     integer,
                pid             integer,
                foreign key(pid) references users(id),
                foreign key(season_id) references seasons(id)
            )",
            NO_PARAMS,
        )
        .expect("Creating badges table");

        conn.execute(
            "create table if not exists tournament_badges (
                id              integer primary key autoincrement,
                image           integer,
                pid             integer,
                foreign key(pid) references users(id),
                foreign key(image) references images(id)
            )",
            NO_PARAMS,
        )
        .expect("Creating badges table");


        conn.execute(
            "create table if not exists seasons (
                id              integer primary key autoincrement,
                start_epoch     integer
            )",
            NO_PARAMS,
        )
        .expect("Creating season table");

        conn.execute(
            "create table if not exists variables (
                id              integer primary key,
                value           integer not null
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        conn.execute(
            "create table if not exists tournaments (
                id              integer primary key autoincrement,
                name            varchar(36),
                prize           integer,
                state           smallint,
                player_count    integer,
                organizer       integer
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        conn.execute(
            "create table if not exists tournament_lists (
                id                      integer primary key autoincrement,
                player                  integer,
                tournament              integer,
                foreign key(player)     references users(id),
                foreign key(tournament) references tournaments(id)
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        conn.execute(
            "create table if not exists tournament_games (
                id                      integer primary key autoincrement,
                bucket                   integer,
                player1                  integer,
                player2                  integer,
                tournament              integer,
                foreign key(tournament) references tournaments(id)
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        // This is the result of a tournament_game
        conn.execute(
            "create table if not exists tournament_matches (
                id                      integer primary key autoincrement,
                game                    integer,
                winner                  integer,
                loser                   integer,
                foreign key(game) references tournament_games(id)
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        conn.execute(
            "create table if not exists tournament_winners (
                id                      integer primary key autoincrement,
                player                  integer,
                tournament              integer,
                foreign key(player)     references users(id),
                foreign key(tournament) references tournaments(id)
            )",
            NO_PARAMS,
        )
        .expect("Create variables table");

        conn.execute(
            "create table if not exists images (
                id              integer primary key autoincrement,
                name            varchar(10) not null
            )",
            NO_PARAMS,
        )
        .expect("Create variables images");



        DataBase {
            conn: conn
        }
    }
}
