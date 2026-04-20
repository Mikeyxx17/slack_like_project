use crate::models::ChatMessage;
use crate::state::AppState;
use axum::{
    extract::{
        State, WebSocketUpgrade as Ws,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn ws_handler(ws: Ws, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    // 1. 制作欢迎卡片
    let welcome_msg = ChatMessage {
        id: None,
        channel: "general".to_string(),
        username: "系统".to_string(),
        content: "一位新伙伴加入了聊天室".to_string(),
        created_at: Some(chrono::Utc::now()),
    };

    // 2. 🗄️ 新增：将欢迎消息正式登记入库
    let _ = sqlx::query("INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)")
        .bind(&welcome_msg.channel)
        .bind(&welcome_msg.username)
        .bind(&welcome_msg.content)
        .execute(&state.db)
        .await;

    // 3. 广播给所有人（大喇叭）
    let _ = state.tx.send(welcome_msg);

    println!("🔌 一个新的 WebSocket 连接已建立");

    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    let history = sqlx::query_as::<_, ChatMessage>(
        "SELECT * FROM (SELECT * FROM messages ORDER BY id DESC LIMIT 50) AS sub ORDER BY id ASC",
    )
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
                                    let db_result = sqlx::query(
                                        "INSERT INTO messages (channel, username, content) VALUES ($1, $2, $3)"
                                    )
                                    .bind(&parsed_msg.channel)
                                    .bind(&parsed_msg.username)
                                    .bind(&parsed_msg.content)
                                    .execute(&state.db)
                                    .await;

                                    if let Err(e) = db_result {
                                        println!("❌ 存入数据库失败: {}", e);
                                    } else {
                                        parsed_msg.created_at = Some(chrono::Utc::now());
                                        let _ = state.tx.send(parsed_msg);
                                    }
                                }
                                Err(e) => println!("❌ 消息格式解析失败: {}", e),
                            }
                        }
                    }
                    _ => {
                        println!("🔌 客户端已主动断开连接");
                        break;
                    }
                }
            }
            recv_result = rx.recv() => {
                match recv_result {
                    Ok(broadcast_msg) => {
                        if let Ok(json_text) = serde_json::to_string(&broadcast_msg) {
                            if let Err(e) = sender.send(Message::Text(json_text.into())).await {
                                println!("❌ 消息推送失败（用户已掉线）: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("⚠️ 广播接收异常: {}", e);
                    }
                }
            }
        }
    }
}
