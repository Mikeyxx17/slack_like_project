use crate::models::ChatMessage;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

// 🧠 系统的全局大脑
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool, // 数据库连接池
    // 🗄️ 频道置物架：用 DashMap 实现高并发安全的内存字典
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>,
}

impl AppState {
    pub async fn get_or_create_channel(&self, name: String) -> broadcast::Sender<ChatMessage> {
        // 第一步 (同步)：如果置物架上已经有这个频道的大喇叭，直接拿走
        if let Some(channel) = self.channels.get(&name) {
            return channel.clone();
        }

        // 第二步 (异步)：没有的话，去数据库登记这个频道 (ON CONFLICT DO NOTHING 保证不重复)
        sqlx::query!(
            "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            name
        )
        .execute(&self.db)
        .await
        .unwrap();

        // 第三步 (同步锁)：在内存里正式创建大喇叭并放上置物架
        let tx = self
            .channels
            .entry(name)
            .or_insert_with(|| {
                let (new_tx, _rx) = broadcast::channel(100);
                new_tx
            })
            .clone();

        tx
    }
}
