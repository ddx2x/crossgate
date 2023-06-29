use chrono::{DateTime, FixedOffset, TimeZone, Utc};

pub fn date_to_unixtime_range(date: &str) -> (i64, i64) {
    let dt = FixedOffset::east_opt(8 * 3600)
        .unwrap() // UTC+8 offset in seconds
        .datetime_from_str(&format!("{} 00:00:00 +0800", date), "%Y-%m-%d %H:%M:%S %z")
        .unwrap();
    let start_unixtime = dt.timestamp();
    let end_unixtime = start_unixtime + 24 * 60 * 60 - 1; // 24 hours in seconds minus 1 second
    (start_unixtime, end_unixtime)
}

pub fn unixtime_to_date(unixtime: i64) -> String {
    let dt = FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .timestamp(unixtime, 0);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod test {
    use crate::utils::time::date_to_unixtime_range;

    #[test]
    fn unix_timestamp_u64() {
        let (start, end) = date_to_unixtime_range("2021-01-01");
        assert_eq!(start, 1609430400);
        assert_eq!(end, 1609516799);
    }
}
