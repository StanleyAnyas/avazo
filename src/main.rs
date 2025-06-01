use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

// use functions::generate_code;
use std::env;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;

mod db;
mod functions;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server().await
}

async fn server() -> std::io::Result<()>{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = MySqlPoolOptions::new()
                                    .max_connections(20)
                                    .connect(&database_url)
                                    .await
                                    .expect("could not connecty to Db");
    let port = 8080;
    println!("Starting server on port {port}");
    let addrs = ("127.0.0.1", port);
    // let frontend_url = std::env::var("FRONTEND_URL").unwrap_or("http://localhost:3000".to_string());
    const NUM: usize = 2;
    HttpServer::new(move || {
        App::new()
        .wrap(
           Cors::permissive()
        )
        .app_data(web::Data::new(pool.clone()))
        .service(handlers::get_food_list)
        .service(handlers::add_food)
        .service(handlers::add_user)
        .service(handlers::delete_food_handler)
        .service(handlers::login_user_handler)
        .service(handlers::verify_code)
        .service(handlers::send_verify_mail)
        .service(handlers::edit_profile_pic)
        .service(handlers::delete_user)
        .service(handlers::get_donations)
        .service(handlers::edit_donation)
        .service(handlers::get_user_active_donations)
        .service(handlers::cancel_reserve)
        .service(handlers::get_user_active_reserve)
        .service(handlers::edit_profile)
        .service(handlers::make_user_reserve)
        .service(handlers::get_reserves)
    })
    .bind(addrs)?
    .workers(NUM)
    .run()
    .await
}