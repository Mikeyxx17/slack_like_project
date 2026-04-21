use crate::models::ChatMessage;
use dashmap::DashMap;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    // pub tx: broadcast::Sender<ChatMessage>, // 单一频道广播
    pub channels: Arc<DashMap<String, broadcast::Sender<ChatMessage>>>, // 多频道广播
}

impl AppState {
    pub async fn get_or_create_channel(&self, name: String) -> broadcast::Sender<ChatMessage> {
        // ⚡️ 第一步 (同步)：快速检查。99% 的情况下，频道都已经存在，这里瞬间返回。
        if let Some(channel) = self.channels.get(&name) {
            return channel.clone();
        }

        // 🐢 第二步 (异步)：如果没找到，大家一起去数据库尝试插入。
        // 利用 ON CONFLICT DO NOTHING，无论多少人并发，数据库里只会有一条记录。
        sqlx::query!(
            "INSERT INTO channels (name) VALUES ($1) ON CONFLICT (name) DO NOTHING",
            name
        )
        .execute(&self.db)
        .await
        .unwrap();

        // 🛡️ 第三步 (同步锁)：数据库完事了，现在大家要在内存里抢着创建 Sender。
        // entry() 像是一个带锁的旋转门，一次只放一个人进。
        let tx = self
            .channels
            .entry(name)
            .or_insert_with(|| {
                // 只有抢到第一名的那个线程，才会执行这里的代码创建新对讲机。
                // 后面的人挤进来时，发现对讲机已经建好了，就会直接拿走备份！
                let (new_tx, _rx) = broadcast::channel(100);
                new_tx
            })
            .clone(); // 拿着对讲机的分身离开

        tx
    }
}
