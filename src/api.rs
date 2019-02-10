use std::collections::HashMap;
use actix::prelude::Addr;
use actix_web::middleware::Response;
use actix_web::middleware::identity::RequestIdentity;
use actix_web::{
    fs::NamedFile, AsyncResponder, FutureResponse, HttpRequest,
    HttpResponse, Responder, Result, Query,
};
use futures::{ future, Future, };
use askama::Template;
use crate::db::{ DbExecutor, IpIn, };

pub struct AppState {
    pub db: Addr<DbExecutor>,
}

#[derive(Template)]
#[template(path = "taste.html")]
struct TasteTemplate {
}

pub fn taste(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let mut _id: Option<String> = None;
    let _ip: Option<String> = if let Some(id) = req.identity() {
        println!("FIRST HITTing the Taste page: ipid = {}", id);
        _id = Some(id);
        None
    } else {
        Some(req.connection_info()
           .remote()
           .map_or("0.0.0.0", |s| s.split(":").nth(0).unwrap())
           .to_owned())
    };
    req.state()
        .db 
        .send(IpIn{
            ipaddr: _ip,
            ident: _id,
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(_out) => {
                if let Some(id) = _out.ipid {
                    req.remember(id)
                };
                let s = TasteTemplate {
                }.render().unwrap();
                Ok(HttpResponse::Ok().body(s))
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Template)]
#[template(path = "explore.html")]
struct ExplTemplate {
    cat1: bool,
    cat2: bool,
    cat3: bool,
    cat4: bool,
    sort: u8,
}

pub fn expl(
(req, query) : (HttpRequest<AppState>, Query<HashMap<String, String>>)
) -> FutureResponse<HttpResponse> {
    //println!("QUERY GET: {:?}", query);
    let mut _id: Option<String> = None;
    let _ip: Option<String> = if let Some(id) = req.identity() {
        println!("FIRST HITTing the EXPLORE page: ipid = {}", id);
        _id = Some(id);
        None
    } else {
        Some(req.connection_info()
           .remote()
           .map_or("0.0.0.0", |s| s.split(":").nth(0).unwrap())
           .to_owned())
    };
    struct Temp {
        cat1: bool,
        cat2: bool,
        cat3: bool,
        cat4: bool,
        sort: u8,
    };
    let mut temp: Temp = Temp {
        cat1: true,
        cat2: true,
        cat3: true,
        cat4: true,
        sort: 0, 
    };
    if query.len() > 0 {
        let mut chkd: u8 = 0;
        if let Some(_) = query.get("cat1") { chkd = chkd + 1; } else { temp.cat1 = false };
        if let Some(_) = query.get("cat2") { chkd = chkd + 1; } else { temp.cat2 = false };
        if let Some(_) = query.get("cat3") { chkd = chkd + 1; } else { temp.cat3 = false };
        if let Some(_) = query.get("cat4") { chkd = chkd + 1; } else { temp.cat4 = false };
        if chkd == 0 {
            temp.cat1 = true; temp.cat2 = true; temp.cat3 = true; temp.cat4 = true;
        };
        temp.sort = if let Some(_s) = query.get("sort") { _s.parse().unwrap_or(0) } else { 0 };
    }
    req.state()
        .db 
        .send(IpIn{
            ipaddr: _ip,
            ident: _id,
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(_out) => {
                if let Some(id) = _out.ipid {
                    req.remember(id)
                };
                let s = ExplTemplate {
                    cat1: temp.cat1,
                    cat2: temp.cat2,
                    cat3: temp.cat3,
                    cat4: temp.cat4,
                    sort: temp.sort,
                }.render().unwrap();
                Ok(HttpResponse::Ok().body(s))
            }
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Template)]
#[template(path = "talk.html")]
struct TalkTemplate {
}

pub fn talk(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let mut _id: Option<String> = None;
    let _ip: Option<String> = if let Some(id) = req.identity() {
        println!("FIRST HITTing the CHAT page: ipid = {}", id);
        _id = Some(id);
        None
    } else {
        Some(req.connection_info()
           .remote()
           .map_or("0.0.0.0", |s| s.split(":").nth(0).unwrap())
           .to_owned())
    };
    req.state()
        .db
        .send(IpIn{
            ipaddr: _ip,
            ident: _id,
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(_out) => {
                if let Some(id) = _out.ipid {
                    req.remember(id)
                };
                let s = TalkTemplate{
                }.render().unwrap();
                Ok(HttpResponse::Ok().body(s))
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Template)]
#[template(path = "terms.html")]
struct TermsTemplate {
}

pub fn terms(_req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let s = TermsTemplate {
    }.render().unwrap();
    Box::new(future::ok(HttpResponse::Ok().body(s)))
}

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
}

pub fn contact(_req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let s = ContactTemplate {
    }.render().unwrap();
    Box::new(future::ok(HttpResponse::Ok().body(s)))
}

pub fn bad_request<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("errors/400.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn not_found<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("errors/404.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}

pub fn internal_server_error<S: 'static>(
    req: &HttpRequest<S>,
    resp: HttpResponse,
) -> Result<Response> {
    let new_resp = NamedFile::open("errors/500.html")?
        .set_status_code(resp.status())
        .respond_to(req)?;
    Ok(Response::Done(new_resp))
}
