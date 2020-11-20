mod server;
mod user;
mod r#match;
mod notification;
mod server_rollback;

use server::DataBase;
use r#match::{MatchInfo, MatchResponse};
use user::{LoginInfo, ChangePasswordInfo, EditUsersInfo};
use notification::NewUserNotificationAns;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;

use std::sync::{Mutex, Arc};

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
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::Conflict().body(format!("{}", e))
    }
}


#[post("/edit-users")]
async fn edit_users(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: EditUsersInfo = match serde_json::from_str(&info)
    {
        Ok(info) => info,
        Err(e) => return HttpResponse::BadRequest().body(format!("{}", e)),
    };

    match DATABASE!(data).edit_users(info.users, info.action, info.token)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}


#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: MatchInfo = match serde_json::from_str(&info)
    {
        Ok(info) => info,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    let token = info.token;
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
    let token = info.user_token.clone();

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
        Err(e) =>
        {
            if let rusqlite::Error::InvalidParameterName(s) = e
            {
                // Absolutely disgusting
                if s == "Waiting for admin"
                {
                    return HttpResponse::Unauthorized().finish();
                }
            }
            HttpResponse::BadRequest().finish()
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

#[get("/all-users/{token}")]
async fn get_all_users(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_all_users(token)
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

#[get("/user-notification/{token}")]
async fn get_new_user_notifications(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_new_user_notifications(token)
    {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}

#[post("/respond-to-user-notification")]
async fn respond_to_new_user(data: web::Data<Arc<Mutex<DataBase>>>, info: String) -> HttpResponse
{
    let info: NewUserNotificationAns = serde_json::from_str(&info).unwrap();
    match DATABASE!(data).respond_to_new_user(info)
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

#[get("/is-admin/{token}")]
async fn get_is_admin(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(token): web::Path<String>) -> HttpResponse
{
    match DATABASE!(data).get_is_admin(token.to_string())
    {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
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

    if cfg!(not(debug_assertions))
    {
        server.bind_openssl(format!("sivert.dev:{}", PORT), get_builder())?
    }
    else
    {
        server.bind(format!("0.0.0.0:{}", PORT))?
    }
    .run()
    .await
}
