use actix_web::{get, web::{Data, Query, ServiceConfig}, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{db::TagDB, models::{MatchedOrder, UserTags}, orderbook::OrderBook, ORDERS_CACHE};

#[derive(Serialize,Deserialize,Debug)]
struct FetchOrdersQuery{
    from : Option<i64>,
    to : Option<i64>
}


#[get("/orders")]
pub async fn fetch_order(tag_db: Data<TagDB>, query: Query<FetchOrdersQuery>) -> impl Responder {
    let orderbook = match OrderBook::new().await {
        Ok(ob) => ob,
        Err(err) => {
            return HttpResponse::BadGateway().json(err);
        }
    };

    let mut cache = ORDERS_CACHE.lock().unwrap();
    let current_time = Utc::now();

    let orders = if current_time.signed_duration_since(cache.last_fetched) > chrono::Duration::minutes(360) {
        println!("Fetching from Orderbook");
        match orderbook.fetch_orders().await {
            Ok(new_orders) => {
                cache.orders = new_orders.clone();
                cache.last_fetched = current_time;
                println!("Fetched Orders from OrderBook");
                new_orders
            },
            Err(err) => {
                println!("Error Fetching Orders : {:?}", err);
                return HttpResponse::BadGateway().json(err);
            }
        }
    } else {
        println!("Used Cache orders");
        cache.orders.clone()
    };

    let filtered_orders: Vec<MatchedOrder> = orders.into_iter()
        .filter(|order| {
            if order.status!=3 || is_testnet(&order.orderPair){
                return false;
            }
            let order_time: DateTime<Utc> = order.CreatedAt.clone();
            let order_epoch = order_time.timestamp_millis();
            let from = query.from.unwrap_or(i64::MIN);
            let to = query.to.unwrap_or(i64::MAX);
            order_epoch >= from && order_epoch <= to
        })
        .collect();

    let mut user_tags_cache: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut tagged_orders: Vec<MatchedOrder> = Vec::new();

    for mut order in filtered_orders {
        let address = &order.maker;
        if let Some(tags) = user_tags_cache.get(address) {
            order.tags = Some(tags.clone());
        } else {
            match tag_db.get_user_tags(address).await {
                Ok(usertags) => {
                    user_tags_cache.insert(address.clone(), usertags.tags.clone());
                    order.tags = Some(usertags.tags);
                },
                Err(err) => {
                    println!("Error Fetching Tags : {:?}", err);
                }
            };
        }

        tagged_orders.push(order);
    }

    HttpResponse::Ok().json(&tagged_orders)
}




#[derive(Serialize,Deserialize,Debug)]
struct TagQuery{
    address : String,
    tag : String
}
#[derive(Serialize,Deserialize,Debug)]
enum Status{
    OK,
    ERROR
}

#[derive(Serialize,Deserialize,Debug)]
struct AddTagResponse{
    status : Status,
    error : Option<String>,
    result : Option<UserTags>
}

#[get("/tag")]
pub async fn add_tag(tag_db : Data<TagDB>, query : Query<TagQuery>) -> impl Responder {
    let address = &query.address;
    let tag = &query.tag;

    match tag_db.add_tag(address,tag).await{
        Ok(updated) => {
            HttpResponse::Ok().json(AddTagResponse{
                status : Status::OK,
                error : None,
                result : Some(updated) 
            })
        },
        Err(err) => {
            HttpResponse::Ok().json(AddTagResponse{
                status : Status::ERROR,
                error : Some(err.to_string()),
                result : None
            })
        }
    }
}

#[derive(Serialize,Deserialize,Debug)]
struct SearchQuery{
    address : String
}

#[derive(Serialize,Deserialize,Debug)]
struct SearchResponse{
    tags : Vec<String>, 
    orders :Vec<MatchedOrder>
}


#[get("/search")]
pub async fn search(tag_db : Data<TagDB>, query : Query<SearchQuery>)-> impl Responder {
    let address = &query.address;
    let mut cache = ORDERS_CACHE.lock().unwrap();
    let current_time = Utc::now();
    let orderbook = match OrderBook::new().await {
        Ok(ob) => ob,
        Err(err) => {
            return HttpResponse::BadGateway().json(err);
        }
    };

    let usertags = match tag_db.get_user_tags(&address).await{
        Ok(tags) => tags.tags,
        Err(err)=>{
            println!("Error fetching tags ; {:?}",err);
            Vec::new()
        }
    };
    let orders = if current_time.signed_duration_since(cache.last_fetched) > chrono::Duration::minutes(360) {
        println!("Fetching from Orderbook");
        match orderbook.fetch_orders().await {
            Ok(new_orders) => {
                cache.orders = new_orders.clone();
                cache.last_fetched = current_time;
                println!("Fetched Orders from OrderBook");
                new_orders
            },
            Err(err) => {
                println!("Error Fetching Orders : {:?}", err);
                return HttpResponse::BadGateway().json(err);
            }
        }
    } else {
        println!("Used Cache orders");
        cache.orders.clone()
    };

    let user_orders : Vec<MatchedOrder> = orders.into_iter()
    .filter(|order| {
        if order.status!=3 || is_testnet(&order.orderPair){
            return false;
        }
        let order_address = &order.maker;
        order_address.to_owned()==address.to_owned()
    })
    .collect();

    HttpResponse::Ok().json(SearchResponse{
        tags : usertags,
        orders : user_orders
    })
}

#[get("/thorchain")]
pub async fn thorchain_data(db : Data<TagDB>) -> impl Responder {
    let swaps = match db.get_thorchain_swaps().await{
        Ok(res)=> res,
        Err(err) => {
            println!("Error fetching ThorchainSwaps : {:?}",err);
            return HttpResponse::InternalServerError().json("Unknown Error Occured");
        }
    };
    HttpResponse::Ok().json(swaps)
}

#[get("/chainflip")]
pub async fn chainflip_data(db : Data<TagDB>) -> impl Responder {
    let swaps = match db.get_chainflip_swaps().await{
        Ok(res)=> res,
        Err(err) => {
            println!("Error fetching ChainFlipSwaps : {:?}",err);
            return HttpResponse::InternalServerError().json("Unknown Error Occured");
        }
    };
    HttpResponse::Ok().json(swaps)
}


#[get("/btc-prices")]
pub async fn btc_prices(db : Data<TagDB>) -> impl Responder {
    let closing_prices = match db.get_btc_closing_prices().await{
        Ok(res)=> res,
        Err(err) => {
            println!("Error fetching ChainFlipSwaps : {:?}",err);
            return HttpResponse::InternalServerError().json("Unknown Error Occured");
        }
    };
    HttpResponse::Ok().json(closing_prices)
}


fn is_testnet(orderpair: &str) -> bool {
    let words = &vec!["testnet", "sepolia"];
    words.iter().any(|&word| orderpair.contains(word))
}


pub fn init(config: &mut ServiceConfig) {
    config
    .service(fetch_order)
    .service(add_tag)
    .service(search)
    .service(thorchain_data)
    .service(chainflip_data)
    .service(btc_prices);
}