use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};
use aws_config::BehaviorVersion;
use diesel::{r2d2, PgConnection};
use dotenvy::dotenv;
use env_logger::Env;

mod pages;
use pages::{auth, index, info, submit};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    let secret_key = Key::generate();

    let endpoint_url =
        std::env::var("AWS_S3_ENDPOINT_URL").expect("AWS_S3_ENDPOINT_URL must be set");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint_url)
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(s3_client.clone()))
            .wrap(Logger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .configure(index::config)
            .configure(submit::config)
            .configure(auth::config)
            .configure(info::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
