mod handler;
mod models;
mod state;

use axum::{Router, routing::get};
use dashmap::DashMap;
use dotenvy::dotenv;
use handler::ws_handler;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::sync::Arc; // 👈 修正 1：必须导入 Arc 才能使用
use tokio::sync::broadcast;

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

    // 1. 修正 2：初始化空的“频道置物架”。
    // 注意：删掉了原来那个单频道的 broadcast::channel(100)，不再需要它了。
    let channels = Arc::new(DashMap::new());

    // 2. 修正 3：先组装好 state。
    // 现在 AppState 结构体里只有 db 和 channels 两个字段。
    let state = AppState { db: pool, channels };

    // 3. 修正 4：利用 state 里的数据库连接去“进货”。
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
        // 👈 这里的 :channel 是路径参数，对应 URL 里的频道名
        .route("/ws/{channel}", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!(
        "🚀 后端引擎已就绪：http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
