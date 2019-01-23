use chrono::prelude::{ NaiveDateTime, Utc, };
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use crate::schema::{ ipaddrs, };

#[derive(Debug, Insertable)]
#[table_name = "ipaddrs"]
pub struct NewIp {
    pub ipaddr: String,
}

#[derive(Debug, Identifiable, Queryable, Serialize)]
#[table_name = "ipaddrs"]
pub struct DoneIp {
    pub id: i32,
    pub ipaddr: String,
    pub timefirst: NaiveDateTime,
    pub timelast: NaiveDateTime,
}

pub fn upsertip(strct: NewIp, conn: &PgConnection) -> QueryResult<DoneIp> {
    use crate::schema::ipaddrs::dsl::*;
    diesel::insert_into(ipaddrs)
        .values(&strct)
        .on_conflict(ipaddr)
        .do_update()
        .set(timelast.eq(Utc::now().naive_utc()))
        .get_result::<DoneIp>(conn)
}
