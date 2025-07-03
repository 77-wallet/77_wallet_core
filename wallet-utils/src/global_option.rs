use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 泛型全局容器封装器
pub struct GlobalOption<T: Clone + Send + Sync + 'static> {
    inner: OnceCell<Arc<RwLock<Option<T>>>>,
    default_fn: fn() -> T,
}

impl<T: Clone + Send + Sync + 'static> GlobalOption<T> {
    /// 构造器，传入默认值函数
    pub const fn new(default_fn: fn() -> T) -> Self {
        Self {
            inner: OnceCell::new(),
            default_fn,
        }
    }

    /// 内部获取实例（惰性初始化）
    fn get_cell(&self) -> &Arc<RwLock<Option<T>>> {
        self.inner.get_or_init(|| Arc::new(RwLock::new(None)))
    }

    /// 设置值（异步写入）
    pub async fn set(&self, val: T) {
        let arc = self.get_cell();
        let mut guard = arc.write().await;
        *guard = Some(val);
    }

    /// 获取 Option 值（可判断是否初始化）
    pub async fn get_opt(&self) -> Option<T> {
        let arc = self.get_cell();
        let guard = arc.read().await;
        guard.clone()
    }

    /// 获取值，若未设置则返回默认值
    pub async fn get_or_default(&self) -> T {
        let arc = self.get_cell();
        let guard = arc.read().await;
        guard.clone().unwrap_or_else(self.default_fn)
    }

    /// 是否已经设置过值
    pub async fn is_initialized(&self) -> bool {
        let arc = self.get_cell();
        let guard = arc.read().await;
        guard.is_some()
    }
}
