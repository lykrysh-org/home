use actix_web::{
    AsyncResponder, FutureResponse, HttpRequest,
    HttpResponse, Path, HttpMessage, Json, Error, 
};
use actix_web::error::ErrorInternalServerError;
use actix_web::middleware::identity::RequestIdentity;
use futures::{ future, Future, Stream };
use crate::session::{ self, ChatCnt, UpLoaded, };
use crate::api::{ AppState };
use crate::talk::db::{
    OneChatOut, ReadChats, CreateChat, ToggleChat, DeleteChat, FlagChat,
};
use crate::talk::model::{ EditChat, };
use crate::talk::mltpart::*;

pub fn loadfirst(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    let ip_id = req.identity().unwrap_or("0".to_owned());
    let _ = session::clear_ccnt(&req);
    let _ = session::set_ccnt(&req, ChatCnt::init());
    req.state()
        .db
        .send(ReadChats{
            offset: 0,
            ipid: ip_id.parse().unwrap_or(0),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(vec) => {
                let cntadd: i64 = vec.len() as i64;
                match cntadd {
                    0 => {
                        let vec: Vec<OneChatOut> = Vec::new();
                        let o = serde_json::to_string(&vec)?;
                        Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                    },
                    _ => {
                        println!("FIRST UPLOADS {}", cntadd);
                        session::set_ccnt(&req, ChatCnt::change(cntadd))?;
                        if let Some(c) = session::get_ccnt(&req)? {
                            println!("FINAL CHAT COUNTER =  {}", c.i);
                        };
                        let o = serde_json::to_string(&vec)?;
                        Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                    },
                }
            },
            Err(e) => Err(e),
        })
        .responder()
}

pub fn loadmore(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    let ip_id = req.identity().unwrap_or("0".to_owned());
    let mut skip: i64 = 0;
    if let Some(c) = session::get_ccnt(&req).unwrap() {
        skip = c.i;
        println!("GOT HERE! {}", skip)
    };
    req.state()
        .db
        .send(ReadChats{
            offset: skip,
            ipid: ip_id.parse().unwrap_or(0),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(vec) => {
                let cntadd: i64 = vec.len() as i64;
                match cntadd {
                    0 => {
                        let vec: Vec<OneChatOut> = Vec::new();
                        let o = serde_json::to_string(&vec)?;
                        Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                    },
                    _ => {
                        println!("NEW UPLOADS {}", cntadd);
                        if let Some(c) = session::get_ccnt(&req).unwrap() {
                            let counter: i64 = c.i + cntadd;
                            session::set_ccnt(&req, ChatCnt::change(counter))?
                        } else {
                            session::set_ccnt(&req, ChatCnt::change(cntadd))?;
                        };
                        if let Some(c) = session::get_ccnt(&req)? {
                            println!("FINAL CHAT COUNTER =  {}", c.i);
                        };
                        let o = serde_json::to_string(&vec)?;
                        Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                    },
                }
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Debug, Deserialize)]
pub struct CreateJ {
    inheritedid: String,
    hasimg: String,
    secret: String,
    whosent: String,
    linky: String,
    description: String,
}

#[derive(Serialize)]
struct TempJ {
    id: String,
    rootnum: String,
    replnum: String,
    posted: String,
    attached: String,   
}

pub fn create(
    (req, j): (HttpRequest<AppState>, Json<CreateJ>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    //println!("{:?} {:?}", req, j );
    let conn_ip = req.connection_info()
       .remote()
       .map_or("0.0.0.0", |s| s.split(":").nth(0).unwrap())
       .to_owned();
    let lnk: Option<String> = match j.hasimg.parse().unwrap_or(0) {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up           
             },
        2 => Some(j.linky.clone()),
        _ => None,
    };
    let replyid: Option<i32> = match j.inheritedid.clone().as_ref() {
        "none" => None,
        _whatever => {
            let i: i32 = _whatever.parse().unwrap_or(0);
            Some(i)
        },
    };
    let name = j.whosent.clone();
    req.state()
        .db
        .send(CreateChat {
            inheritedid: replyid,
            secret: j.secret.clone(),
            whosent: name.to_string(),
            ip: conn_ip,
            linky: lnk,
            description: j.description.clone().trim().to_string(),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(t) => {
                if let Some(c) = session::get_ccnt(&req).unwrap() {
                    let counter: i64 = c.i + 1;
                    session::set_ccnt(&req, ChatCnt::change(counter))?;
                } else {
                    session::set_ccnt(&req, ChatCnt::change(1 as i64))?;
                };
                if let Some(c) = session::get_ccnt(&req)? {
                    println!("ADDED CHAT COUNTER =  {}", c.i);
                };
                let att: String = match t.attached {
                    Some(link) => link.to_string(),
                    None => "none".to_string(),
                };
                let out = TempJ {
                    id: t.id.to_owned().to_string(),
                    rootnum: t.rootnum.to_owned().to_string(),
                    replnum: t.replnum.to_owned().to_string(),
                    posted: t.timeposted.to_owned().to_string(),
                    attached: att,
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Err(e) => Err(e),
        })
        .responder()
}

pub fn multipart(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    //println!("{:?}", req);
    Box::new(
        req.multipart()
            .map_err(ErrorInternalServerError)
            .map(handle_multipart_item)
            .flatten()
            .collect()
            .map(move |name| {
                println!("{}", &name[0]);
                let _ = session::set_uploaded(
                    &req,
                    UpLoaded::add(&name[0]),
                );
                HttpResponse::Ok().finish()
            })
            .map_err(|e| {
                println!("failed multipart: {}", e);
                e
            }),
    )
}

#[derive(Debug, Deserialize)]
pub struct PassdJ {
    taskid: String,
    method: String,
    passwd: String,
}

pub fn passd(
    (req, j) : (HttpRequest<AppState>, Json<PassdJ>),
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    //println!("{:?} {:?}", req, j );
    let id = j.taskid.parse::<i32>().unwrap();
    match j.method.as_ref() {
        "put" => Box::new(toggle(req, &id, &j.passwd)),
        "delete" => Box::new(delete(req, &id, &j.passwd)),
        _ => Box::new(future::ok(HttpResponse::Ok().finish())),
    }
}


#[derive(Serialize)]
struct OutJ {
    state: String,
}

fn toggle(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> impl Future<Item = HttpResponse, Error = Error> {
    req.state()
        .db
        .send(ToggleChat { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                let out = OutJ {
                    state: "wrong".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Ok(_taskid) => {
                let out = OutJ {
                    state: "correct".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Err(e) => Err(e),
        })
        .responder()
}

fn delete(
    req: HttpRequest<AppState>,
    id: &i32,
    mypw: &String,
) -> impl Future<Item = HttpResponse, Error = Error> {
    req.state()
        .db
        .send(DeleteChat { id: *id, pw: mypw.to_string() })
        .from_err()
        .and_then(move |res| match res {
            Ok(0) => {
                let out = OutJ {
                    state: "wrong".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Ok(_) => {
                if let Some(c) = session::get_ccnt(&req).unwrap() {
                    let counter: i64 = if c.i > 0 {
                        c.i - 1
                    } else {
                        0
                    };
                    session::set_ccnt(&req, ChatCnt::change(counter as i64))?;
                } else {
                    session::set_ccnt(&req, ChatCnt::change(0 as i64))?;
                };
                if let Some(c) = session::get_ccnt(&req)? {
                    println!("DELETED CHAT COUNTER =  {}", c.i);
                };
                let out = OutJ {
                    state: "deleted".to_owned(),
                };
                let o = serde_json::to_string(&out)?;
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct UpdateParams {
    id: i32,
}

#[derive(Debug, Deserialize)]
pub struct EditJ {
    hasimg: String,
    linky: String,
    description: String,
}

pub fn edit(
    (req, params, j): (HttpRequest<AppState>, Path<UpdateParams>, Json<EditJ>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    //println!("{:?} {:?}", req, j );
    let hinum = j.hasimg.parse().unwrap_or(0);
    let lnk: Option<String> = match hinum {
        1 => {
                let up = match session::get_uploaded(&req).unwrap() {
                    Some(up) => Some(up.uploaded),
                    None => None
                };
                session::clear_uploaded(&req);
                up
             },
        2 => Some(j.linky.clone()),
        _ => None,
    };
    if hinum == 4 {
        req.state()
            .db
            .send(EditChat {
                id: params.id,
                linky: lnk,
                desc: j.description.trim().to_string(),
                sameimg: true,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    let out = OutJ {
                        state: "same".to_owned(),
                    };
                    let o = serde_json::to_string(&out)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
            })
            .responder()
    } else {
        req.state()
            .db
            .send(EditChat {
                id: params.id,
                linky: lnk.clone(),
                desc: j.description.trim().to_string(),
                sameimg: false,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    let s: String = match lnk {
                        Some(word) => word,
                        None => "none".to_string(),
                    };
                    let out = OutJ {
                        state: s.to_owned(),
                    };
                    let o = serde_json::to_string(&out)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
            })
            .responder()
    }
}

#[derive(Debug, Deserialize)]
pub struct FlagJ {
    dir: String,
}

pub fn flag(
    (req, params, j): (HttpRequest<AppState>, Path<UpdateParams>, Json<FlagJ>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    //println!("{:?}", req);
    let conn_ip = req.connection_info()
       .remote()
       .map_or("0.0.0.0", |s| s.split(":").nth(0).unwrap())
       .to_owned();
    req.state()
            .db
            .send(FlagChat {
                ip: conn_ip,
                chatid: params.id,
                dir: j.dir.to_string(),
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(num) => {
                    let out = OutJ {
                        state: num.to_owned().to_string(),
                    };
                    let o = serde_json::to_string(&out)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
        })
        .responder()
}
