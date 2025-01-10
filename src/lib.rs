use std::future::{ready, Ready};

use actix_identity::Identity;
use actix_web::{web, FromRequest};
use diesel::{
    query_dsl::methods::{FindDsl, SelectDsl},
    r2d2::ConnectionManager,
    PgConnection, RunQueryDsl,
};
use r2d2_postgres::r2d2;
use schema::users::{self};
use uuid::Uuid;

pub mod schema;

pub type DbConn = PgConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub enum LocalUser {
    Anonymous,
    Authenticated { id: Uuid, nickname: String },
}

impl FromRequest for LocalUser {
    type Error = actix_web::Error;
    type Future = Ready<actix_web::Result<LocalUser>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // TODO: add logging so we can see what's going on

        // get the identity from the request
        let Ok(identity) = Identity::from_request(req, payload).into_inner() else {
            println!("No identity");
            return ready(Ok(LocalUser::Anonymous));
        };

        // get the user id from the identity
        let Ok(id) = identity.id() else {
            println!("No id");
            return ready(Ok(LocalUser::Anonymous));
        };

        let Ok(id) = Uuid::parse_str(&id) else {
            println!("Invalid id");
            return ready(Ok(LocalUser::Anonymous));
        };

        // get the pool from the request
        let Some(pool) = req.app_data::<web::Data<DbPool>>() else {
            println!("No pool");
            return ready(Ok(LocalUser::Anonymous));
        };

        // get a connection from the pool
        let Ok(mut conn) = pool.get() else {
            println!("No connection");
            return ready(Ok(LocalUser::Anonymous));
        };

        // get the user from the database
        let Ok((user_id, nickname)) = users::table
            .find(id)
            .select((users::id, users::nickname))
            .first::<(Uuid, String)>(&mut conn)
        else {
            println!("No user");
            return ready(Ok(LocalUser::Anonymous));
        };

        ready(Ok(LocalUser::Authenticated {
            id: user_id,
            nickname,
        }))
    }
}
