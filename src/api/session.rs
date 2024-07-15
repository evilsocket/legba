use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;

use crate::api::SharedState;

#[get("/sessions")]
pub async fn list(state: web::Data<SharedState>) -> HttpResponse {
    HttpResponse::Ok().json(state.read().await.active_sessions())
}

#[get("/session/{session_id}")]
pub async fn show(path: web::Path<String>, state: web::Data<SharedState>) -> HttpResponse {
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
pub async fn stop(path: web::Path<String>, state: web::Data<SharedState>) -> HttpResponse {
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
pub async fn start(
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
