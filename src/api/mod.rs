use std::sync::Arc;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Result;
use actix_web::web;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::Options;
use crate::session::Error;

mod handlers;
mod sessions;

use sessions::*;

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
            .service(handlers::session_new)
            .service(handlers::session_stop)
            .service(handlers::session_show)
            .service(handlers::sessions_list)
            .service(handlers::plugins_list),
    );
}

pub(crate) async fn start(opts: Options) -> Result<(), Error> {
    let address = opts.api.unwrap();

    if !address.contains(':') {
        return Err("no port specified, please specify a port in the format host:port".to_string());
    }

    log::info!("starting api on http://{} ...", &address);

    if !address.contains("localhost") && !address.contains("127.0.0.1") {
        log::warn!(
            "this server does not provide any authentication and you are binding it to an external address, use with caution!"
        );
    }

    if opts.api_allowed_origin.to_lowercase() == "any" {
        log::warn!(
            "Any CORS origin policy specified, this server will accept requests from any origin"
        );
    }

    let state = Arc::new(RwLock::new(Sessions::new(opts.concurrency)));

    HttpServer::new(move || {
        let cors = match opts.api_allowed_origin.to_lowercase().as_str() {
            "any" => Cors::permissive(),
            _ => Cors::permissive().allowed_origin(&opts.api_allowed_origin),
        };

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .configure(config)
            .default_service(web::route().to(not_found))
        //.wrap(actix_web::middleware::Logger::default())
    })
    .bind(&address)
    .map_err(|e| e.to_string())?
    .run()
    .await
    .map_err(|e| e.to_string())
}
