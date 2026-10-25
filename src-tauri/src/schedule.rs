//! Weekly schedule maths, kept free of clocks and time zones so it can be tested.
//!
//! The schedule fires at most once per week. If the computer was off at the
//! scheduled time, the clean runs the next time CleanCraft is running.

use crate::settings::Schedule;
use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime};

/// The latest scheduled moment at or before `now` (local time).
pub fn most_recent_slot(now: NaiveDateTime, schedule: &Schedule) -> NaiveDateTime {
    let today = now.date();
    let days_back =
        (today.weekday().num_days_from_monday() as i64 - schedule.weekday as i64).rem_euclid(7);
    let time = NaiveTime::from_hms_opt(schedule.hour.min(23) as u32, 0, 0).expect("valid hour");
    let slot = (today - Duration::days(days_back)).and_time(time);
    if slot > now {
        slot - Duration::days(7)
    } else {
        slot
    }
}

/// True when the schedule is on and its latest slot has not been served yet.
pub fn is_due(now: NaiveDateTime, last_run: Option<NaiveDateTime>, schedule: &Schedule) -> bool {
    if !schedule.enabled {
        return false;
    }
    let slot = most_recent_slot(now, schedule);
    last_run.is_none_or(|last| last < slot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // 2026-01-05 is a Monday.
    fn at(day: u32, hour: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 1, day)
            .unwrap()
            .and_hms_opt(hour, min, 0)
            .unwrap()
    }

    fn sunday_10() -> Schedule {
        Schedule {
            enabled: true,
            weekday: 6,
            hour: 10,
        }
    }

    #[test]
    fn most_recent_slot_looks_back_to_the_scheduled_weekday() {
        // Wednesday 7th → previous Sunday 4th, 10:00.
        assert_eq!(most_recent_slot(at(7, 12, 0), &sunday_10()), at(4, 10, 0));
        // Sunday 11th 09:59 → still the previous Sunday.
        assert_eq!(most_recent_slot(at(11, 9, 59), &sunday_10()), at(4, 10, 0));
        // Sunday 11th 10:00 → today.
        assert_eq!(most_recent_slot(at(11, 10, 0), &sunday_10()), at(11, 10, 0));
    }

    #[test]
    fn due_once_per_slot() {
        let s = sunday_10();
        assert!(is_due(at(11, 10, 1), None, &s), "never ran");
        assert!(
            is_due(at(11, 10, 1), Some(at(4, 10, 5)), &s),
            "last week's run"
        );
        assert!(
            !is_due(at(11, 10, 30), Some(at(11, 10, 1)), &s),
            "already ran this slot"
        );
        assert!(
            !is_due(at(13, 8, 0), Some(at(11, 10, 1)), &s),
            "mid-week after the run"
        );
    }

    #[test]
    fn missed_slot_runs_on_the_next_check() {
        // Computer was off on Sunday; it is Tuesday now and the last run was a week earlier.
        assert!(is_due(at(13, 9, 0), Some(at(4, 10, 0)), &sunday_10()));
    }

    #[test]
    fn disabled_schedule_is_never_due() {
        let s = Schedule {
            enabled: false,
            ..sunday_10()
        };
        assert!(!is_due(at(11, 10, 1), None, &s));
    }

    #[test]
    fn monday_midnight_schedule() {
        let s = Schedule {
            enabled: true,
            weekday: 0,
            hour: 0,
        };
        assert_eq!(most_recent_slot(at(5, 0, 0), &s), at(5, 0, 0));
        assert_eq!(most_recent_slot(at(11, 23, 59), &s), at(5, 0, 0));
    }
}
