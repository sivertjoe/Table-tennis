#![feature(proc_macro_hygiene, decl_macro)]
mod server;
mod user;
mod r#match;

use server::DataBase;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;

use serde::Deserialize;

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
async fn create_user(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(name): web::Path<String>) -> impl Responder
{
    match DATABASE!(data).create_user(name.to_string())
    {
        Ok(_) => format!("Created user"),
        Err(e) => format!("Error: {}", e)
    }
}


#[derive(Deserialize)]
struct MatchRes 
{
    winner: String,
    loser: String,
}

#[post("/register-match")]
async fn register_match(data: web::Data<Arc<Mutex<DataBase>>>, info: web::Query<MatchRes>)-> impl Responder
{
    let res = info.into_inner();

    match DATABASE!(data).register_match(res.winner, res.loser)
    {
        Ok(_) => format!("Updated elos"),
        Err(e) => format!("Error: {}", e)
    }
}


#[get("/users")]
async fn get_users(data: web::Data<Arc<Mutex<DataBase>>>) -> impl Responder
{
    match DATABASE!(data).get_users()
    {
        Ok(data) => data,
        Err(e) => format!("Something went wrong {}\n", e)
    }
}

#[get("/profile/{name}")]
async fn get_profile(data: web::Data<Arc<Mutex<DataBase>>>, web::Path(name): web::Path<String>) -> impl Responder
{
    match DATABASE!(data).get_profile(name)
    {
        Ok(data) => data,
        Err(e) => format!("Something went wrong {}\n", e)
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
