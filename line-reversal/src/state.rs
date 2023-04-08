use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use crate::{
    errors::LRCPError,
    types::{PosDataStruct, Session, SocketAddrSessionDb},
};

use tokio::sync::Mutex;

// Reference: https://github.com/tokio-rs/mini-redis/blob/master/src/db.rs
#[derive(Debug, Clone)]
pub struct Db {
    /// Handle to shared state. The background task will also have an
    /// `Arc<Shared>`.
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    /// The shared state is guarded by a mutex.
    ///
    /// NORMALLY! This would be a `std::sync::Mutex` and
    /// not a Tokio mutex. This is because there are no asynchronous operations
    /// being performed while holding the mutex. Additionally, the critical
    /// sections are very small.
    ///
    /// A Tokio mutex is mostly intended to be used when locks need to be held
    /// across `.await` yield points. All other cases are **usually** best
    /// served by a std mutex. If the critical section does not include any
    /// async operations but is long (CPU intensive or performing blocking
    /// operations), then the entire operation, including waiting for the mutex,
    /// is considered a "blocking" operation and `tokio::task::spawn_blocking`
    /// should be used.
    ///
    /// However, it is much easier to use tokio Mutex to avail of the ? operator.
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    sessions: SocketAddrSessionDb,
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}

impl Db {
    pub fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                sessions: HashMap::new(),
            }),
        });
        Db { shared }
    }

    pub async fn add_session(&self, addr: SocketAddr, session: Session, pos_data: PosDataStruct) {
        let mut state = self.shared.state.lock().await;

        //NOTE: the entry API is used here
        state
            .sessions
            .entry(addr)
            .or_default()
            .insert(session, pos_data);
    }
}
