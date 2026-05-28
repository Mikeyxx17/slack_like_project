// 频道 REST API 处理器 — GET/POST /api/channels

use crate::models::{Channel, CreateChannelInput};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;

pub async fn get_channels(State(state): State<AppState>) -> Json<Vec<Channel>> {
    let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(channels)
}

pub async fn create_channel(
    State(state): State<AppState>,
    Json(input): Json<CreateChannelInput>,
) -> axum::http::StatusCode {
    let result = sqlx::query!(
        "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
        input.name
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => axum::http::StatusCode::CREATED,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}
