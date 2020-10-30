#![feature(proc_macro_hygiene, decl_macro)]
mod server;
mod user;
mod r#match;

use server::DataBase;
use r#match::Match;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use actix_web::middleware::Logger;

use std::sync::{Mutex, Arc};

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
    match DATABASE!(data).create_user(name.to_string())
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

#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<DataBase>>>) -> HttpResponse
{
    match DATABASE!(data).get_users()
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

    let data = Arc::new(Mutex::new(DataBase::new()));
    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(create_user)
            .service(get_profile)
            .service(get_users)
            .service(register_match)
    })
    .bind(format!("0.0.0.0:{}", PORT))?
    .run()
    .await
}
