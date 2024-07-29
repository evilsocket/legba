use std::collections::HashMap;

use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use clap::Parser;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::api::SharedState;
use crate::plugins;
use crate::Options;

lazy_static! {
    // nasty hack to check for plugin specific options
    static ref OPTIONS_MAP: HashMap<String, serde_json::Value> = {
        let opts = serde_json::to_string(&Options::parse()).unwrap();
        serde_json::from_str(&opts).unwrap()
    };
}

#[derive(Serialize)]
struct Plugin {
    name: String,
    description: String,
    strategy: String,
    options: Option<serde_json::Value>,
    override_payload: Option<String>,
}

fn get_plugin_options(plugin_name: &str) -> Option<serde_json::Value> {
    // nasty hack to check for plugin specific options
    let opt_name = plugin_name.replace('.', "_");
    let opt_parts: Vec<&str> = plugin_name.splitn(2, '.').collect();
    let opt_root = if opt_parts.len() == 2 {
        opt_parts[0]
    } else {
        &opt_name
    };

    match OPTIONS_MAP.get(&opt_name) {
        None => match OPTIONS_MAP.get(opt_root) {
            None => None,
            Some(v) => Some(v.clone()),
        },
        Some(v) => Some(v.clone()),
    }
}

#[get("/plugins")]
pub async fn plugins_list(_: web::Data<SharedState>) -> HttpResponse {
    let mut list = vec![];

    for (name, plug) in plugins::manager::INVENTORY.lock().unwrap().iter() {
        list.push(Plugin {
            name: name.to_string(),
            description: plug.description().to_string(),
            strategy: plug.payload_strategy().to_string(),
            override_payload: match plug.override_payload() {
                Some(over) => Some(over.as_string()),
                None => None,
            },
            options: get_plugin_options(name),
        })
    }

    HttpResponse::Ok().json(list)
}

#[get("/sessions")]
pub async fn sessions_list(state: web::Data<SharedState>) -> HttpResponse {
    HttpResponse::Ok().json(&*state.read().await)
}

#[get("/session/{session_id}")]
pub async fn session_show(path: web::Path<String>, state: web::Data<SharedState>) -> HttpResponse {
    let session_id = path.into_inner();
    let session_id = match uuid::Uuid::parse_str(&session_id) {
        Ok(uuid) => uuid,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    match state.read().await.get_session(&session_id) {
        Some(session) => HttpResponse::Ok().json(session),
        None => HttpResponse::NotFound().body("not found"),
    }
}

#[get("/session/{session_id}/stop")]
pub async fn session_stop(path: web::Path<String>, state: web::Data<SharedState>) -> HttpResponse {
    let session_id = path.into_inner();
    let session_id = match uuid::Uuid::parse_str(&session_id) {
        Ok(uuid) => uuid,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    match state.read().await.stop_session(&session_id) {
        Ok(_) => HttpResponse::Ok().body("session stopping"),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[post("/session/new")]
pub async fn session_new(
    state: web::Data<SharedState>,
    req: HttpRequest,
    argv: web::Json<Vec<String>>,
) -> HttpResponse {
    let client = req.peer_addr().unwrap();
    match state
        .write()
        .await
        .start_new_session(client.to_string(), argv.0)
        .await
    {
        Ok(session_id) => HttpResponse::Ok().json(session_id),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
