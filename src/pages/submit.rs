use std::io::Write;

use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{
    error::{ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
    get, post, web, HttpResponse,
};
use aws_sdk_s3::{primitives::ByteStream, Client};
use coisando_coisas::{
    schema::{
        attachments, listings,
        sql_types::{ListingCampus, ListingType},
    },
    DbPool, LocalUser,
};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{IsNull, ToSql},
    ExpressionMethods, RunQueryDsl,
};
use maud::html;
use uuid::Uuid;

use crate::pages::render_base;

#[get("/novo")]
async fn render_submit(local_user: LocalUser) -> actix_web::Result<HttpResponse> {
    let markup = render_base(
        html! {
            form .vstack action="/novo" method="post" enctype="multipart/form-data" {
                h2 { "Novo item" }

                div .form-floating.mb-3 {
                    input type="text" class="form-control" id="title" name="title" placeholder="";
                    label for="title" { "Título" }
                }

                div .form-floating.mb-3 {
                    textarea class="form-control" id="description" name="description" placeholder="" {}
                    label for="description" { "Descrição" }
                }

                label for="type" { "Tipo" }
                select .form-select.mb-3 id="listing_type" name="listing_type" {
                    option { "Doação" }
                    option { "Empréstimo" }
                    option { "Troca" }
                    option { "Pedido" }
                }

                label for="campus" { "Campus" }
                select .form-select.mb-3 id="campus" name="campus" {
                    option { "Darcy Ribeiro" }
                    option { "Planaltina" }
                    option { "Ceilândia" }
                    option { "Gama" }
                }

                label for="images" { "Imagens" }
                input .form-control.mb-3 type="file" id="images" name="images" accept="image/*" multiple;

                button type="submit" class="btn btn-primary" { "Enviar" }
            }
        },
        local_user,
    );
    Ok(HttpResponse::Ok().body(markup.into_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = ListingCampus)]
enum Campus {
    DarcyRibeiro,
    Planaltina,
    Ceilandia,
    Gama,
}

impl ToSql<ListingCampus, Pg> for Campus {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            Campus::DarcyRibeiro => out.write_all(b"DARCY")?,
            Campus::Planaltina => out.write_all(b"PLANALTINA")?,
            Campus::Ceilandia => out.write_all(b"CEILANDIA")?,
            Campus::Gama => out.write_all(b"GAMA")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ListingCampus, Pg> for Campus {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"DARCY" => Ok(Campus::DarcyRibeiro),
            b"PLANALTINA" => Ok(Campus::Planaltina),
            b"CEILANDIA" => Ok(Campus::Ceilandia),
            b"GAMA" => Ok(Campus::Gama),
            _ => Err("Unknown campus".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = ListingType)]
enum Type {
    Donation,
    Loan,
    Exchange,
    Request,
}

impl ToSql<ListingType, Pg> for Type {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            Type::Donation => out.write_all(b"DONATION")?,
            Type::Loan => out.write_all(b"LOAN")?,
            Type::Exchange => out.write_all(b"EXCHANGE")?,
            Type::Request => out.write_all(b"REQUEST")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ListingType, Pg> for Type {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"DONATION" => Ok(Type::Donation),
            b"LOAN" => Ok(Type::Loan),
            b"EXCHANGE" => Ok(Type::Exchange),
            b"REQUEST" => Ok(Type::Request),
            _ => Err("Unknown listing type".into()),
        }
    }
}

#[derive(MultipartForm)]
struct ItemForm {
    title: Text<String>,
    description: Text<String>,
    listing_type: Text<String>,
    campus: Text<String>,
    #[multipart(limit = "10MB")]
    images: Vec<TempFile>,
}

#[post("/novo")]
async fn submit_item(
    pool: web::Data<DbPool>,
    s3_client: web::Data<Client>,
    local_user: LocalUser,
    MultipartForm(form): MultipartForm<ItemForm>,
) -> actix_web::Result<HttpResponse> {
    match local_user {
        LocalUser::Anonymous => return Err(ErrorUnauthorized("Usuário não autenticado")),
        LocalUser::Pending => return Err(ErrorForbidden("Usuário não confirmado")),
        LocalUser::Authenticated { id: creator_id, .. } => {
            let title = form.title.into_inner();
            let description = form.description.into_inner();
            let images = form.images;

            if images.len() == 0 {
                return Ok(HttpResponse::Found()
                    .append_header(("Location", "/novo?erro=sem-imagem"))
                    .finish());
            }

            // validate type and campus
            let listing_type = match form.listing_type.as_str() {
                "Doação" => Type::Donation,
                "Empréstimo" => Type::Loan,
                "Troca" => Type::Exchange,
                "Pedido" => Type::Request,
                _ => return Err(ErrorBadRequest("Tipo de anúncio inválido")),
            };
            let campus = match form.campus.as_str() {
                "Darcy Ribeiro" => Campus::DarcyRibeiro,
                "Planaltina" => Campus::Planaltina,
                "Ceilândia" => Campus::Ceilandia,
                "Gama" => Campus::Gama,
                _ => return Err(ErrorBadRequest("Campus inválido")),
            };

            // get a connection from the pool
            let Ok(mut conn) = pool.get() else {
                return Err(ErrorInternalServerError(
                    "Não foi possível conectar ao banco de dados",
                ));
            };

            // insert into database
            let listing_id = match diesel::insert_into(listings::table)
                .values((
                    listings::id.eq(Uuid::new_v4()),
                    listings::title.eq(title),
                    listings::description.eq(description),
                    listings::type_.eq(listing_type),
                    listings::campus.eq(campus),
                    listings::creator_id.eq(creator_id),
                ))
                .returning(listings::id)
                .get_result::<Uuid>(&mut conn)
            {
                Ok(id) => id,
                Err(e) => return Err(ErrorInternalServerError(e)),
            };

            // upload images to cloudflare r2
            for image in images {
                let img_id = Uuid::new_v4();
                let path = format!("/tmp/{}", img_id);

                // persist image file
                if let Err(e) = image.file.persist(&path) {
                    // log error
                    log::error!(
                        "Não foi possível salvar o anexo {} no item {}: {:?}",
                        img_id,
                        listing_id,
                        e
                    );
                    continue;
                }

                // create ByteStream from path
                let stream = match ByteStream::from_path(&path).await {
                    Ok(stream) => stream,
                    Err(e) => {
                        // log error
                        log::error!(
                            "Não foi possível criar ByteStream para o anexo {} no item {}: {:?}",
                            img_id,
                            listing_id,
                            e
                        );
                        continue;
                    }
                };

                // upload image using s3 sdk
                if let Err(e) = s3_client
                    .put_object()
                    .bucket("coisandocoisas")
                    .key(&format!("{}/{}", creator_id, img_id))
                    .body(stream)
                    .send()
                    .await
                {
                    // log error
                    log::error!(
                        "Não foi possível enviar o anexo {} do item {}: {:?}",
                        img_id,
                        listing_id,
                        e
                    );
                    continue;
                }

                // insert attachment into database
                if let Err(e) = diesel::insert_into(attachments::table)
                    .values((
                        attachments::id.eq(img_id),
                        attachments::listing_id.eq(listing_id),
                    ))
                    .execute(&mut conn)
                {
                    // log error
                    log::error!(
                        "Não foi possível inserir informações do anexo {}, item {} no banco de dados: {:?}",
                        img_id, listing_id,
                        e
                    );
                    continue;
                }
            }

            Ok(HttpResponse::SeeOther()
                .append_header(("Location", format!("/item/{}", listing_id)))
                .finish())
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(render_submit).service(submit_item);
}
