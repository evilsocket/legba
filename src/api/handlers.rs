use std::collections::HashMap;
use std::sync::LazyLock;

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use clap::CommandFactory;
use clap::Parser;
use serde::Serialize;

use crate::Options;
use crate::api::SharedState;
use crate::plugins;

// nasty hack to check for plugin specific options
static OPTIONS_MAP: LazyLock<HashMap<String, serde_json::Value>> = LazyLock::new(|| {
    let opts = serde_json::to_string(&Options::parse()).unwrap();
    serde_json::from_str(&opts).unwrap()
});

#[derive(Serialize)]
struct PluginOption {
    name: String,
    description: String,
    value: serde_json::Value,
}

#[derive(Serialize)]
struct Plugin {
    name: String,
    description: String,
    strategy: String,
    options: HashMap<String, PluginOption>,
    override_payload: Option<String>,
}

fn get_plugin_option_help(opt_name: &str) -> String {
    let cmd = Options::command();
    let args = cmd.get_arguments();

    for arg in args {
        if opt_name == arg.get_id() {
            return if let Some(help) = arg.get_help() {
                help.ansi().to_string()
            } else {
                "".to_string()
            };
        }
    }

    "".to_string()
}

fn get_plugin_options(plugin_name: &str) -> HashMap<String, PluginOption> {
    let mut options: HashMap<String, PluginOption> = HashMap::new();

    // nasty hack to check for plugin specific options
    let opt_name = plugin_name.replace('.', "_");
    let opt_parts: Vec<&str> = plugin_name.splitn(2, '.').collect();
    let opt_root = if opt_parts.len() == 2 {
        opt_parts[0]
    } else {
        &opt_name
    };

    let opts = match OPTIONS_MAP.get(&opt_name) {
        None => OPTIONS_MAP.get(opt_root).cloned(),
        Some(v) => Some(v.clone()),
    };

    if let Some(serde_json::Value::Object(opts)) = opts {
        for (opt_name, opt_val) in opts.iter() {
            options.insert(
                opt_name.to_owned(),
                PluginOption {
                    name: opt_name.to_owned(),
                    description: get_plugin_option_help(opt_name),
                    value: opt_val.clone(),
                },
            );
        }
    }

    options
}

#[get("/plugins")]
pub async fn plugins_list(_: web::Data<SharedState>) -> HttpResponse {
    let mut list = vec![];
    let mut consumed = vec![];

    for (name, plug) in plugins::manager::INVENTORY.lock().unwrap().iter() {
        let options = get_plugin_options(name);
        for key in options.keys() {
            consumed.push(key.to_string());
        }

        list.push(Plugin {
            name: name.to_string(),
            description: plug.description().to_string(),
            strategy: plug.payload_strategy().to_string(),
            override_payload: plug.override_payload().map(|s| s.as_string()),
            options,
        })
    }

    let mut not_consumed = HashMap::new();

    for (name, value) in OPTIONS_MAP.iter() {
        if !consumed.contains(name) && !value.is_object() {
            not_consumed.insert(
                name.to_string(),
                PluginOption {
                    name: name.to_string(),
                    description: get_plugin_option_help(name),
                    value: value.clone(),
                },
            );
        }
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
