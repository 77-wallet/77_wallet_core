#[derive(serde::Serialize, thiserror::Error, Debug, Clone)]
pub enum SnowflakeError {
    #[error("Clock moved backwards.  Refusing to generate id for {0} milliseconds")]
    ClockMoveBackward(u128),
    #[error("datacenterId:{0} must be less than {1}")]
    DataCenterIdInvalid(u128, u128),
    #[error("workerId:{0} must be less than {1}")]
    WorkerIdInvalid(u128, u128),
    #[error("get_time error!")]
    GetTimeFailed,
}
