use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct CTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub auto: bool, // 自适应对天的理解
    pub ts: i64,    // Unix timestamp
}

impl CTime {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        auto: bool,
    ) -> Self {
        let mut time = Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            auto,
            ts: 0,
        };
        time.set_timestamp();
        time
    }

    pub fn to_string(&self) -> String {
        if self.hour == 0 && self.minute == 0 {
            format!("{:04}/{:02}/{:02}", self.year, self.month, self.day)
        } else {
            format!(
                "{:04}/{:02}/{:02} {:02}:{:02}",
                self.year, self.month, self.day, self.hour, self.minute
            )
        }
    }

    pub fn to_date_str(&self, splt: &str) -> String {
        format!(
            "{:04}{}{:02}{}{:02}",
            self.year, splt, self.month, splt, self.day
        )
    }

    pub fn to_date(&self) -> Self {
        Self::new(self.year, self.month, self.day, 0, 0, 0, false)
    }

    pub fn set_timestamp(&mut self) {
        let datetime = if self.hour == 0 && self.minute == 0 && self.auto {
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                chrono::NaiveTime::from_hms_opt(23, 59, self.second).unwrap(),
            )
        } else {
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap(),
                chrono::NaiveTime::from_hms_opt(self.hour, self.minute, self.second).unwrap(),
            )
        };
        self.ts = datetime.timestamp();
    }
}

impl PartialEq for CTime {
    fn eq(&self, other: &Self) -> bool {
        self.ts == other.ts
    }
}

impl PartialOrd for CTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ts.partial_cmp(&other.ts)
    }
}

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Time {
    pub timestamp: i64,
    datetime: NaiveDateTime,
}

impl Time {
    pub fn new(timestamp: i64) -> Self {
        let datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).expect("Invalid timestamp");
        Self {
            timestamp,
            datetime,
        }
    }

    pub fn from_str(time_str: &str) -> Result<Self, String> {
        // Supports multiple formats: "YYYY-MM-DD HH:MM:SS" or "YYYYMMDD"
        let datetime = if time_str.contains('-') {
            NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| e.to_string())?
        } else {
            let date = NaiveDate::parse_from_str(time_str, "%Y%m%d").map_err(|e| e.to_string())?;
            date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        };

        Ok(Self {
            timestamp: datetime.timestamp(),
            datetime,
        })
    }

    pub fn to_str(&self) -> String {
        self.datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn to_date_str(&self) -> String {
        self.datetime.format("%Y%m%d").to_string()
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
