mod server;
mod user;
mod r#match;

use server::DataBase;
use r#match::Match;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;

use std::sync::{Mutex, Arc};
use std::env::args;

const PORT: u32 = 58642;

macro_rules! DATABASE
{
    ($data:expr) =>
    {
        &*$data.get_ref().lock().expect("Could not get  database lock");
    }
}


#[post("/create-user/{name}")]
async fn create_user(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(name): web::Path<String>) -> HttpResponse
{
    let password = "@test".to_string();
    match DATABASE!(data).create_user(name.to_string(), password)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::Conflict().body(format!("{}", e))
    }
}


#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: web::Query<Match>) -> HttpResponse
{
    let res = info.into_inner();

    match DATABASE!(data).register_match(res)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::Conflict().body(format!("{}", e))
    }
}

#[post("respond-to-match")]
async fn respond_to_match(data: web::Data<Arc<Mutex<DataBase>>>, info: web::Query<Match>) -> HttpResponse
{
    let id = 0;
    let answer = true;
    let token: String = "".to_string();

    match DATABASE!(data).response_to_match(id, answer, token)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}

#[post("/login")]
async fn login(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    let name = "Sivert".to_string();
    let passwd = "@new".to_string();

    match DATABASE!(data).login(name, passwd)
    {
        Ok(uuid) => HttpResponse::Ok().json(uuid),
        Err(e) => HttpResponse::BadRequest().body(format!("{}", e))
    }
}

#[post("/change-password")]
async fn change_password(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    let name = "Sivert".to_string();
    let passwd = "@uit".to_string();
    let new_password = "@new".to_string();

    match DATABASE!(data).change_password(name, passwd, new_password)
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
    let data = Arc::new(Mutex::new(DataBase::new()));
    //data.lock().expect("Getting lock").migrate();
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(Cors::default().allow_any_header())
            .service(create_user)
            .service(get_profile)
            .service(get_users)
            .service(register_match)
            .service(respond_to_match)
            .service(get_history)
            .service(login)
            .service(change_password)
    })
    .bind(format!("{}:{}", addr, PORT))?
    .run()
    .await
}
