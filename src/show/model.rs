use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::dsl::{ date, now, };
use chrono::prelude::{ NaiveDate, };
use crate::schema::*;

#[derive(Debug, Identifiable, Queryable, Serialize)]
#[table_name="shows"]
pub struct Show {
    pub id: i32,
    pub imgnum: i32,
    pub title: String,
    pub year: i32,
    pub intro: String,
    pub limitdate: Option<NaiveDate>,
    pub popular: i32,
    pub mature: bool,
    pub movin: bool,
    pub still: bool,
    pub graph: bool,
    pub anime: bool,
    pub illeg: bool,
    pub cat1: bool,
    pub cat2: bool,
    pub cat3: bool,
    pub cat4: bool,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Show, foreign_key = "showid")]
#[table_name="smakers"]
pub struct Maker {
    pub id: i32,
    pub name: String,
    pub showid: i32,
}

pub struct AllShows {
    pub categories: Vec<u8>,
    pub sort: u8,
    pub media: Option<u8>,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Show, foreign_key = "showid")]
#[table_name="spages"]
pub struct Page {
    pub id: i32,
    pub showid: i32,
    pub mediahost: Option<String>,
    pub mediaid: Option<String>,
    pub reference: Option<String>,
    pub ends: Option<i32>,
}

impl Show {

    // return 6 shows after analysing ipid history

    pub fn tasteshows(conn: &PgConnection) -> QueryResult<Vec<Show>> {
        use crate::schema::shows::dsl::*;
        shows
            .filter(limitdate.is_null().or(limitdate.ge(date(now).nullable())))
            .order((popular.desc(), id.desc()))
            .limit(6)
            .load::<Show>(conn)
    }

    // return all shows upon query

    pub fn retrieveshows(conn: &PgConnection, ass: &AllShows) -> QueryResult<Vec<Show>> {
        use crate::schema::shows::dsl::*;
        let mut qr = shows.into_boxed();
        println!("{:?}", ass.categories);
        if ass.categories.len() == 0 {
            qr = qr.filter(illeg.eq(false))
        }
        else if ass.categories.len() > 0 && ass.categories.len() < 4 {
            let mut cat = ass.categories.clone();
            match cat[0] {
                0 => qr = qr.filter(cat1),
                1 => qr = qr.filter(cat2),
                2 => qr = qr.filter(cat3),
                3 => qr = qr.filter(cat4),
                _ => println!("impossible"),
            }
            cat.remove(0);
            for c in cat {
                match c {
                    0 => qr = qr.or_filter(cat1),
                    1 => qr = qr.or_filter(cat2),
                    2 => qr = qr.or_filter(cat3),
                    3 => qr = qr.or_filter(cat4),
                    _ => println!("unknown category"),
                }
            }
        }; 
        if let Some(num) = ass.media {
            match num {
                0 => qr = qr.filter(movin),
                1 => qr = qr.filter(still),
                2 => qr = qr.filter(graph),
                3 => qr = qr.filter(anime),
                _ => println!("unknown media"),
            }    
        };
        qr = qr.filter(limitdate.is_null().or(limitdate.ge(date(now).nullable())));
        match ass.sort {
            0 => qr = qr.order(id.desc()),
            1 => qr = qr.order((popular.desc(), id.desc())),
            2 => qr = qr.order((year.desc(), popular.desc())),
            3 => qr = qr.order((year.asc(), popular.desc())),
            _ => println!("unknown sorting"),
        };
        qr.load::<Show>(conn)
    }

    pub fn get_show(conn: &PgConnection, sid: &i32) -> QueryResult<Show> {
        use crate::schema::shows::dsl::*;
        shows
            .filter(id.eq(sid))
            .first::<Show>(conn)
    }

    pub fn get_makers(conn: &PgConnection, show: &Show) -> Vec<String> {
        use crate::schema::smakers::dsl::*;
        return Maker::belonging_to(show)
        .select(name)
        .load::<String>(conn)
        .expect("Error loading makers")
    }

    pub fn get_page(conn: &PgConnection, show: &Show) -> Page {
        return Page::belonging_to(show)
        .first::<Page>(conn)
        .expect("Error getting page")
    }
}
