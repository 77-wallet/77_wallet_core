use chrono::{DateTime, Duration, NaiveDateTime, SecondsFormat, Utc};
const FMT_DATETIME: &str = "%Y-%m-%d %H:%M:%S";

// utc now datetime
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_utc_format_time() -> String {
    now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

// 时间格式到时间戳 input is utc datetime
pub fn datetime_to_timestamp(date: &str) -> i64 {
    let time = NaiveDateTime::parse_from_str(date, FMT_DATETIME).unwrap_or_default();
    time.and_utc().timestamp()
}

pub fn now_plus_days(n: i64) -> DateTime<Utc> {
    let now = Utc::now();
    now + Duration::days(n)
}

#[cfg(test)]
mod test {
    use crate::time::{now_plus_days, now_utc_format_time};
    use chrono::{DateTime, Utc};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_time() {
        // 获取当前时间
        let start = SystemTime::now();

        // 将当前时间转换为UNIX时间戳
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        // 输出秒数
        println!("UNIX 时间戳: {}", since_the_epoch.as_secs());
    }

    #[test]
    fn test_plus_days() {
        let now_plus_days = now_plus_days(1);
        println!("now_plus_days: {}", now_plus_days);
    }

    #[test]
    fn test_now() {
        let time = now_utc_format_time();
        println!("{}", time)
    }

    #[test]
    fn test_time_utc() {
        let timestamp = 1729750497;
        let _time = DateTime::from_timestamp(timestamp, 0).unwrap();

        println!("{}", _time);
        println!("now {}", Utc::now());
    }
}
