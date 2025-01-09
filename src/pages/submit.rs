use actix_identity::Identity;
use actix_web::{error::ErrorInternalServerError, get, web, HttpResponse};
use coisando_coisas::{schema::users, DbPool};
use diesel::{
    query_dsl::methods::{FindDsl, SelectDsl},
    OptionalExtension, RunQueryDsl,
};
use maud::html;
use uuid::Uuid;

use crate::pages::render_base;

#[get("/novo")]
async fn render_submit(
    identity: Option<Identity>,
    pool: web::Data<DbPool>,
) -> actix_web::Result<HttpResponse> {
    // get a connection from the pool
    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível conectar ao banco de dados",
        ));
    };

    // check if is logged in
    let Some(identity) = identity else {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/entrar"))
            .finish());
    };
    let Ok(id) = identity.id() else {
        return Err(ErrorInternalServerError(
            "Não foi possível verificar sua sessão. Por favor, entre na sua conta novamente.",
        ));
    };

    // parse the user id
    let Ok(user_id) = Uuid::parse_str(&id) else {
        return Err(ErrorInternalServerError(
            "Não foi possível verificar sua sessão. Por favor, entre na sua conta novamente.",
        ));
    };

    // get user profile
    let Ok(nickname) = users::table
        .find(user_id)
        .select(users::nickname)
        .first::<String>(&mut conn)
        .optional()
    else {
        return Err(ErrorInternalServerError(
            "Não foi possível procurar o perfil do usuário",
        ));
    };

    let markup = render_base(
        html! {
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
        },
        nickname,
    );
    Ok(HttpResponse::Ok().body(markup.into_string()))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(render_submit);
}
