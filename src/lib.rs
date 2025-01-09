use diesel::{r2d2::ConnectionManager, PgConnection};
use r2d2_postgres::r2d2;

pub mod schema;

pub type DbConn = PgConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
