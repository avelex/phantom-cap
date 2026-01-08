use actix_files as fs;
use actix_web::{App, HttpServer, middleware::Logger, web};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};

use url::Url;

mod format;
mod handlers;
mod models;
mod query;
mod schema;
mod templates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .parse::<Url>()
        .expect("Invalid database URL");

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .unwrap();

    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder().build(manager).await.unwrap();

    HttpServer::new(move || {
        // {Real IP} {PATH} {STATUS CODE} {TOOK MS}
        let logger = Logger::new("ip=%{r}a path=%U status=%s took_ms=%D");

        App::new()
            .wrap(logger)
            .app_data(web::Data::new(pool.clone()))
            .service(handlers::home)
            .service(handlers::search_cap)
            .service(handlers::show_cap_info)
            .service(handlers::show_cap_transfers)
            .service(handlers::show_cap_versions)
            .service(handlers::show_package_info)
            .service(fs::Files::new("/static", "static").show_files_listing())
            .default_service(web::route().to(handlers::not_found))
    })
    .bind((host, port))?
    .run()
    .await
}
