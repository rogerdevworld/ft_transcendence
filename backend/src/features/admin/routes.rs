use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    features::admin::handlers::{
        delete_room, delete_track, delete_user, get_stats, get_track, get_user, list_rooms,
        list_tracks, list_users, revoke_user_sessions, update_track, update_user,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/stats", get(get_stats))
        .route("/admin/users", get(list_users))
        .route("/admin/users/{id}", get(get_user).patch(update_user).delete(delete_user))
        .route(
            "/admin/users/{id}/revoke-sessions",
            post(revoke_user_sessions),
        )
        .route("/admin/tracks", get(list_tracks))
        .route(
            "/admin/tracks/{id}",
            get(get_track).patch(update_track).delete(delete_track),
        )
        .route("/admin/rooms", get(list_rooms))
        .route("/admin/rooms/{id}", delete(delete_room))
}