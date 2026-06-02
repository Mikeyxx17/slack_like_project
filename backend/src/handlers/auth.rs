// 用户认证处理器 — 注册、登录

use crate::auth::AuthUser;
use crate::auth::Claims;
use crate::models::MeResponse;
use crate::models::{AuthResponse, LoginInput, RegisterInput, User};
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
// use jsonwebtoken::errors::ErrorKind::Json;
use jsonwebtoken::{EncodingKey, Header, encode};
// use tower_http::classify::ServerErrorsFailureClass::StatusCode;

//用户注册函数
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

// 用户登录函数
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> impl IntoResponse {
    let user_result = sqlx::query_as!(
        User,
        "SELECT id, email, password_hash, username, display_name, avatar_url, bio, created_at FROM users WHERE email = $1",
        input.email
    )
    .fetch_optional(&state.db)
    .await;
    match user_result {
        Ok(Some(user)) => {
            let password_ok = bcrypt::verify(&input.password, &user.password_hash).unwrap_or(false);

            if password_ok {
                let exp = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize;

                let my_claims = Claims {
                    sub: user.id,
                    email: user.email.clone(),
                    username: user.username.clone(),
                    exp,
                };
                // -------------------------------------------------------------------------------------
                let secret_string = std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "development_fallback_secret_key_look_out".to_string());

                let encoding_key = EncodingKey::from_secret(secret_string.as_bytes());

                let token = match encode(&Header::default(), &my_claims, &encoding_key) {
                    Ok(t) => t,
                    Err(err) => {
                        println!("JWT 签发失败: {}", err);
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                //--------------------------------------------------------------------------------------------

                let response_body = AuthResponse {
                    token,
                    username: user.username,
                    display_name: user.display_name,
                    avatar_url: user.avatar_url,
                };

                // 返回 200 OK 并附带 JSON 数据
                (StatusCode::OK, Json(response_body)).into_response()
            } else {
                (StatusCode::UNAUTHORIZED, "邮箱或密码错误").into_response()
            }
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "邮箱或密码错误").into_response(),
        Err(e) => {
            println!("登录查询数据库失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_current_user(user: AuthUser) -> impl IntoResponse {
    let response_body = MeResponse {
        id: user.user_id,
        username: user.username,
        email: user.email,
    };
    (StatusCode::OK, Json(response_body))
}
