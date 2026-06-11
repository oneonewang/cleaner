//! 业务核心逻辑(不依赖 Tauri,方便单测)

pub mod browser_cache;
pub mod large_files;
pub mod paths;
pub mod progress;
pub mod registry;
pub mod size;
pub mod system_junk;
pub mod trash;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex;

/// 全局扫描取消标志表
#[derive(Default)]
pub struct CancelRegistry {
    map: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl CancelRegistry {
    pub fn new_flag(&self, scan_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.map.lock().insert(scan_id.to_string(), flag.clone());
        flag
    }

    pub fn cancel(&self, scan_id: &str) {
        if let Some(flag) = self.map.lock().get(scan_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_cancelled(&self, scan_id: &str) -> bool {
        self.map
            .lock()
            .get(scan_id)
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub fn remove(&self, scan_id: &str) {
        self.map.lock().remove(scan_id);
    }
}

use once_cell::sync::Lazy;
pub static CANCEL_REGISTRY: Lazy<CancelRegistry> = Lazy::new(CancelRegistry::default);
