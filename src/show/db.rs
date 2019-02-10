use std::ops::Deref;
use actix::prelude::{ Handler, Message, };
use actix_web::{ Error, error, };
use crate::db::{ DbExecutor };
use crate::show::model::{ Show, AllShows, Page, };

#[derive(Serialize)]
pub struct OneShowOut {
    pub makers: Vec<String>,
    pub oneshow: Show,
}

pub struct TastSixIn {
    pub ipid: i32,
}

impl Message for TastSixIn {
    type Result = Result<Vec<OneShowOut>, Error>;
}

impl Handler<TastSixIn> for DbExecutor {
    type Result = Result<Vec<OneShowOut>, Error>;

    fn handle(&mut self, _in: TastSixIn, _: &mut Self::Context) -> Self::Result {
        let sixshows = Show::tasteshows(self.get_conn()?.deref())
            .map_err(|_| error::ErrorInternalServerError("Error reading 6"));
        match sixshows {
            Ok(six) => {    
                let mut v = Vec::new();
                for s in six {
                    let ppl = Show::get_makers(self.get_conn()?.deref(), &s);
                    v.push(OneShowOut{
                        makers: ppl, 
                        oneshow: s,
                    })
                }
                Ok(v)
            },
            Err(e) => Err(e),
        }      
    }
}

pub struct ExplSqlIn {
    pub ipid: i32,
    pub categories: Vec<u8>,
    pub sort: u8,
    pub media: Option<u8>,
}

impl Message for ExplSqlIn {
    type Result = Result<Vec<OneShowOut>, Error>;
}

impl Handler<ExplSqlIn> for DbExecutor {
    type Result = Result<Vec<OneShowOut>, Error>;

    fn handle(&mut self, _in: ExplSqlIn, _: &mut Self::Context) -> Self::Result {
        let pass: AllShows = AllShows{
            categories: _in.categories,
            sort: _in.sort,
            media: _in.media,
        };
        let allshows = Show::retrieveshows(self.get_conn()?.deref(), &pass)
            .map_err(|_| error::ErrorInternalServerError("Error reading sql"));
        match allshows {
            Ok(six) => {    
                let mut v = Vec::new();
                for s in six {
                    let ppl = Show::get_makers(self.get_conn()?.deref(), &s);
                    v.push(OneShowOut{
                        makers: ppl, 
                        oneshow: s,
                    })
                }
                Ok(v)
            },
            Err(e) => Err(e),
        } 
    }
}


pub struct PageIn {
    pub ipid: i32,
    pub id: i32,
}

impl Message for PageIn {
    type Result = Result<Page, Error>;
}

impl Handler<PageIn> for DbExecutor {
    type Result = Result<Page, Error>;

    fn handle(&mut self, _in: PageIn, _: &mut Self::Context) -> Self::Result {
        let theshow = Show::get_show(self.get_conn()?.deref(), &_in.id)
            .map_err(|_| error::ErrorInternalServerError("Error get the show"));
        match theshow {
            Ok(show) => {    
                Ok(Show::get_page(self.get_conn()?.deref(), &show))
            },
            Err(e) => Err(e),
        } 
    }
}
