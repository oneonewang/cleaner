//! 共享命令工具

use std::sync::Arc;

use crate::core::CANCEL_REGISTRY;

/// 申请一个 scan_id 并返回它的取消标志
pub fn new_cancel(scan_id: &str) -> Arc<std::sync::atomic::AtomicBool> {
    CANCEL_REGISTRY.new_flag(scan_id)
}

/// 移除 scan_id 注册
pub fn drop_cancel(scan_id: &str) {
    CANCEL_REGISTRY.remove(scan_id)
}
