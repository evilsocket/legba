use std::sync::Arc;

use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Result;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::session::Error;
use crate::Options;

mod session;
mod state;

use state::*;

#[derive(Serialize)]
struct Response {
    pub message: String,
}

async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Resource not found".to_string(),
    };
    Ok(HttpResponse::NotFound().json(response))
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(session::start)
            .service(session::show)
            .service(session::list),
    );
}

pub(crate) async fn start(opts: Options) -> Result<(), Error> {
    let address = opts.api.unwrap();

    log::info!("starting api on http://{} ...", &address);

    let state = Arc::new(RwLock::new(State::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(config)
            .default_service(web::route().to(not_found))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(&address)
    .map_err(|e| e.to_string())?
    .run()
    .await
    .map_err(|e| e.to_string())
}
