mod server;
mod user;
mod r#match;
mod notification;
mod server_rollback;

use server::{DataBase, ServerError}; 
use r#match::{MatchInfo, MatchResponse};
use user::{LoginInfo, ChangePasswordInfo, EditUsersInfo};
use notification::NewUserNotificationAns;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;

use std::sync::{Mutex, Arc};
use serde_json::json;

const PORT: u32 = 58642;
const DATABASE_FILE: &'static str = "db.db";


macro_rules! DATABASE
{
    ($data:expr) =>
    {
        &*$data.get_ref().lock().expect("Could not get  database lock");
    }
}

fn response_code(e: ServerError) -> u8
{
    match e
    {
        ServerError::UserNotExist => 1,
        ServerError::UsernameTaken => 2,
        ServerError::WrongUsernameOrPassword => 3,
        ServerError::PasswordNotMatch => 4,
        ServerError::Unauthorized => 5,
        ServerError::WaitingForAdmin => 6,
        ServerError::InactiveUser => 7,
        ServerError::Critical(_) => 8,
        _ => 69
    }
}

fn response_error(e: ServerError) -> serde_json::Value
{
    json!({"status": response_code(e)})
}

fn response_ok_with(name: &str, item: String) -> serde_json::Value
{
    json!({"status": 0, name: item})
}

fn response_ok() -> serde_json::Value
{
    json!({"status": 0})
}



#[post("/create-user")]
async fn create_user(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: LoginInfo = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).create_user(info.username.clone(), info.password.clone())
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}


#[post("/edit-users")]
async fn edit_users(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: EditUsersInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).edit_users(info.users, info.action, info.token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}


#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchInfo = serde_json::from_str(&info).unwrap();

    match DATABASE!(data).register_match(info.winner, info.loser, info.token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[post("respond-to-match")]
async fn respond_to_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchResponse = serde_json::from_str(&info).unwrap();
    let id = info.match_notification_id;
    let answer = info.ans;
    let token = info.user_token.clone();

    match DATABASE!(data).respond_to_match(id, answer, token)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[post("/login")]
async fn login(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: LoginInfo = serde_json::from_str(&info).unwrap();
    let name = info.username.clone();
    let password = info.password.clone();

    match DATABASE!(data).login(name, password)
    {
        Ok(uuid) => HttpResponse::Ok().json(response_ok_with("token", uuid)),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[post("/change-password")]
async fn change_password(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: ChangePasswordInfo = serde_json::from_str(&info).unwrap();
    let name = info.username.clone();
    let password = info.password.clone();
    let new_password = info.new_password.clone();

    match DATABASE!(data).change_password(name, password, new_password)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_users()
    {
        Ok(data) => HttpResponse::Ok().json(json!({"status": 0, "result": data})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/all-users/{token}")]
async fn get_all_users(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_all_users(token)
    {
        Ok(data) => HttpResponse::Ok().json(json!({"status": 0, "result": data})),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

#[get("/notifications/{token}")]
async fn get_notifications(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_notifications(token)
    {
        Ok(notifications) => HttpResponse::Ok().json(json!({"status": 0, "result": notifications})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/user-notification/{token}")]
async fn get_new_user_notifications(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_new_user_notifications(token)
    {
        Ok(notifications) => HttpResponse::Ok().json(json!({"status": 0, "result": notifications})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[post("/respond-to-user-notification")]
async fn respond_to_new_user(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: NewUserNotificationAns = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).respond_to_new_user(info)
    {
        Ok(_) => HttpResponse::Ok().json(response_ok()),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/history")]
async fn get_history(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_history()
    {
        Ok(data) => HttpResponse::Ok().json(json!({"status": 0, "result": data})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/user/{name}")]
async fn get_profile(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(name): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_profile(name)
    {
        Ok(data) => HttpResponse::Ok().json(json!({"status": 0, "result": data})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

#[get("/is-admin/{token}")]
async fn get_is_admin(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_is_admin(token.to_string())
    {
        Ok(val) => HttpResponse::Ok().json(json!({"status": 0, "result": val})),
        Err(e) => match e
        {
            ServerError::Rusqlite(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::Ok().json(response_error(e))
        }
    }
}

fn get_builder() -> openssl::ssl::SslAcceptorBuilder
{
     let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
     builder
          .set_private_key_file("privkey.pem", SslFiletype::PEM)
          .expect("failed to open/read key.pem");
      builder
          .set_certificate_chain_file("cert.pem")
          .expect("failed to open/read cert.pem");
      builder
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let data = Arc::new(Mutex::new(DataBase::new(DATABASE_FILE)));

    let server = HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(Cors::default()
                  .allow_any_header()
                  .allow_any_origin()
                  .allow_any_method())
            .service(create_user)
            .service(edit_users)
            .service(get_profile)
            .service(get_users)
            .service(get_all_users)
            .service(register_match)
            .service(respond_to_match)
            .service(respond_to_new_user)
            .service(get_history)
            .service(get_notifications)
            .service(get_new_user_notifications)
            .service(get_is_admin)
            .service(login)
            .service(change_password)
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
