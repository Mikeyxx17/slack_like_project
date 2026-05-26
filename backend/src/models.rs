use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================
// models.rs — 数据模型定义
//
// ChatMessage: 聊天消息结构体
//   - 前端通过 WebSocket 发送 JSON，后端反序列化为此结构体
//   - 后端广播消息时序列化为 JSON 推送给前端
//   - 数据库 messages 表的读写映射（sqlx::FromRow）
//
// Channel: 频道结构体
//   - 仅用于 GET /api/channels 接口的 JSON 响应
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChatMessage {
    pub id: Option<i32>,
    pub channel: String,
    pub username: String,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
