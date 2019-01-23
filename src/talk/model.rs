use chrono::prelude::{ NaiveDateTime, };
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use crate::schema::{
    chats, csecrets, historychats, cflags,
};
use crate::model::{ DoneIp };

#[derive(Debug, Insertable)]
#[table_name = "chats"]
pub struct NewChat {
    pub ipid: i32,
    pub whosent: String,
    pub attached: Option<String>,
    pub description: String,
}

#[derive(Debug, Insertable)]
#[table_name = "csecrets"]
pub struct NewSecret {
    pub secret: String,
    pub chatid: i32,
}

#[derive(Debug, Identifiable, Queryable, Serialize)]
pub struct Chat {
    pub id: i32,
    pub ipid: i32,
    pub rootnum: i32,
    pub replnum: i32,
    pub timeposted: NaiveDateTime,
    pub whosent: String,
    pub flag: i32,
    pub attached: Option<String>,
    pub description: String,
}

#[derive(Debug, Insertable)]
#[table_name = "historychats"]
pub struct HistoryChat {
    pub chatid: i32,
    pub ipid: i32,
    pub whathappened: String,
    pub rootnum: i32,
    pub replnum: i32,
    pub timeposted: NaiveDateTime,
    pub whosent: String,
    pub flag: i32,
    pub attached: Option<String>,
    pub description: String,
}

pub struct EditChat {
    pub id: i32,
    pub linky: Option<String>,
    pub desc: String,
    pub sameimg: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "cflags"]
pub struct FlagIp {
    pub ipid: i32,
    pub chatid: i32,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Chat, foreign_key = "chatid")]
#[belongs_to(DoneIp, foreign_key = "ipid")]
#[table_name="cflags"]
pub struct CFlag {
    pub id: i32,
    pub ipid: i32,
    pub chatid: i32,
    pub timeflagged: NaiveDateTime,
}

impl Chat {

    pub fn you_flagged_this_chat(conn: &PgConnection, c: &Chat, i: &i32) -> QueryResult<Vec<i32>> {
        use crate::schema::cflags::dsl::*;
        CFlag::belonging_to(c)
            .filter(ipid.eq(i))
            .select(chatid)
            .load::<i32>(conn)
    }

    pub fn readchats(conn: &PgConnection, offset: i64) -> QueryResult<Vec<Chat>> {
        use crate::schema::chats::dsl::*;
        chats
            .order((rootnum.desc(), replnum.asc()))
            .filter(flag.lt(4))
            .limit(5)
            .offset(offset)
            .load::<Chat>(conn)
    }

    pub fn inserttask(nc: NewChat, conn: &PgConnection) -> QueryResult<Chat> {
        diesel::insert_into(chats::table).values(&nc).get_result::<Chat>(conn)
    }

    pub fn get_max_replnum(parentid: i32, conn: &PgConnection) -> QueryResult<i32> {
        use crate::schema::chats::dsl::*;
        chats
            .filter(rootnum.eq(parentid))
            .select(replnum)
            .order(replnum.desc())
            .first::<i32>(conn)
    }

    pub fn set_as_repl(idd: i32, parentid: i32, repl: i32, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        let updated = diesel::update(chats.find(idd));
        updated
            .set((rootnum.eq(parentid), replnum.eq(repl)))
            .execute(conn)
    }

    pub fn set_as_root(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        let updated = diesel::update(chats.find(idd));
        updated
            .set(rootnum.eq(idd))
            .execute(conn)
    }

    pub fn get_chat(idd: i32, conn: &PgConnection) -> QueryResult<Chat> {
        use crate::schema::chats::dsl::*;
        chats
            .filter(id.eq(idd))
            .first::<Chat>(conn)
    }

    pub fn insertsecret(key: NewSecret, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(csecrets::table).values(&key).execute(conn)
    }

    pub fn get_secret(idd: i32, conn: &PgConnection) -> QueryResult<String> {
        use crate::schema::csecrets::dsl::*;
        csecrets
            .filter(chatid.eq(idd))
            .select(secret)
            .first::<String>(conn)
    }

    pub fn delete_with_id(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        diesel::delete(chats.find(idd)).execute(conn)
    }

    pub fn inserthistory(ch: HistoryChat, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::historychats::dsl::*;
        diesel::insert_into(historychats)
            .values(&ch)
            .execute(conn)
    }

    pub fn re_write_desc(t: &EditChat, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        let updated = diesel::update(chats.find(t.id));
        if t.sameimg {
            updated
                .set(description.eq(t.desc.clone()))
                .execute(conn)
        } else {
            updated
                .set((description.eq(t.desc.clone()), attached.eq(t.linky.clone())))
                .execute(conn)
        }
    }

    pub fn add_flagnum(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        diesel::update(chats.find(idd)).set(flag.eq(flag + 1)).execute(conn)
    }

    pub fn sub_flagnum(idd: i32, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::chats::dsl::*;
        diesel::update(chats.find(idd)).set(flag.eq(flag - 1)).execute(conn)
    }

    pub fn who_flagged(fg: FlagIp, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::cflags::dsl::*;
        diesel::insert_into(cflags).values(fg).execute(conn)
    }

    pub fn rm_who_flagged(fg: FlagIp, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::cflags::dsl::*;
        diesel::delete(cflags.filter(ipid.eq(fg.ipid).and(chatid.eq(fg.chatid)))).execute(conn)
    }

    pub fn get_flag(idd: i32, conn: &PgConnection) -> QueryResult<i32> {
        use crate::schema::chats::dsl::*;
        chats
            .filter(id.eq(idd))
            .select(flag)
            .first::<i32>(conn)
    }
}
