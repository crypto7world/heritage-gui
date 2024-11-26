use btc_heritage_wallet::Database;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

fn database_rwlock() -> &'static RwLock<Database> {
    static DATABASE: OnceLock<RwLock<Database>> = OnceLock::new();
    DATABASE.get_or_init(|| {
        let config = crate::clients::config();
        RwLock::new(
            Database::new(&config.datadir, config.network).expect("Could not open the database"),
        )
    })
}

pub fn database() -> RwLockReadGuard<'static, Database> {
    database_rwlock().read().unwrap()
}

pub fn database_mut() -> RwLockWriteGuard<'static, Database> {
    database_rwlock().write().unwrap()
}
