use std::collections::HashMap;
use actix_web::{
    AsyncResponder, FutureResponse, HttpRequest,
    HttpResponse, Path, Error, Query,
};
use actix_web::middleware::identity::RequestIdentity;
use futures::{ Future, };
use crate::api::{ AppState, };
use crate::show::db::{ TastSixIn, ExplSqlIn, PageIn, };

pub fn tastesix(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    let ip_id = req.identity().unwrap_or("0".to_owned());
    req.state()
        .db
        .send(TastSixIn{
            ipid: ip_id.parse().unwrap_or(0),
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(vec) => {
                let o = serde_json::to_string(&vec)?; 
                //println!("{:?}", o);
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())   
            },
            Err(e) => Err(e),
        })
        .responder()
}

pub fn explsql(
(req, query) : (HttpRequest<AppState>, Query<HashMap<String, String>>)
) -> impl Future<Item = HttpResponse, Error = Error> {

    println!("QUERY POST: {:?}", query);
    let ip_id = req.identity().unwrap_or("0".to_owned());

    let mut categories: Vec<u8> = Vec::new();
    let mut sort: u8 = 0;
    let mut media: Option<u8> = None;
    if query.len() > 0 {
        if let Some(_) = query.get("cat1") {categories.push(0)};
        if let Some(_) = query.get("cat2") {categories.push(1)};
        if let Some(_) = query.get("cat3") {categories.push(2)};
        if let Some(_) = query.get("cat4") {categories.push(3)};
        sort = if let Some(_s) = query.get("sort") { _s.parse().unwrap_or(0) } else { 0 };
        media = match query.get("media") {
            Some(m) => {
                let mslic: &str = &m[..];
                match mslic {
                    "video" => Some(0),
                    "photos" => Some(1),
                    "graphics" => Some(2),
                    "animation" => Some(3),
                    _ => None,
                }
            },
            None => None,
        }
    }
    req.state()
        .db
        .send(ExplSqlIn{
            ipid: ip_id.parse().unwrap_or(0),
            categories: categories,
            sort: sort,
            media: media,
        })
        .from_err()
        .and_then(move |res| match res {
            Ok(vec) => {
                let o = serde_json::to_string(&vec)?; 
                //println!("{:?}", o);
                Ok(HttpResponse::Ok().content_type("application/json").body(o).into())   
            },
            Err(e) => Err(e),
        })
        .responder()
}

#[derive(Deserialize)]
pub struct ShowParams {
    sid: i32,
}

#[derive(Serialize)]
struct PageJ {
    mediahost: String,
    mediaid: String,
    reference: String,
    ends: i32,
}

pub fn show(
(req, param): (HttpRequest<AppState>, Path<ShowParams>)
) -> FutureResponse<HttpResponse> {
    println!("{:?} {}", req, param.sid);
    let ip_id = req.identity().unwrap_or("0".to_owned());
    req.state()
        .db
            .send(PageIn {
                ipid: ip_id.parse().unwrap_or(0),
                id: param.sid,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(_out) => {
                    let host: String = match _out.mediahost {
                        Some(h) => h.to_string(),
                        None => "".to_string(),
                    };
                    let mid: String = match _out.mediaid {
                        Some(i) => i.to_string(),
                        None => "".to_string(),
                    };
                    let fere: String = match _out.reference {
                        Some(r) => r.to_string(),
                        None => "".to_string(),
                    };
                    let es: i32 = match _out.ends {
                        Some(e) => e,
                        None => 0,
                    };
                    let j = PageJ {
                        mediahost: host,
                        mediaid: mid,
                        reference: fere,
                        ends: es,
                    };
                    let o = serde_json::to_string(&j)?;
                    Ok(HttpResponse::Ok().content_type("application/json").body(o).into())
                },
                Err(e) => Err(e),
            })
            .responder()
}
