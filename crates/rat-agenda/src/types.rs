//! Core data types for agenda items and data sources.

/// Simple date representation (year, month 1-12, day 1-31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Day of week: 0=Monday .. 6=Sunday (ISO 8601).
    pub fn weekday(&self) -> u32 {
        // Tomohiko Sakamoto's algorithm
        let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year;
        if self.month < 3 {
            y -= 1;
        }
        let dow =
            (y + y / 4 - y / 100 + y / 400 + t[(self.month - 1) as usize] + self.day as i32) % 7;
        // Convert from Sunday=0 to Monday=0
        ((dow + 6) % 7) as u32
    }

    /// Number of days in this month.
    pub fn days_in_month(&self) -> u32 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.year % 4 == 0 && (self.year % 100 != 0 || self.year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Advance by one day.
    pub fn next_day(self) -> Self {
        let dim = self.days_in_month();
        if self.day < dim {
            Self {
                day: self.day + 1,
                ..self
            }
        } else if self.month < 12 {
            Self {
                month: self.month + 1,
                day: 1,
                ..self
            }
        } else {
            Self {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        }
    }

    /// Go back by one day.
    pub fn prev_day(self) -> Self {
        if self.day > 1 {
            Self {
                day: self.day - 1,
                ..self
            }
        } else if self.month > 1 {
            let prev_month = Self {
                month: self.month - 1,
                day: 1,
                ..self
            };
            Self {
                day: prev_month.days_in_month(),
                ..prev_month
            }
        } else {
            let prev_year = Self {
                year: self.year - 1,
                month: 12,
                day: 1,
            };
            Self {
                day: prev_year.days_in_month(),
                ..prev_year
            }
        }
    }

    /// Advance by N days.
    pub fn add_days(self, n: i32) -> Self {
        let mut d = self;
        if n >= 0 {
            for _ in 0..n {
                d = d.next_day();
            }
        } else {
            for _ in 0..(-n) {
                d = d.prev_day();
            }
        }
        d
    }

    /// First day of this month.
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// Next month, same day (clamped).
    pub fn next_month(self) -> Self {
        let (y, m) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        let dim = Date::new(y, m, 1).days_in_month();
        Date::new(y, m, self.day.min(dim))
    }

    /// Previous month, same day (clamped).
    pub fn prev_month(self) -> Self {
        let (y, m) = if self.month == 1 {
            (self.year - 1, 12)
        } else {
            (self.year, self.month - 1)
        };
        let dim = Date::new(y, m, 1).days_in_month();
        Date::new(y, m, self.day.min(dim))
    }

    /// Monday of the week containing this date.
    pub fn week_start(self) -> Self {
        let wd = self.weekday(); // 0=Mon
        self.add_days(-(wd as i32))
    }

    /// Format as "YYYY-MM-DD".
    pub fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Short month name.
    pub fn month_name(&self) -> &'static str {
        match self.month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "???",
        }
    }

    /// Short day-of-week name.
    pub fn weekday_name(&self) -> &'static str {
        match self.weekday() {
            0 => "Mon",
            1 => "Tue",
            2 => "Wed",
            3 => "Thu",
            4 => "Fri",
            5 => "Sat",
            6 => "Sun",
            _ => "???",
        }
    }
}

/// Simple time representation (24-hour).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
}

impl Time {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }
}

/// A date range (inclusive start, exclusive end).
#[derive(Debug, Clone, Copy)]
pub struct DateRange {
    pub start: Date,
    pub end: Date,
}

impl DateRange {
    pub fn new(start: Date, end: Date) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, date: &Date) -> bool {
        *date >= self.start && *date < self.end
    }
}

/// A single agenda item.
#[derive(Debug, Clone)]
pub struct AgendaItem {
    pub id: String,
    pub title: String,
    pub status: Option<String>,
    pub priority: Option<char>,
    pub tags: Vec<String>,
    pub scheduled: Option<Date>,
    pub deadline: Option<Date>,
    pub time_start: Option<Time>,
    pub time_end: Option<Time>,
    pub source_file: Option<String>,
    pub source_line: Option<usize>,
}

impl AgendaItem {
    /// The primary date for display (scheduled takes precedence).
    pub fn display_date(&self) -> Option<Date> {
        self.scheduled.or(self.deadline)
    }
}

/// Trait for providing agenda data. Object-safe.
pub trait AgendaDataSource {
    /// Return items whose scheduled or deadline falls within `range`.
    fn items(&self, range: DateRange) -> Vec<AgendaItem>;
}

/// Blanket impl: Vec<AgendaItem> as a static data source.
impl AgendaDataSource for Vec<AgendaItem> {
    fn items(&self, range: DateRange) -> Vec<AgendaItem> {
        self.iter()
            .filter(|item| {
                item.scheduled.map_or(false, |d| range.contains(&d))
                    || item.deadline.map_or(false, |d| range.contains(&d))
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_weekday() {
        // 2026-03-26 is a Thursday → weekday 3
        assert_eq!(Date::new(2026, 3, 26).weekday(), 3);
        // 2026-03-23 is Monday → weekday 0
        assert_eq!(Date::new(2026, 3, 23).weekday(), 0);
    }

    #[test]
    fn date_next_prev_day() {
        let d = Date::new(2026, 1, 31);
        assert_eq!(d.next_day(), Date::new(2026, 2, 1));
        assert_eq!(Date::new(2026, 2, 1).prev_day(), Date::new(2026, 1, 31));
    }

    #[test]
    fn date_leap_year() {
        assert_eq!(Date::new(2024, 2, 1).days_in_month(), 29);
        assert_eq!(Date::new(2025, 2, 1).days_in_month(), 28);
    }

    #[test]
    fn date_week_start() {
        // Thursday 2026-03-26 → Monday 2026-03-23
        assert_eq!(Date::new(2026, 3, 26).week_start(), Date::new(2026, 3, 23));
    }

    #[test]
    fn date_range_contains() {
        let r = DateRange::new(Date::new(2026, 3, 1), Date::new(2026, 4, 1));
        assert!(r.contains(&Date::new(2026, 3, 15)));
        assert!(!r.contains(&Date::new(2026, 4, 1))); // exclusive end
    }

    #[test]
    fn vec_data_source() {
        let items = vec![
            AgendaItem {
                id: "1".into(),
                title: "A".into(),
                status: None,
                priority: None,
                tags: vec![],
                scheduled: Some(Date::new(2026, 3, 15)),
                deadline: None,
                time_start: None,
                time_end: None,
                source_file: None,
                source_line: None,
            },
            AgendaItem {
                id: "2".into(),
                title: "B".into(),
                status: None,
                priority: None,
                tags: vec![],
                scheduled: Some(Date::new(2026, 4, 15)),
                deadline: None,
                time_start: None,
                time_end: None,
                source_file: None,
                source_line: None,
            },
        ];
        let range = DateRange::new(Date::new(2026, 3, 1), Date::new(2026, 4, 1));
        let result = items.items(range);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "A");
    }
}
