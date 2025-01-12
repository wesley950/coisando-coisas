use std::time::Duration;

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, web, HttpResponse,
};
use aws_sdk_s3::{presigning::PresigningConfig, Client};
use coisando_coisas::{
    schema::{attachments, listings, users},
    AccountStatus, Campus, DbConn, DbPool, LocalUser, Type,
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl,
};
use maud::html;
use uuid::Uuid;

use super::{render_base, PaginationQuery};

struct User {
    username: String,
    avatar_url: String,
}

impl User {
    fn new(username: String, avatar_seed: Uuid) -> Self {
        let avatar_url = format!(
            "https://api.dicebear.com/9.x/dylan/svg?seed={}&radius=50&backgroundColor=29e051,619eff,ffa6e6,b6e3f4,c0aede,d1d4f9,ffd5dc,ffdfbf&hair=buns,flatTop,fluffy,longCurls,parting,plain,roundBob,shaggy,shortCurls,spiky,wavy,bangs&mood=happy,hopeful,superHappy",
            avatar_seed
        );
        Self {
            username,
            avatar_url,
        }
    }
}

struct Listing {
    id: Uuid,
    title: String,
    description: String,
    type_: Type,
    campus: Campus,
    images: Vec<String>,
    user: User,
}

fn get_listing_images(listing_id: Uuid, uploader_id: Uuid, conn: &mut DbConn) -> Vec<String> {
    let Ok(results) = attachments::table
        .filter(attachments::listing_id.eq(listing_id))
        .select(attachments::id)
        .load::<Uuid>(conn)
    else {
        return vec![];
    };
    let urls = results
        .iter()
        .map(|id| format!("/attachments/{}/{}", uploader_id, id))
        .collect();

    urls
}

#[get("/")]
async fn render_index(
    pool: web::Data<DbPool>,
    local_user: LocalUser,
    pagination: web::Query<PaginationQuery>,
) -> actix_web::Result<HttpResponse> {
    let offset = pagination.deslocamento.unwrap_or(0);
    let limit = pagination.quantidade.unwrap_or(10);

    // get db connection
    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível obter uma conexão com o banco de dados",
        ));
    };

    // get just the content we need
    let Ok(results) = listings::table
        .inner_join(users::table.on(listings::creator_id.eq(users::id)))
        .limit(limit as i64)
        .offset(offset as i64)
        .order_by(listings::created_at.desc())
        .select((
            listings::id,
            listings::title,
            listings::description,
            listings::type_,
            listings::campus,
            users::id,
            users::nickname,
            users::avatar_seed,
        ))
        .load::<(Uuid, String, String, Type, Campus, Uuid, String, Uuid)>(&mut conn)
    else {
        return Err(ErrorInternalServerError("Não foi possível obter os itens"));
    };

    // convert to a more convenient format
    let listings: Vec<Listing> = results
        .iter()
        .map(
            |(id, title, description, listing_type, campus, creator_id, nickname, avatar_seed)| {
                Listing {
                    id: *id,
                    title: title.clone(),
                    description: description.clone(),
                    type_: *listing_type,
                    campus: *campus,
                    images: get_listing_images(*id, *creator_id, &mut conn),
                    user: User::new(nickname.clone(), *avatar_seed),
                }
            },
        )
        .collect();

    let markup = render_base(
        html! {
            // hero
            h1 .text-center { "Bem-vindo ao Coisando Coisas!" }
            p .lead.text-center { "Onde estudantes compartilham, trocam e salvam o planeta. 😃" }

            // search form
            div .form-floating.mb-3 {
                input type="text" class="form-control" id="search" placeholder="";
                label .text-muted for="search" { i .bi.bi-binoculars-fill {} " Do que você precisa?" }
            }

            // results
            div .row.row-cols-1.row-cols-md-2.row-cols-lg-3.g-4 {
                @for item in listings {
                    div .col {
                        .card.card-body.bg-body-tertiary.border-0.shadow-sm.px-0 {
                            // simple avatar
                            p .px-3 { img src=(item.user.avatar_url) width=(32) height=(32) {} " " (item.user.username) }

                            // carousel
                            div .carousel.slide #(format!("carousel-{}", item.id)) {
                                div .carousel-inner {
                                    @for (i, image) in item.images.iter().enumerate() {
                                        div class={@if i == 0 { "carousel-item active" } @else { "carousel-item" }} {
                                            img src=(image) class="d-block w-100" alt=(item.title);
                                        }
                                    }
                                }
                                @if item.images.len() > 1 {
                                    button .carousel-control-prev role="button" data-bs-target=(format!("#carousel-{}", item.id)) data-bs-slide="prev" {
                                        span .carousel-control-prev-icon aria-hidden="true" {}
                                        span .visually-hidden { "Previous" }
                                    }
                                    button .carousel-control-next role="button" data-bs-target=(format!("#carousel-{}", item.id)) data-bs-slide="next" {
                                        span .carousel-control-next-icon aria-hidden="true" {}
                                        span .visually-hidden { "Next" }
                                    }
                                }
                            }

                            div .vstack.gap-2.px-3 {
                                // details
                                h4 .mt-2.card-title { (item.title) }
                                div .row.g-2 {
                                    div .col {
                                        strong.text-nowrap {
                                            i .fa-solid.fa-map-location {}
                                            " "
                                            (item.campus)
                                        }
                                    }
                                    div .col {
                                        strong.text-nowrap {
                                            @match &item.type_ {
                                                Type::Donation => { i .fa-solid.fa-gift {} }
                                                Type::Loan => { i .fa-solid.fa-hand-holding {} }
                                                Type::Exchange => { i .fa-solid.fa-exchange-alt {} }
                                                Type::Request => { i .fa-solid.fa-hand-paper {} }
                                            }
                                            " "
                                            (item.type_)
                                        }
                                    }
                                }
                                p .d-block.text-truncate.text-wrap.card-text style="height: 3em" { (item.description) }
                                a .text-decoration-none.text-center href=(format!("/item/{}", item.id)) { i .fa-solid.fa-circle-info {} " Detalhes" }
                            }
                        }
                    }
                }
            }

            // pagination maybe?
        },
        local_user,
    );

    Ok(HttpResponse::Ok().body(markup.into_string()))
}

async fn generate_get_presigned_url(
    s3_client: &Client,
    user_id: Uuid,
    attachment_id: Uuid,
) -> actix_web::Result<String> {
    let expires_in = Duration::from_secs(120);
    let Ok(presigning_cfg) = PresigningConfig::expires_in(expires_in) else {
        return Err(ErrorInternalServerError("Não foi possível gerar a URL"));
    };
    let Ok(request) = s3_client
        .get_object()
        .bucket("coisandocoisas")
        .key(format!("{}/{}", user_id, attachment_id))
        .presigned(presigning_cfg)
        .await
    else {
        return Err(ErrorInternalServerError("Não foi possível gerar a URL"));
    };

    Ok(request.uri().to_string())
}

#[get("/attachments/{user_id}/{attachment_id}")]
async fn view_attachment(
    pool: web::Data<DbPool>,
    s3_client: web::Data<Client>,
    path: web::Path<(Uuid, Uuid)>,
) -> actix_web::Result<HttpResponse> {
    let (user_id, attachment_id) = path.into_inner();

    let Ok(mut conn) = pool.get() else {
        return Err(ErrorInternalServerError(
            "Não foi possível obter uma conexão com o banco de dados",
        ));
    };

    // check if target user is valid
    let Ok(result) = users::table
        .filter(
            users::id
                .eq(user_id)
                .and(users::status.eq(AccountStatus::CONFIRMED)),
        )
        .select(users::id)
        .first::<Uuid>(&mut conn)
        .optional()
    else {
        return Err(ErrorInternalServerError(
            "Não foi possível verificar o usuário",
        ));
    };
    if result.is_none() {
        return Err(ErrorBadRequest("Usuário inválido"));
    }

    // TODO: check if post is valid or still up

    // generate a presigned URL
    let url = generate_get_presigned_url(&s3_client, user_id, attachment_id).await?;
    Ok(HttpResponse::Found()
        .append_header(("Location", url))
        .finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(render_index).service(view_attachment);
}
