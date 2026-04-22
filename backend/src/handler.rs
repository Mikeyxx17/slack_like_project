use crate::models::Channel;
// handler.rs
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

// 🚪 前台接待员：负责拦截升级 HTTP 请求
pub async fn ws_handler(
    ws: Ws,
    // 💡 魔法1：自动从 URL 中抓取频道名字，比如访问 /ws/rust，这里就会提取出 "rust"
    Path(channel_name): Path<String>,
    // 💡 魔法2：拿到我们在 main.rs 里注入的全局 AppState（大脑）
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 升级协议后，把用户请进 handle_socket 这个“房间”
    ws.on_upgrade(|socket| handle_socket(socket, state, channel_name))
}

// 🛋️ 真正的聊天室内部逻辑
async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String) {
    // 1. 拿对讲机：去 DashMap 里找当前频道的广播大喇叭，没有就建一个
    let tx = state.get_or_create_channel(channel_name.clone()).await;

    // 2. 欢迎仪式：组装一条系统欢迎消息
    let welcome_msg = ChatMessage {
        id: None,
        channel: channel_name.clone(),
        username: "系统".to_string(),
        content: format!("一位新伙伴加入了 #{} 聊天室", channel_name),
        created_at: Some(chrono::Utc::now()),
    };

    // 把欢迎消息存入数据库
    let _ = sqlx::query("INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)")
        .bind(&welcome_msg.channel)
        .bind(&welcome_msg.username)
        .bind(&welcome_msg.content)
        .execute(&state.db)
        .await;

    // 用当前频道的大喇叭广播这条欢迎消息
    let _ = tx.send(welcome_msg);

    // 3. 拆分网线：把全双工的 socket 劈成“只发(sender)”和“只收(receiver)”两半
    let (mut sender, mut receiver) = socket.split();
    // 从大喇叭那里领一台只听当前频道的收音机
    let mut rx = tx.subscribe();

    // 4. 翻阅旧账：去数据库捞取这个频道最近的 50 条消息
    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages WHERE channel = $1 ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
    .bind(&channel_name) 
    .fetch_all(&state.db)
    .await;

    // 把捞到的历史消息打包成 JSON，顺着 sender 专线发给当前刚进来的用户
    if let Ok(msgs) = history {
        for msg in msgs {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(_) = sender.send(Message::Text(json.into())).await {
                    return;
                }
            }
        }
    }

    // 5. 并发心脏：死循环监听
    loop {
        tokio::select! {
            // 路线 A：客户端发了一句话过来
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            // 按照 ChatMessage 模具解析前端发来的 JSON
                            match serde_json::from_str::<ChatMessage>(&text) {
                                Ok(mut parsed_msg) => {
                                    // ⚠️ 强制打上当前频道的标签，防止用户在前端篡改频道名
                                    parsed_msg.channel = channel_name.clone();

                                    // 存入数据库
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
                                        let _ = tx.send(parsed_msg);
                                    } else if let Err(e) = db_result {
                                        // 💡 加上这一行，如果数据库报错，你就能在黑窗口看到了！
                                        println!("❌ 数据库写入失败: {}", e);
                                    }
                                }
                                Err(e) => println!("❌ 解析失败: {}", e),
                            }
                        }
                    }
                    _ => break, // 用户断开连接，跳出循环
                }
            }
            // 路线 B：频道里其他人说话了，收音机响了
            recv_result = rx.recv() => {
                if let Ok(broadcast_msg) = recv_result {
                    if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                        // 把别人说的话推送到当前用户的屏幕上
                        if let Err(_) = sender.send(Message::Text(json_text.into())).await {
                            break;
                        }
                    }
                }
            }
        }
    }
}


pub async fn get_channels(State(state):State<AppState>)-> Json<Vec<Channel>> {
    let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
        .fetch_all(&state.db)
        .await
        .unwrap();

    Json(channels)
}
