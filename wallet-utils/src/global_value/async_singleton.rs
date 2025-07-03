use std::sync::Arc;
use tokio::sync::OnceCell;

/// 异步单例，只初始化一次且不可变
pub struct AsyncSingleton<T: Send + Sync + 'static> {
    inner: OnceCell<Arc<T>>,
}

impl<T: Send + Sync + 'static> AsyncSingleton<T> {
    /// 构造函数，返回空容器
    pub const fn new() -> Self {
        Self {
            inner: OnceCell::const_new(),
        }
    }

    /// 异步初始化，如果已经初始化直接返回
    /// 用法：`get_or_init(async || { ... }).await`
    pub async fn get_or_init<F, Fut>(&self, f: F) -> Arc<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.inner
            .get_or_init(|| async {
                let val = f().await;
                Arc::new(val)
            })
            .await
            .clone()
    }

    /// 判断是否已经初始化
    pub fn is_initialized(&self) -> bool {
        self.inner.get().is_some()
    }
}
