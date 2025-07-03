use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 异步初始化 + 可选默认值 + 异步读写共享状态
///
/// 支持：
/// 
/// — 异步单次初始化（异步函数）
///
/// — 默认值支持（同步默认值函数）
///
/// — 异步读写（RwLock 保护）
///
/// T 需要实现 Clone + Send + Sync + 'static
pub struct AsyncSharedState<T: Clone + Send + Sync + 'static> {
    inner: OnceCell<Arc<RwLock<Option<T>>>>,
    default_fn: Option<fn() -> T>,
}

impl<T: Clone + Send + Sync + 'static> AsyncSharedState<T> {
    /// 创建实例，可选传入默认值函数
    pub const fn new(default_fn: Option<fn() -> T>) -> Self {
        Self {
            inner: OnceCell::new(),
            default_fn,
        }
    }

    /// 获取内部的 Arc<RwLock<Option<T>>>，延迟初始化
    fn get_cell(&self) -> &Arc<RwLock<Option<T>>> {
        self.inner.get_or_init(|| Arc::new(RwLock::new(None)))
    }

    /// 异步单次初始化
    /// 只在首次调用时生效，后续调用无效
    pub async fn init_once<F, Fut>(&self, f: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let arc = self.get_cell();
        let mut guard = arc.write().await;
        if guard.is_none() {
            let val = f().await;
            *guard = Some(val);
        }
    }

    /// 异步读取 Option<T>
    pub async fn get(&self) -> Option<T> {
        let arc = self.get_cell();
        let guard = arc.read().await;
        guard.clone()
    }

    /// 异步读取，若无值则返回默认值（同步默认函数）
    pub async fn get_or_default(&self) -> T {
        let arc = self.get_cell();
        let guard = arc.read().await;
        if let Some(v) = &*guard {
            v.clone()
        } else if let Some(def_fn) = self.default_fn {
            def_fn()
        } else {
            panic!("AsyncSharedState: no value set and no default function provided");
        }
    }

    /// 异步写入，覆盖旧值
    pub async fn set(&self, val: T) {
        let arc = self.get_cell();
        let mut guard = arc.write().await;
        *guard = Some(val);
    }

    /// 是否已初始化（有值）
    pub async fn is_initialized(&self) -> bool {
        let arc = self.get_cell();
        let guard = arc.read().await;
        guard.is_some()
    }
}
