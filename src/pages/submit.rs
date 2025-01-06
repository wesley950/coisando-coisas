use actix_web::{get, web, HttpResponse};
use maud::html;

use crate::pages::render_base;

#[get("/novo")]
async fn render_submit() -> HttpResponse {
    let markup = render_base(html! {
        div .vstack gap-3 {
            h2 { "Novo item" }

            div .form-floating.mb-3 {
                input type="text" class="form-control" id="title" placeholder=" ";
                label for="title" { "Título" }
            }

            div .form-floating.mb-3 {
                textarea class="form-control" id="description" placeholder=" " {}
                label for="description" { "Descrição" }
            }

            label for="type" { "Tipo" }
            select .form-select.mb-3 id="type" {
                option { "Doação" }
                option { "Empréstimo" }
                option { "Troca" }
                option { "Pedido" }
            }

            label for="campus" { "Campus" }
            select .form-select.mb-3 id="campus" {
                option { "Darcy Ribeiro" }
                option { "Planaltina" }
                option { "Ceilândia" }
                option { "Gama" }
            }

            label for="images" { "Imagens" }
            input .form-control.mb-3 type="file" id="images" accept="image/*" multiple;

            button type="submit" class="btn btn-primary" { "Enviar" }
        }
    });
    HttpResponse::Ok().body(markup.into_string())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(render_submit);
}
