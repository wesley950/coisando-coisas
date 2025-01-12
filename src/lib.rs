use std::{
    future::{ready, Ready},
    io::Write,
};

use actix_identity::Identity;
use actix_web::{web, FromRequest};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    query_dsl::methods::{FindDsl, SelectDsl},
    r2d2::ConnectionManager,
    serialize::{IsNull, ToSql},
    PgConnection, RunQueryDsl,
};
use r2d2_postgres::r2d2;
use schema::{sql_types::UserStatus, users};
use uuid::Uuid;

pub mod schema;

pub type DbConn = PgConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

// map db enum to rust enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = UserStatus)]
pub enum AccountStatus {
    PENDING,
    CONFIRMED,
    DISABLED,
}

impl ToSql<UserStatus, Pg> for AccountStatus {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match *self {
            AccountStatus::PENDING => out.write_all(b"PENDING")?,
            AccountStatus::CONFIRMED => out.write_all(b"CONFIRMED")?,
            AccountStatus::DISABLED => out.write_all(b"DISABLED")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<UserStatus, Pg> for AccountStatus {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"PENDING" => Ok(AccountStatus::PENDING),
            b"CONFIRMED" => Ok(AccountStatus::CONFIRMED),
            b"DISABLED" => Ok(AccountStatus::DISABLED),
            _ => Err("Unknown user status".into()),
        }
    }
}
pub enum LocalUser {
    Anonymous,
    Pending,
    Authenticated {
        id: Uuid,
        nickname: String,
        avatar_seed: Uuid,
    },
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
        let Ok((user_id, nickname, avatar_seed, status)) = users::table
            .find(id)
            .select((
                users::id,
                users::nickname,
                users::avatar_seed,
                users::status,
            ))
            .first::<(Uuid, String, Uuid, AccountStatus)>(&mut conn)
        else {
            println!("No user");
            return ready(Ok(LocalUser::Anonymous));
        };

        if let AccountStatus::PENDING = status {
            return ready(Ok(LocalUser::Pending));
        }

        ready(Ok(LocalUser::Authenticated {
            id: user_id,
            nickname,
            avatar_seed,
        }))
    }
}
