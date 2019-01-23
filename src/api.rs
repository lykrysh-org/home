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
    horror: bool,
    wet: bool,
    poetic: bool,
    weird: bool,
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
        horror: bool,
        wet: bool,
        poetic: bool,
        weird: bool,
        sort: u8,
    };
    let mut temp: Temp = Temp {
        horror: true,
        wet: true,
        poetic: true,
        weird: true,
        sort: 0, 
    };
    if query.len() > 0 {
        let mut chkd: u8 = 0;
        if let Some(_) = query.get("horror") { chkd = chkd + 1; } else { temp.horror = false };
        if let Some(_) = query.get("wet") { chkd = chkd + 1; } else { temp.wet = false };
        if let Some(_) = query.get("poetic") { chkd = chkd + 1; } else { temp.poetic = false };
        if let Some(_) = query.get("weird") { chkd = chkd + 1; } else { temp.weird = false };
        if chkd == 0 {
            temp.horror = true; temp.wet = true; temp.poetic = true; temp.weird = true;
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
                    horror: temp.horror,
                    wet: temp.wet,
                    poetic: temp.poetic,
                    weird: temp.weird,
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
