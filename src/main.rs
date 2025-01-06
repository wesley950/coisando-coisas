use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer};
use env_logger::Env;
use pages::{index::render_index, submit::render_submit};

mod pages;

#[get("/")]
async fn hello() -> HttpResponse {
    let markup = render_index();
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/novo")]
async fn submit() -> HttpResponse {
    let markup = render_submit();
    HttpResponse::Ok().body(markup.into_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(hello)
            .service(submit)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
