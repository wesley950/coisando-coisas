use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;

mod pages;
use pages::{auth, index, info, submit};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(index::config)
            .configure(submit::config)
            .configure(auth::config)
            .configure(info::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
