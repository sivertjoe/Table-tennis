use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Serialize;
use serde_derive::Deserialize;
use serde_json::json;
use server::{
    spawn_season_checker, ChangePasswordInfo, CreateTournament, DataBase, DeleteMatchInfo,
    EditUsersInfo, GetTournamentOptions, JoinTournament, LoginInfo, MatchInfo, NewEditMatchInfo,
    NotificationAns, NotificationInfo, NotificationType, RegisterTournamentMatch,
    RequestResetPassword, StatsUsers,
};
use server_core::{
    constants::{CANCEL_SEASON, START_SEASON, STOP_SEASON},
    types::{ServerError, TournamentError::*},
};

const PORT: u32 = 58642;
pub const DATABASE_FILE: &str = "db.db";


macro_rules! DATABASE {
    ($data:expr) => {
        match $data.get_ref().lock()
        {
            Ok(guard) => guard,
            Err(p_err) => p_err.into_inner(),
        }
    };
}

fn response_code(e: ServerError) -> u8
{
    println!("{:?}", e);
    match e
    {
        ServerError::Critical(_) => 1,
        ServerError::UserNotExist => 2,
        ServerError::UsernameTaken => 3,
        ServerError::WrongUsernameOrPassword => 4,
        ServerError::PasswordNotMatch => 5,
        ServerError::Unauthorized => 6,
        ServerError::WaitingForAdmin => 7,
        ServerError::InactiveUser => 8,
        ServerError::ResetPasswordDuplicate => 9,
        ServerError::InvalidUsername => 10,
        ServerError::Tournament(t) => match t
        {
            WrongState => 11,
            NoTournament => 12,
            NotOrganizer => 13,
            InvalidGame => 14,
            WrongTournamentCount => 15,
            AlreadyJoined => 16,
            GameAlreadyPlayed => 17,
            InvalidTtype => 18,
            CannotRerun => 19,
        },
        _ => 69,
    }
}

fn response_error(e: ServerError) -> serde_json::Value
{
    json!({ "status": response_code(e) })
}

fn response_ok_with<T>(item: T) -> serde_json::Value
where
    T: Serialize,
{
    json!({"status": 0, "result": item})
}

fn response_ok() -> serde_json::Value
{
    json!({"status": 0})
}


#[post("/create-user")]
async fn create_user(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: LoginInfo = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).create_user(info.username, info.password)
    {
        Ok(s) => HttpResponse::Ok().json(response_ok_with(s)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct Variable
{
    variable: String,
}

#[post("admin/get-variable")]
async fn get_variable(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: Variable = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).get_variable(info.variable)
    {
        Ok(val) => HttpResponse::Ok().json(response_ok_with(val.to_string())),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct EditVariable
{
    variable: String,
    new_val:  String,
    token:    String,
}

#[post("admin/set-variable")]
async fn set_variable(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: EditVariable = serde_json::from_str(&info).unwrap();
    let new_val: i64 = info.new_val.parse().unwrap();
    match DATABASE!(data).set_variable(info.token, info.variable, new_val)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[post("/edit-users")]
async fn edit_users(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: EditUsersInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).edit_users(info.users, info.action, info.token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[post("/request-reset-password")]
async fn request_reset_password(data: web::Data<Arc<Mutex<DataBase>>>, info: String)
    -> HttpResponse
{
    let info: RequestResetPassword = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).request_reset_password(info.name)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).register_match(info.winner, info.loser, info.token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/login")]
async fn login(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: LoginInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).login(info.username, info.password)
    {
        Ok(uuid) => HttpResponse::Ok().json(response_ok_with(uuid)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/change-password")]
async fn change_password(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: ChangePasswordInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).change_password(info.username, info.password, info.new_password)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[get("/active-users")]
async fn get_active_users(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_non_inactive_users()
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_users()
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[get("/all-users/{token}")]
async fn get_all_users(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(token): web::Path<String>,
) -> HttpResponse
{
    match DATABASE!(data).get_all_users(token)
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}

#[get("/notifications")]
async fn get_notifications(
    data: web::Data<Arc<Mutex<DataBase>>>,
    info: web::Query<NotificationInfo>,
) -> HttpResponse
{
    let _type = info.r#type.clone();
    let token = info.token.clone();

    NotificationType::try_from(_type.clone()).map_or(
        HttpResponse::Ok().json(
            json!({"status": 69, "result": format!("no notification type matching {}", _type)}),
        ),
        |t| match DATABASE!(data).get_notifications(t, token)
        {
            Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
            Err(e) => HttpResponse::Ok().json(response_error(e)),
        },
    )
}

#[post("/notifications")]
async fn respond_to_notification(
    data: web::Data<Arc<Mutex<DataBase>>>,
    info: String,
) -> HttpResponse
{
    NotificationAns::try_from(info).map_or(
        HttpResponse::Ok().json(json!({"status": 69, "result": "invalid notification type"})),
        |notification_ans| match DATABASE!(data).respond_to_notification(notification_ans)
        {
            Ok(_) => HttpResponse::Ok().json(response_ok()),
            Err(e) => HttpResponse::Ok().json(response_error(e)),
        },
    )
}

#[get("/history")]
async fn get_history(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_history()
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/stats")]
async fn get_stats(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: StatsUsers = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).get_stats(info)
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("delete-match")]
async fn delete_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: DeleteMatchInfo = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).delete_match(info)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/edit-match")]
async fn edit_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: NewEditMatchInfo = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).edit_match(info)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/edit-history")]
async fn get_edit_history(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_edit_match_history()
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/user/{name}")]
async fn get_profile(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(name): web::Path<String>,
) -> HttpResponse
{
    match DATABASE!(data).get_user(&name)
    {
        Ok(data) => HttpResponse::Ok().json(response_ok_with(data)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct Users
{
    users: Vec<String>,
}

#[post("/get-multiple-users")]
async fn get_multiple_users(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: Users = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).get_multiple_users(info.users)
    {
        Ok(users) => HttpResponse::Ok().json(response_ok_with(users)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}


#[post("/create-tournament")]
async fn create_tournament(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: CreateTournament = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).create_tournament(info)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/recreate-tournament")]
async fn recreate_tournament(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    #[derive(Deserialize)]
    struct RecreateTournament
    {
        token: String,
        tid:   i64,
    }

    let info: RecreateTournament = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).recreate_tournament(info.token, info.tid)
    {
        Ok(id) => HttpResponse::Ok().json(response_ok_with(id)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/join-tournament")]
async fn join_tournament(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: JoinTournament = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).join_tournament(info.token, info.tid)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}
#[post("/register-tournament-match")]
async fn register_tournament_match(
    data: web::Data<Arc<Mutex<DataBase>>>,
    info: String,
) -> HttpResponse
{
    let info: RegisterTournamentMatch = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).register_tournament_match(info)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/delete-tournament")]
async fn delete_tournament(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    #[derive(Deserialize)]
    struct DeleteTournament
    {
        tid:   i64,
        token: String,
    }

    let info: DeleteTournament = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).delete_tournament(info.token, info.tid)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/leave-tournament")]
async fn leave_tournament(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: JoinTournament = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).leave_tournament(info.token, info.tid)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/tournament-infos")]
async fn get_tournament_infos(
    data: web::Data<Arc<Mutex<DataBase>>>,
    info: web::Query<GetTournamentOptions>,
) -> HttpResponse
{
    let info: GetTournamentOptions = info.into_inner();
    match DATABASE!(data).get_tournament_infos(info)
    {
        Ok(tournaments) => HttpResponse::Ok().json(response_ok_with(tournaments)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/tournament-table/{id}")]
async fn get_tournament_table(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(id): web::Path<i64>,
) -> HttpResponse
{
    match DATABASE!(data).get_upper_to_lower_table(id)
    {
        Ok(tournaments) => HttpResponse::Ok().json(response_ok_with(tournaments)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}
#[get("/tournament/{id}")]
async fn get_tournament(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(id): web::Path<i64>,
) -> HttpResponse
{
    match DATABASE!(data).get_tournament_from_id(id)
    {
        Ok(tournaments) => HttpResponse::Ok().json(response_ok_with(tournaments)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/is-admin/{token}")]
async fn get_is_admin(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(token): web::Path<String>,
) -> HttpResponse
{
    match DATABASE!(data).get_is_admin(token)
    {
        Ok(val) => HttpResponse::Ok().json(response_ok_with(val)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/admin/roll-back/{token}")]
async fn roll_back(
    data: web::Data<Arc<Mutex<DataBase>>>,
    web::Path(token): web::Path<String>,
) -> HttpResponse
{
    match DATABASE!(data).admin_rollback(token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[get("/season_length")]
async fn get_season_length(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_season_length()
    {
        Ok(n_months) => HttpResponse::Ok().json(response_ok_with(n_months)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct EditSeasonLength
{
    token:   String,
    new_val: i64,
}


#[post("/season_length")]
async fn set_season_length(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: EditSeasonLength = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).set_season_length(info.token, info.new_val)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct Token
{
    token: String,
}

fn change_season(data: web::Data<Arc<Mutex<DataBase>>>, info: String, val: i64) -> HttpResponse
{
    let info: Token = serde_json::from_str(&info).unwrap();

    let res = DATABASE!(data).get_is_admin(info.token);
    match res
    {
        Ok(true) =>
        {
            match val
            {
                START_SEASON =>
                {
                    DATABASE!(data).start_new_season().expect("Starting new season");
                },
                STOP_SEASON | CANCEL_SEASON =>
                {
                    DATABASE!(data).end_season(val == STOP_SEASON).expect("Ending season");
                },
                _ =>
                {},
            };

            HttpResponse::Ok().json(response_ok())
        },
        Ok(false) => HttpResponse::Ok().json(response_error(ServerError::Unauthorized)),

        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[post("/start_season")]
async fn start_season(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    change_season(data, info, START_SEASON)
}

#[post("/stop_season")]
async fn stop_season(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    change_season(data, info, STOP_SEASON)
}

#[post("/cancel_season")]
async fn cancel_season(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    change_season(data, info, CANCEL_SEASON)
}

#[get("/leaderboard_info")]
async fn get_leaderboard_info(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    let s = DATABASE!(data);
    match (s.get_users(), s.get_is_season(), s.get_latest_season_number())
    {
        (Ok(users), Ok(is_season), Ok(len)) => HttpResponse::Ok().json(response_ok_with(
            json!({"users": users, "is_season": is_season, "season_number": len}),
        )),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[get("season_start")]
async fn get_season_start_date(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_season_start()
    {
        Ok(date) => HttpResponse::Ok().json(response_ok_with(date)),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

#[derive(Deserialize)]
struct SqlCommand
{
    token:   String,
    command: String,
}

#[post("admin/execute-sql")]
async fn execute_sql(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: SqlCommand = serde_json::from_str(&info).unwrap();
    let s = DATABASE!(data);

    match s.get_is_admin(info.token.clone())
    {
        Ok(true) => match s.execute_sql(info.command)
        {
            Ok(string) => HttpResponse::Ok().json(response_ok_with(string)),
            Err(e) => HttpResponse::Ok().json(response_ok_with(&format!("{}", e))),
        },
        Ok(false) => HttpResponse::InternalServerError().json(json!({"status": 5, "result": ""})),
        Err(e) => HttpResponse::Ok().json(response_error(e)),
    }
}

fn get_builder() -> openssl::ssl::SslAcceptorBuilder
{
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("db/privkey.pem", SslFiletype::PEM)
        .expect("failed to open/read key.pem");
    builder
        .set_certificate_chain_file("db/fullchain.pem")
        .expect("failed to open/read cert.pem");
    builder
}


fn handle_args(data: &Arc<Mutex<DataBase>>)
{
    let vec = std::env::args().collect::<Vec<String>>();
    match vec.as_slice()
    {
        [_, flag, name] =>
        {
            // Make user admin
            if flag.as_str() == "-a"
            {
                data.lock().unwrap().create_superuser(name.clone()).unwrap();
            }
        },
        _ =>
        {},
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let data = Arc::new(Mutex::new(DataBase::new(DATABASE_FILE)));
    handle_args(&data);

    spawn_season_checker(data.clone());

    let assets_path = if cfg!(debug_assertions) { "assets" } else { "./db/assets" };

    let server = HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(Cors::default().allow_any_header().allow_any_origin().allow_any_method())
            .service(Files::new("/assets", assets_path).show_files_listing())
            .service(create_user)
            .service(edit_users)
            .service(edit_match)
            .service(delete_match)
            .service(get_profile)
            .service(get_users)
            .service(get_all_users)
            .service(register_match)
            .service(get_history)
            .service(get_edit_history)
            .service(get_is_admin)
            .service(login)
            .service(change_password)
            .service(request_reset_password)
            .service(roll_back)
            .service(get_active_users)
            .service(get_season_length)
            .service(set_season_length)
            .service(stop_season)
            .service(start_season)
            .service(cancel_season)
            .service(get_leaderboard_info)
            .service(get_stats)
            .service(get_multiple_users)
            .service(execute_sql)
            .service(get_variable)
            .service(set_variable)
            .service(get_season_start_date)
            .service(get_notifications)
            .service(respond_to_notification)
            .service(create_tournament)
            .service(join_tournament)
            .service(leave_tournament)
            .service(register_tournament_match)
            .service(delete_tournament)
            .service(get_tournament_infos)
            .service(get_tournament)
            .service(get_tournament_table)
            .service(recreate_tournament)
    });

    if cfg!(debug_assertions)
    {
        server.bind(format!("0.0.0.0:{}", PORT))?
    }
    else
    {
        server.bind_openssl(format!("0.0.0.0:{}", PORT), get_builder())?
    }
    .run()
    .await
}
