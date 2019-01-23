use std::ops::Deref;
use actix::prelude::{ Actor, SyncContext, Handler, Message, };
use actix_web::{ error, Error, };
use diesel::pg::PgConnection;
use diesel::r2d2::{ ConnectionManager, Pool, PoolError, PooledConnection, };
use crate::model;
use crate::model::{ NewIp };
    
type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub struct DbExecutor(pub PgPool);

impl DbExecutor {
    pub fn get_conn(&self) -> Result<PgPooledConnection, Error> {
        self.0.get().map_err(|e| error::ErrorInternalServerError(e))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct IpOut {
    pub ipid: Option<String>,
}

pub struct IpIn {
    pub ipaddr: Option<String>,
    pub ident: Option<String>,
}

impl Message for IpIn {
    type Result = Result<IpOut, Error>;
}

impl Handler<IpIn> for DbExecutor {
    type Result = Result<IpOut, Error>;

    fn handle(&mut self, _in: IpIn, _: &mut Self::Context) -> Self::Result {
        let mut newipid: Option<String> = None;
        let _id: i32 = if let Some(id) = _in.ident {
            id.parse().unwrap_or(0)
        } else {
            match _in.ipaddr {
                Some(addr) => {
                    let di = model::upsertip(NewIp{ipaddr: addr}, self.get_conn()?.deref()).unwrap();
                    println!("FROM UPSERT IP {:?}", di);
                    let tt = di.id.to_owned();
                    newipid = Some(tt.to_string());
                    tt
                },
                None => 0,
            }
        };
        Ok(IpOut{ipid: newipid})
    }
}
