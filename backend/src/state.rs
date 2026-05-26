use crate::models::ChatMessage;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

// ============================================================
// state.rs — 全局应用状态
//
// AppState 是整个后端服务的共享状态，包含：
//   1. db: PostgreSQL 连接池，所有数据库操作共用
//   2. channels: DashMap 存储「频道名 → broadcast::Sender」映射
//      - DashMap 是并发安全的 HashMap，多线程读写无需额外锁
//      - 每个频道对应一个 broadcast channel（容量 100），实现发布-订阅
//
// get_or_create_channel():
//   用户连接到某频道时调用，确保该频道的 broadcast channel 存在。
//   分三步：查内存 → 登记数据库 → 创建内存条目
// ============================================================

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>,
}

impl AppState {
    /// 获取或创建指定频道的 broadcast::Sender。
    /// 先查内存（DashMap），未命中则写入数据库并创建新的 broadcast channel。
    /// 返回的 Sender 可用于广播消息，Receiver 可通过 subscribe() 获取。
    pub async fn get_or_create_channel(&self, name: String) -> broadcast::Sender<ChatMessage> {
        // ① 同步检查：DashMap 中已有该频道则直接返回
        if let Some(channel) = self.channels.get(&name) {
            return channel.clone();
        }

        // ② 异步写入：向数据库登记频道（ON CONFLICT DO NOTHING 保证幂等）
        sqlx::query!(
            "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            name
        )
        .execute(&self.db)
        .await
        .unwrap();

        // ③ 同步创建：在 DashMap 中原子创建新的 broadcast channel
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
