use actix_web::{get, web, HttpResponse};
use maud::html;

use super::render_base;

#[get("/sobre")]
async fn about_page() -> HttpResponse {
    let markup = render_base(
        html! {
            h1 { "Sobre" }
            p { "Página sobre o projeto Coisando Coisas." }
        },
        None,
    );
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/política-de-privacidade")]
async fn privacy_page() -> HttpResponse {
    let markup = render_base(
        html! {
            h1 { "Política de Privacidade" }
            p { "Página sobre a política de privacidade do projeto Coisando Coisas." }
        },
        None,
    );
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/termos-de-uso")]
async fn terms_page() -> HttpResponse {
    let markup = render_base(
        html! {
            h1 { "Termos de Uso" }
            p { "Página sobre os termos de uso do projeto Coisando Coisas." }
        },
        None,
    );
    HttpResponse::Ok().body(markup.into_string())
}

#[get("/diretrizes-da-comunidade")]
async fn community_guidelines_page() -> HttpResponse {
    let markup = render_base(
        html! {
            h1 { "Diretrizes da Comunidade" }
            p { "Página sobre as diretrizes da comunidade do projeto Coisando Coisas." }
        },
        None,
    );
    HttpResponse::Ok().body(markup.into_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(about_page)
        .service(privacy_page)
        .service(terms_page)
        .service(community_guidelines_page);
}
