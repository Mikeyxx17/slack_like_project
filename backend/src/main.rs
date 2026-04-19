mod handler;
mod models;
mod state;

use axum::{Router, routing::get};
use dotenvy::dotenv;
use handler::ws_handler;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
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

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败，请检查 SQL 脚本");

    let (tx, _rx) = broadcast::channel(1000);
    let state = AppState { db: pool, tx };
    let app = Router::new()
        .route("/ws", get(ws_handler)) // 👈 现在这里可以直接使用 ws_handler 了
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!(
        "🚀 后端引擎已就绪：http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}
