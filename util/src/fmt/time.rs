const ONE_SECOND: u64 = 1;
const ONE_MINUTE: u64 = ONE_SECOND * 60;
const ONE_HOUR: u64 = ONE_MINUTE * 60;
const ONE_DAY: u64 = ONE_HOUR * 24;

pub fn format_time(time_secs: u64) -> String {
    if time_secs < ONE_DAY {
        let seconds = time_secs % ONE_MINUTE;
        let minutes = (time_secs / ONE_MINUTE) % ONE_MINUTE;
        let hours = time_secs / ONE_HOUR;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        let remainder = time_secs % ONE_DAY;
        let days = time_secs / ONE_DAY;
        let seconds = remainder % ONE_MINUTE;
        let minutes = (remainder / ONE_MINUTE) % ONE_MINUTE;
        let hours = remainder / ONE_HOUR;
        format!("{days}d {hours:02}:{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time() {
        assert_eq!(format_time(0), "00:00:00");
        assert_eq!(format_time(59), "00:00:59");
        assert_eq!(format_time(60), "00:01:00");
        assert_eq!(format_time(3599), "00:59:59");
        assert_eq!(format_time(3600), "01:00:00");
        assert_eq!(format_time(3661), "01:01:01");
        assert_eq!(format_time(86399), "23:59:59");
        assert_eq!(format_time(86400), "1d 00:00:00");
        assert_eq!(format_time(86401), "1d 00:00:01");
        assert_eq!(format_time(95400), "1d 02:30:00");
        assert_eq!(format_time(8726399), "100d 23:59:59");
    }
}