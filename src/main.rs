mod models;
mod orderbook;
mod routes;
mod db;
use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{get, main, web::Data, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use db::TagDB;
use lazy_static::lazy_static;
use models::MatchedOrder;
use routes::init;

pub struct OrdersCache{
    orders : Vec<MatchedOrder>,
    last_fetched : DateTime<Utc>
}



lazy_static! {
    static ref ORDERS_CACHE: Mutex<OrdersCache> = Mutex::new(OrdersCache {
        orders: Vec::new(),
        last_fetched: Utc::now() - chrono::Duration::minutes(46),
    });
}

#[get("/")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok().json("Welcome to Analytics Backend")
}

#[main]
async fn main() -> Result<(), std::io::Error> {
    
    let tags_db = TagDB::init().await.expect("Error connecting to Postgres");
    let tagsdb_data = Data::new(tags_db.clone());  
    HttpServer::new( move || {
        App::new()
        .app_data(tagsdb_data.clone())
        .wrap(Cors::default()
                .allow_any_origin()
                .allow_any_header() 
                .allow_any_method()
            )
        .service(home)
        .configure(init)
    }).bind(("0.0.0.0",5000))?
    .run()
    .await
}
