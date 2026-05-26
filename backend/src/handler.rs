use crate::models::Channel;
use crate::{models::ChatMessage};
use crate::state::AppState;
use axum::{
    extract::{
        Path, State, WebSocketUpgrade as Ws,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    Json
};

use futures::{sink::SinkExt, stream::StreamExt};

// ============================================================
// handler.rs — 路由处理器
//
// 包含三个处理函数：
//   1. ws_handler()       — WebSocket 升级入口
//   2. handle_socket()    — WebSocket 连接生命周期管理（核心）
//   3. get_channels()     — GET  /api/channels  获取频道列表
//   4. create_channel()   — POST /api/channels  创建新频道
//
// 前后端交互说明：
//   - ws_handler 接收前端的 WebSocket 连接请求（new WebSocket("/ws/频道名")）
//   - handle_socket 内部通过 tokio::select! 同时监听两条线路：
//       A. receiver.next() → 前端发来的消息 → 存数据库 → 广播给该频道所有人
//       B. rx.recv()       → 频道内其他人广播的消息 → 推送给当前用户
//   - get_channels / create_channel 是标准 REST API，前端用 fetch() 调用
// ============================================================

/// POST /api/channels 的请求体
#[derive(serde::Deserialize)]
pub struct CreateChannelInput {
    pub name: String,
}

// ============================================================
// WebSocket 升级入口
// 前端通过 new WebSocket("/ws/{channel}") 发起连接时触发
// ============================================================
pub async fn ws_handler(
    ws: Ws,
    Path(channel_name): Path<String>,  // 从 URL 路径提取频道名
    State(state): State<AppState>,     // 全局应用状态
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state, channel_name))
}

// ============================================================
// WebSocket 连接生命周期管理（核心逻辑）
//
// 三个步骤：
//   Step 1 — 加入频道：获取/创建 broadcast channel，发欢迎消息
//   Step 2 — 发送历史：查询最近 50 条消息推送给新用户
//   Step 3 — 事件循环：tokio::select! 同时监听「用户发言」和「频道广播」
// ============================================================
async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String) {
    // Step 1: 加入频道
    // 从 DashMap 中获取或创建该频道的 broadcast::Sender
    let tx = state.get_or_create_channel(channel_name.clone()).await;

    // 构造系统欢迎消息
    let welcome_msg = ChatMessage {
        id: None,
        channel: channel_name.clone(),
        username: "系统".to_string(),
        content: format!("一位新伙伴加入了 #{} 聊天室", channel_name),
        created_at: Some(chrono::Utc::now()),
    };

    // 欢迎消息持久化到数据库
    let _ = sqlx::query("INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)")
        .bind(&welcome_msg.channel)
        .bind(&welcome_msg.username)
        .bind(&welcome_msg.content)
        .execute(&state.db)
        .await;

    // 通过 broadcast channel 广播欢迎消息（该频道所有在线用户可见）
    let _ = tx.send(welcome_msg);

    // 拆分 WebSocket 为发送端和接收端
    let (mut sender, mut receiver) = socket.split();
    // 订阅该频道的广播（获取 Receiver）
    let mut rx = tx.subscribe();

    // Step 2: 发送历史消息
    // 查询最近 50 条消息，按时间正序排列
    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages WHERE channel = $1 ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
    .bind(&channel_name)
    .fetch_all(&state.db)
    .await;

    // 逐条推送给新连接的用户
    if let Ok(msgs) = history {
        for msg in msgs {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(_) = sender.send(Message::Text(json.into())).await {
                    return;
                }
            }
        }
    }

    // Step 3: 事件循环
    loop {
        tokio::select! {
            // ====================================================
            // 线路 A: 当前用户通过前端 WebSocket 发送了一条消息
            // 数据流: 前端 socket.send(JSON) → 后端 receiver → 存 DB → broadcast → 所有该频道用户
            // ====================================================
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            // 解析前端发来的 JSON（格式: {channel, username, content}）
                            match serde_json::from_str::<ChatMessage>(&text) {
                                Ok(mut parsed_msg) => {
                                    // 强制覆盖 channel 字段，防止前端篡改频道名跨频道灌水
                                    parsed_msg.channel = channel_name.clone();

                                    // 写入数据库
                                    let db_result = sqlx::query(
                                        "INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)"
                                    )
                                    .bind(&parsed_msg.channel)
                                    .bind(&parsed_msg.username)
                                    .bind(&parsed_msg.content)
                                    .execute(&state.db)
                                    .await;

                                    if let Ok(_) = db_result {
                                        parsed_msg.created_at = Some(chrono::Utc::now());
                                        // 广播给该频道的所有订阅者（包括发送者自己）
                                        let _ = tx.send(parsed_msg);
                                    } else if let Err(e) = db_result {
                                        println!("数据库写入失败: {}", e);
                                    }
                                }
                                Err(e) => println!("解析失败: {}", e),
                            }
                        }
                    }
                    _ => break,  // WebSocket 断开，退出循环
                }
            }

            // ====================================================
            // 线路 B: 频道内其他人通过 broadcast 发送了消息
            // 数据流: 其他人发言 → tx.send → broadcast channel → rx.recv → sender.send → 前端 onmessage
            // ====================================================
            recv_result = rx.recv() => {
                if let Ok(broadcast_msg) = recv_result {
                    if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                        // 将广播消息推送给当前用户的 WebSocket 连接
                        if let Err(_) = sender.send(Message::Text(json_text.into())).await {
                            break;
                        }
                    }
                }
            }
        }
    }
}

// ============================================================
// GET /api/channels
// 前端通过 fetch('/api/channels') 调用，用于加载侧边栏频道列表
// 返回: Channel[] JSON 数组
// ============================================================
pub async fn get_channels(State(state): State<AppState>) -> Json<Vec<Channel>> {
    let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(channels)
}

// ============================================================
// POST /api/channels
// 前端通过 fetch('/api/channels', {method: 'POST', body: ...}) 调用
// 请求体: {"name": "频道名"}
// 返回: 201 (创建成功) 或 500 (服务器错误)
// ============================================================
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
        Ok(_) => axum::http::StatusCode::CREATED,            // 201
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR, // 500
    }
}
