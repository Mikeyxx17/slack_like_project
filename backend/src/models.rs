use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// 定义数据的“模具”
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Option<i32>,                   // 数据库自动生成的 ID
    pub channel: String,                   // 频道名 (如 #general)
    pub username: String,                  // 发送者用户名
    pub content: String,                   // 消息正文
    pub created_at: Option<DateTime<Utc>>, // 时间字段
}
