// 频道 REST API 处理器 — GET/POST /api/channels

use crate::auth::AuthUser;
use crate::models::{Channel, CreateChannelInput};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};

pub async fn get_channels(State(state): State<AppState>) -> Json<Vec<Channel>> {
    let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(channels)
}

pub async fn create_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Json(input): Json<CreateChannelInput>,
) -> impl IntoResponse {
    println!(
        "用户 {} 正在尝试创建名为 {} 的新频道...",
        user.username, input.name
    );
    let result = sqlx::query!(
        "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
        input.name
    )
    .execute(&state.db)
    .await;
    match result {
        Ok(_) => {
            // 5. 频道创建成功后，别忘了动态在内存里为这个新频道开通一个广播通道
            let (tx, _rx) = tokio::sync::broadcast::channel(100);
            state.channels.insert(input.name.clone(), tx);

            StatusCode::CREATED.into_response()
        }
        Err(e) => {
            println!("创建频道失败: {}", e);
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}
