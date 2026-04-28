mod handler;
mod models;
mod state;

use axum::{
    Router,
    routing::{get, post},
};
use dashmap::DashMap;
use dotenvy::dotenv;
use handler::get_channels;
use handler::{create_channel, ws_handler};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("请在 .env 文件中设置 DATABASE_URL");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("无法连接到数据库");

    // 自动运行数据库迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败，请检查 SQL 脚本");

    // 初始化空的“频道置物架”。
    let channels = Arc::new(DashMap::new());
    let state = AppState { db: pool, channels };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 🚀 “早晨开店进货”：把数据库里记下来的老频道全捞出来，提前在内存里建好大喇叭
    let saved_channels = sqlx::query!("SELECT name FROM channels")
        .fetch_all(&state.db)
        .await
        .expect("无法从数据库加载频道列表");

    for row in saved_channels {
        // 为每一个查出来的频道创建一个专属的对讲机
        let (tx, _rx) = broadcast::channel(100);
        state.channels.insert(row.name, tx);
    }

    println!("✅ 已成功加载 {} 个频道", state.channels.len());

    let app = Router::new()
        .route("/ws/{channel}", get(ws_handler))
        .route("/api/channels", get(get_channels).post(create_channel))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!(
        "🚀 后端引擎已就绪：http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
