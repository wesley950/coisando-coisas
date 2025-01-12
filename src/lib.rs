use std::{
    fmt,
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
use schema::{
    sql_types::{ListingCampus, ListingType, UserStatus},
    users,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = ListingCampus)]
pub enum Campus {
    DarcyRibeiro,
    Planaltina,
    Ceilandia,
    Gama,
}

impl fmt::Display for Campus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Campus::DarcyRibeiro => write!(f, "Darcy Ribeiro"),
            Campus::Planaltina => write!(f, "Planaltina"),
            Campus::Ceilandia => write!(f, "Ceilândia"),
            Campus::Gama => write!(f, "Gama"),
        }
    }
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
pub enum Type {
    Donation,
    Loan,
    Exchange,
    Request,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Donation => write!(f, "Doação"),
            Type::Loan => write!(f, "Empréstimo"),
            Type::Exchange => write!(f, "Troca"),
            Type::Request => write!(f, "Pedido"),
        }
    }
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
