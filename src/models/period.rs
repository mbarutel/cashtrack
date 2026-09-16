use chrono::{Datelike, Days, Months, NaiveDate, TimeDelta, Weekday};
use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::Subcommand)]
pub enum TimePeriod {
    Weekly,
    Fortnightly,
    Monthly,
    Yearly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl TimePeriod {
    pub fn range_containing(self, today: NaiveDate) -> DateRange {
        let week = today.week(Weekday::Mon);

        match self {
            Self::Weekly => DateRange::new(week.first_day(), week.last_day()),
            Self::Fortnightly => DateRange::new(week.first_day() - Days::new(7), week.last_day()),
            Self::Monthly => {
                DateRange::month_starting(today.with_day(1).expect("day 1 is valid for any month"))
            }
            Self::Yearly => DateRange::year_of(today.year()),
        }
    }
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        debug_assert!(start <= end, "DateRange start must not be after end");
        Self { start, end }
    }

    pub fn shift(self, period: TimePeriod, n: i32) -> Self {
        match period {
            TimePeriod::Weekly => self.shift_days(7 * i64::from(n)),
            TimePeriod::Fortnightly => self.shift_days(14 * i64::from(n)),
            TimePeriod::Monthly => Self::month_starting(shift_months(self.start, n)),
            TimePeriod::Yearly => Self::year_of(self.start.year() + n),
        }
    }

    pub fn next(self, period: TimePeriod) -> Self {
        self.shift(period, 1)
    }

    pub fn prev(self, period: TimePeriod) -> Self {
        self.shift(period, -1)
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        self.start <= date && date <= self.end
    }

    fn shift_days(self, days: i64) -> Self {
        let delta = TimeDelta::days(days);
        Self::new(self.start + delta, self.end + delta)
    }

    fn month_starting(first: NaiveDate) -> Self {
        Self::new(first, first + Months::new(1) - Days::new(1))
    }

    fn year_of(year: i32) -> Self {
        Self::new(
            NaiveDate::from_ymd_opt(year, 1, 1).expect("Jan 1 is always valid"),
            NaiveDate::from_ymd_opt(year, 12, 31).expect("Dec 31 is always valid"),
        )
    }
}

fn shift_months(date: NaiveDate, n: i32) -> NaiveDate {
    let months = Months::new(n.unsigned_abs());
    if n >= 0 {
        date + months
    } else {
        date - months
    }
}

impl fmt::Display for TimePeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            Self::Weekly => "Weekly",
            Self::Fortnightly => "Fortnightly",
            Self::Monthly => "Monthly",
            Self::Yearly => "Yearly",
        })
    }
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            self.start.format("%d %b %Y"),
            self.end.format("%d %b %Y"),
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::Days;

    use super::*;

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    const TODAY: (i32, u32, u32) = (2026, 8, 5); // Wed, 05 Aug 2026
    fn today() -> NaiveDate {
        d(TODAY.0, TODAY.1, TODAY.2)
    }

    #[test]
    fn weekly() {
        let r = TimePeriod::Weekly.range_containing(today());
        assert_eq!(r, DateRange::new(d(2026, 8, 3), d(2026, 8, 9)));
    }

    #[test]
    fn weekly_on_monday_and_sunday_edges() {
        let monday = TimePeriod::Weekly.range_containing(d(2026, 8, 3));
        let sunday = TimePeriod::Weekly.range_containing(d(2026, 8, 9));
        assert_eq!(monday, sunday);
        assert_eq!(monday.start, d(2026, 8, 3));
    }

    #[test]
    fn fortnightly() {
        let r = TimePeriod::Fortnightly.range_containing(today());
        assert_eq!(r, DateRange::new(d(2026, 7, 27), d(2026, 8, 9)));
    }

    #[test]
    fn monthly() {
        let r = TimePeriod::Monthly.range_containing(today());
        assert_eq!(r, DateRange::new(d(2026, 8, 1), d(2026, 8, 31)));
    }

    #[test]
    fn monthly_handles_leap_february() {
        let r = TimePeriod::Monthly.range_containing(d(2028, 2, 10));
        assert_eq!(r.end, d(2028, 2, 29));
    }

    #[test]
    fn yearly() {
        let r = TimePeriod::Yearly.range_containing(today());
        assert_eq!(r, DateRange::new(d(2026, 1, 1), d(2026, 12, 31)));
    }

    #[test]
    fn next_fortnight_is_adjacent_not_overlapping() {
        let r = TimePeriod::Fortnightly.range_containing(today());
        let next = r.next(TimePeriod::Fortnightly);
        assert_eq!(next.start, r.end + Days::new(1));
        assert_eq!(next, DateRange::new(d(2026, 8, 10), d(2026, 8, 23)));
    }

    #[test]
    fn prev_month_across_year_boundary() {
        let jan = TimePeriod::Monthly.range_containing(d(2026, 1, 15));
        let dec = jan.prev(TimePeriod::Monthly);
        assert_eq!(dec, DateRange::new(d(2025, 12, 1), d(2025, 12, 31)));
    }

    #[test]
    fn shifting_months_realigns_end_to_month_length() {
        let jan = TimePeriod::Monthly.range_containing(d(2026, 1, 31));
        let feb = jan.next(TimePeriod::Monthly);
        assert_eq!(feb, DateRange::new(d(2026, 2, 1), d(2026, 2, 28)));
    }

    #[test]
    fn shift_is_reversible() {
        for period in [
            TimePeriod::Weekly,
            TimePeriod::Fortnightly,
            TimePeriod::Monthly,
            TimePeriod::Yearly,
        ] {
            let r = period.range_containing(today());
            assert_eq!(r.next(period).prev(period), r, "{period}");
            assert_eq!(r.shift(period, 3).shift(period, -3), r, "{period}");
        }
    }

    #[test]
    fn display() {
        let r = TimePeriod::Monthly.range_containing(today());
        assert_eq!(r.to_string(), "01 Aug 2026 – 31 Aug 2026");
    }
}
