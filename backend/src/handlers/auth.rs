// 用户认证处理器 — 注册、登录

use crate::models::RegisterInput;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> impl IntoResponse {
    let hashed_password = bcrypt::hash(&input.password, 10).unwrap();

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)",
        input.username,
        input.email,
        hashed_password
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::CONFLICT,
    }
}
