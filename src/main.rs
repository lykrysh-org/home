#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::env;
use dotenv::dotenv;
use actix::prelude::SyncArbiter;
use actix_web::middleware::session::{ CookieSessionBackend, SessionStorage, };
use actix_web::middleware::identity::{ CookieIdentityPolicy, IdentityService, };
use actix_web::middleware::{ ErrorHandlers, Logger, };
use actix_web::{ fs, http, server, App, };

mod session;
mod api;
mod db;
mod model;
mod schema;
mod show;
mod talk;

static SESSION_KEY: &[u8] = &[0; 32];
static IDENTITY_KEY: &[u8] = &[0; 32];

fn main() {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "all=debug,actix_web=info");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let system = actix::System::new("all");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&database_url).expect("Failed to create pool");
    let addr = SyncArbiter::start(num_cpus::get(), move || db::DbExecutor(pool.clone()));

    let app = move || {
        debug!("Constructing the App");

        let session_store = SessionStorage::new(
            CookieSessionBackend::signed(SESSION_KEY).secure(false),
        );

        let cook_identity = IdentityService::new(
            CookieIdentityPolicy::new(IDENTITY_KEY).name("lykrysh").secure(false),
        );

        let error_handlers = ErrorHandlers::new()
            .handler(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                api::internal_server_error,
            )
            .handler(http::StatusCode::BAD_REQUEST, api::bad_request)
            .handler(http::StatusCode::NOT_FOUND, api::not_found);

        let state = api::AppState {
            db: addr.clone(),
        };

        let err_files = fs::StaticFiles::new("errors/").expect("failed constructing err files handler");
        let static_files = fs::StaticFiles::new("frontend/").expect("failed constructing static files handler");
        let stuff = fs::StaticFiles::new("stuff/").expect("failed constructing stuff handler");
        let bin = fs::StaticFiles::new("bin/").expect("failed constructing bin handler");

        App::with_state(state)
            .middleware(Logger::default())
            .middleware(session_store)
            .middleware(cook_identity)
            .middleware(error_handlers)
            .route("/", http::Method::GET, api::expl)
            .route("/limited", http::Method::GET, api::taste)
            .resource("/z/{sid}", |r| { r.method(http::Method::POST).with_async(show::api::show) })
            .resource("/s", |r| { r.method(http::Method::POST).with_async(show::api::tastesix) })
            .resource("/a", |r| { r.method(http::Method::POST).with_async(show::api::explsql) })
            .route("/talk", http::Method::GET, api::talk)
            .route("/p", http::Method::POST, talk::api::multipart)
            .resource("/n", |r| { r.method(http::Method::POST).with_async(talk::api::create) })
            .resource("/l1", |r| { r.method(http::Method::POST).with_async(talk::api::loadfirst) })
            .resource("/l2", |r| { r.method(http::Method::POST).with_async(talk::api::loadmore) })
            .resource("/x/{id}", |r| { r.method(http::Method::POST).with_async(talk::api::passd) })
            .resource("/x/{id}/e", |r| { r.method(http::Method::POST).with_async(talk::api::edit) })
            .resource("/x/{id}/f", |r| { r.method(http::Method::POST).with_async(talk::api::flag) })
            .route("/about", http::Method::GET, api::terms)
            .route("/contact", http::Method::GET, api::contact)
            .handler("/errors", err_files)
            .handler("/static", static_files)
            .handler("/stuff", stuff)
            .handler("/bin", bin)
    };

    debug!("Starting server");
    server::new(app).bind("localhost:8088").unwrap().start();
    let _ = system.run();
}
