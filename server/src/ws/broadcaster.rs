use std::sync::Arc;

use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

use crate::ws::events::WsEvent;

type Sender = UnboundedSender<WsEvent>;

#[derive(Clone)]
pub struct Broadcaster {
    inner: Arc<Inner>,
}

struct Inner {
    connections: DashMap<Uuid, Vec<Sender>>,
    pool: PgPool,
}

impl Broadcaster {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: Arc::new(Inner {
                connections: DashMap::new(),
                pool,
            }),
        }
    }

    pub fn register(&self, user_id: Uuid) -> UnboundedReceiver<WsEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.inner.connections.entry(user_id).or_default().push(tx);
        rx
    }

    fn cleanup_dead(&self, user_id: Uuid) {
        if let Some(mut entry) = self.inner.connections.get_mut(&user_id) {
            entry.retain(|tx| !tx.is_closed());
        }
        if let Some(entry) = self.inner.connections.get(&user_id)
            && entry.is_empty()
        {
            drop(entry);
            self.inner.connections.remove(&user_id);
        }
    }

    pub fn to_user(&self, user_id: Uuid, event: WsEvent) {
        let mut had_dead = false;
        if let Some(entry) = self.inner.connections.get(&user_id) {
            for tx in entry.iter() {
                if tx.send(event.clone()).is_err() {
                    had_dead = true;
                }
            }
        }
        if had_dead {
            self.cleanup_dead(user_id);
        }
    }

    pub async fn to_team(&self, team_id: Uuid, event: WsEvent) {
        let user_ids = match sqlx::query_scalar!(
            r#"
            SELECT user_id
            FROM team_members
            WHERE team_id = $1 AND status = 'active'
            "#,
            team_id,
        )
        .fetch_all(&self.inner.pool)
        .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("broadcaster: failed to fetch team members: {e}");
                return;
            }
        };
        for user_id in user_ids {
            self.to_user(user_id, event.clone());
        }
    }
}
