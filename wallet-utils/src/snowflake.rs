use once_cell::sync::Lazy;
use std::{
    hash::{Hash as _, Hasher as _},
    sync::Mutex,
};

static WORKER: Lazy<Mutex<SnowflakeIdWorkerInner>> = Lazy::new(|| {
    Mutex::new(SnowflakeIdWorkerInner::new(1, 1).expect("Failed to create SnowflakeIdWorkerInner"))
});

// 2023-05-24
const TWEPOCH: u128 = 1684922700000;
const WORKER_ID_BITS: u128 = 5;
const DATA_CENTER_ID_BITS: u128 = 5;
// 31
pub(crate) const MAX_WORKER_ID: u128 = (-1 ^ (-1 << WORKER_ID_BITS)) as u128;
// 31
const MAX_DATA_CENTER_ID: u128 = (-1 ^ (-1 << DATA_CENTER_ID_BITS)) as u128;
const SEQUENCE_BITS: u128 = 12;
const WORKER_ID_SHIFT: u128 = SEQUENCE_BITS;
const DATA_CENTER_ID_SHIFT: u128 = SEQUENCE_BITS + WORKER_ID_BITS;
const TIMESTAMP_LEFT_SHIFT: u128 = SEQUENCE_BITS + WORKER_ID_BITS + DATA_CENTER_ID_BITS;
// 4095
const SEQUENCE_MASK: u128 = (-1 ^ (-1 << SEQUENCE_BITS)) as u128;

pub struct SnowflakeIdWorkerInner {
    worker_id: u128,
    data_center_id: u128,
    sequence: u128,
    last_timestamp: u128,
}

impl SnowflakeIdWorkerInner {
    pub(crate) fn new(
        worker_id: u128,
        data_center_id: u128,
    ) -> Result<SnowflakeIdWorkerInner, crate::error::Error> {
        if worker_id > MAX_WORKER_ID {
            return Err(
                crate::error::SnowflakeError::WorkerIdInvalid(worker_id, MAX_WORKER_ID).into(),
            );
        }

        if data_center_id > MAX_DATA_CENTER_ID {
            return Err(crate::error::SnowflakeError::DataCenterIdInvalid(
                data_center_id,
                MAX_DATA_CENTER_ID,
            )
            .into());
        }

        Ok(SnowflakeIdWorkerInner {
            worker_id,
            data_center_id,
            sequence: 0,
            last_timestamp: 0,
        })
    }

    pub fn next_id(&mut self) -> Result<u64, crate::Error> {
        let mut timestamp = Self::get_time()?;
        if timestamp < self.last_timestamp {
            return Err(crate::error::SnowflakeError::ClockMoveBackward(
                self.last_timestamp - timestamp,
            )
            .into());
        }

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = Self::til_next_mills(self.last_timestamp)?;
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        Ok((((timestamp - TWEPOCH) << TIMESTAMP_LEFT_SHIFT)
            | (self.data_center_id << DATA_CENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence)
            .try_into()
            .unwrap())
    }

    fn til_next_mills(last_timestamp: u128) -> Result<u128, crate::Error> {
        let mut timestamp = Self::get_time()?;
        while timestamp <= last_timestamp {
            timestamp = Self::get_time()?;
        }
        Ok(timestamp)
    }

    fn get_time() -> Result<u128, crate::Error> {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(s) => Ok(s.as_millis()),
            Err(_) => Err(crate::error::SnowflakeError::GetTimeFailed.into()),
        }
    }
}

pub fn get_uid() -> Result<u64, crate::Error> {
    let mut worker = WORKER.lock().unwrap();
    // let mut worker = SnowflakeIdWorkerInner::new(1, 1)?;
    let id = worker.next_id()?;
    Ok(id)
}

pub fn gen_hash_uid(params: Vec<&str>) -> String {
    let id = params.join("").to_string();
    let mut s = std::hash::DefaultHasher::new();
    id.hash(&mut s);
    s.finish().to_string()
}
