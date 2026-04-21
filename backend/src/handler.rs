use crate::models::ChatMessage;
use crate::state::AppState;
use axum::{
    extract::{
        Path, // 👈 别忘了导入 Path
        State,
        WebSocketUpgrade as Ws,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(
    ws: Ws,
    Path(channel_name): Path<String>, // 👈 抓取 URL 里的频道名
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 把名字传给 handle_socket
    ws.on_upgrade(|socket| handle_socket(socket, state, channel_name))
}

async fn handle_socket(socket: WebSocket, state: AppState, channel_name: String) {
    // 🚀 核心升级：利用你写的超级武器，拿到对应的频道广播器
    let tx = state.get_or_create_channel(channel_name.clone()).await;

    // 接下来的逻辑里，凡是用到 state.tx 的地方，全部换成 tx 即可
    let welcome_msg = ChatMessage {
        id: None,
        channel: channel_name.clone(), // 👈 使用动态频道名
        username: "系统".to_string(),
        content: format!("一位新伙伴加入了 #{} 聊天室", channel_name),
        created_at: Some(chrono::Utc::now()),
    };

    // 存入数据库
    let _ = sqlx::query("INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)")
        .bind(&welcome_msg.channel)
        .bind(&welcome_msg.username)
        .bind(&welcome_msg.content)
        .execute(&state.db)
        .await;

    // 广播给这个频道的所有人
    let _ = tx.send(welcome_msg);

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe(); // 👈 订阅当前频道的广播

    // 加载历史记录时，也只加载当前频道的
    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages WHERE channel = $1 ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
    .bind(&channel_name) // 👈 增加筛选条件
    .fetch_all(&state.db)
    .await;

    if let Ok(msgs) = history {
        for msg in msgs {
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(_) = sender.send(Message::Text(json.into())).await {
                    return;
                }
            }
        }
    }

    loop {
        tokio::select! {
            user_msg = receiver.next() => {
                match user_msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            match serde_json::from_str::<ChatMessage>(&text) {
                                Ok(mut parsed_msg) => {
                                    // 确保消息存入正确的频道
                                    parsed_msg.channel = channel_name.clone();

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
                                        let _ = tx.send(parsed_msg); // 👈 广播到当前频道
                                    }
                                }
                                Err(e) => println!("❌ 解析失败: {}", e),
                            }
                        }
                    }
                    _ => break, // 掉线断开
                }
            }
            recv_result = rx.recv() => {
                if let Ok(broadcast_msg) = recv_result {
                    if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                        if let Err(_) = sender.send(Message::Text(json_text.into())).await {
                            break;
                        }
                    }
                }
            }
        }
    }
}
