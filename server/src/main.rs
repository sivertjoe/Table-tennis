mod server;
mod user;
mod r#match;
mod notification;
mod server_rollback;

use server::DataBase;
use r#match::{MatchInfo, MatchResponse};
use user::{LoginInfo, ChangePasswordInfo};

use actix_web::{get, post, web, App, web::Json, HttpResponse, HttpServer};
use actix_cors::Cors;

use serde_derive::Deserialize;

use std::sync::{Mutex, Arc};
use std::env::args;

const PORT: u32 = 58642;
const DATABASE_FILE: &'static str = "db.db";

macro_rules! DATABASE
{
    ($data:expr) =>
    {
        &*$data.get_ref().lock().expect("Could not get  database lock");
    }
}


#[post("/create-user")]
async fn create_user(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: LoginInfo = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).create_user(info.username.clone(), info.password.clone())
    {
        Ok(uuid) => HttpResponse::Ok().json(uuid),
        Err(e) => HttpResponse::Conflict().body(format!("{}", e))
    }
}


#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchInfo = serde_json::from_str(&info).unwrap();
    let token = if info.token.is_empty() { None } else { Some(info.token) };

    match DATABASE!(data).register_match(info.winner.clone(), info.loser.clone(), token)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::Conflict().body(format!("{}", e))
    }
}

#[post("respond-to-match")]
async fn respond_to_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchResponse = serde_json::from_str(&info).unwrap();
    let id = info.match_notification_id;
    let answer = info.ans;
    let token = info.token.clone();

    match DATABASE!(data).respond_to_match(id, answer, token)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
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
        Ok(uuid) => HttpResponse::Ok().json(uuid),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
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
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_users()
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

#[get("/notifications/{token}")]
async fn get_notifications(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_notifications(token)
    {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}

#[get("/history")]
async fn get_history(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_history()
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::NotFound().finish()
    }
}

#[get("/user/{name}")]
async fn get_profile(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(name): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_profile(name)
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::NotFound().body(format!("{}", e))
    }
}




#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    let args: Vec<String> = args().collect();
    let mut addr = "0.0.0.0";
    if args.len() > 1
    {
        // Assume the IP is passed
         addr = args[0].as_str();
    }
    let data = Arc::new(Mutex::new(DataBase::new(DATABASE_FILE)));
    //data.lock().expect("Getting lock").migrate();
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(Cors::default()
                  .allow_any_header()
                  .allow_any_origin()
                  .allow_any_method())
            .service(create_user)
            .service(get_profile)
            .service(get_users)
            .service(register_match)
            .service(respond_to_match)
            .service(get_history)
            .service(get_notifications)
            .service(login)
            .service(change_password)
    })
    .bind(format!("{}:{}", addr, PORT))?
    .run()
    .await
}
