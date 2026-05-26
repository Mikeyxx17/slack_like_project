mod handler;
mod models;
mod state;

use axum::{Router, routing::get};
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

// ============================================================
// main.rs — 后端服务入口
//
// 启动 Axum HTTP 服务器，监听 0.0.0.0:3000。
// 提供两条通信线路：
//   1. REST:   GET/POST /api/channels  — 频道列表查询和创建
//   2. WS:     GET /ws/{channel}       — 实时聊天消息推送
//
// 启动流程：
//   加载 .env → 连接 PostgreSQL → 运行迁移 → 预热频道 → 注册路由 → 启动服务
// ============================================================

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 从 .env 读取 PostgreSQL 连接字符串
    let database_url = env::var("DATABASE_URL").expect("请在 .env 文件中设置 DATABASE_URL");

    // 创建数据库连接池（最多 5 个并发连接）
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("无法连接到数据库");

    // 自动执行 migrations/ 目录下的 SQL 迁移脚本
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败，请检查 SQL 脚本");

    // 初始化内存中的频道字典（DashMap：并发安全的 HashMap）
    let channels = Arc::new(DashMap::new());
    let state = AppState { db: pool, channels };

    // CORS 配置：开发阶段允许所有来源
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 预热：从数据库加载所有已有频道，在内存中预先创建 broadcast channel
    // 避免首个用户连接时的冷启动延迟
    let saved_channels = sqlx::query!("SELECT name FROM channels")
        .fetch_all(&state.db)
        .await
        .expect("无法从数据库加载频道列表");

    for row in saved_channels {
        // 每个频道一个独立的 broadcast channel（容量 100）
        let (tx, _rx) = broadcast::channel(100);
        state.channels.insert(row.name, tx);
    }

    println!("已成功加载 {} 个频道", state.channels.len());

    // 路由注册
    let app = Router::new()
        .route("/ws/{channel}", get(ws_handler))    // WebSocket 升级
        .route("/api/channels", get(get_channels).post(create_channel))  // REST API
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!(
        "后端引擎已就绪：http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
