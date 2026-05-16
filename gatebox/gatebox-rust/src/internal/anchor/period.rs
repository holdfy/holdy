use chrono::{Datelike, NaiveDate, Utc};

use crate::internal::anchor::types::PeriodType;

/// PeriodFromTime: retorna (day, day_id) como padrão (conforme period.go).
pub fn period_from_time(dt: chrono::DateTime<Utc>) -> (PeriodType, String) {
    let date = dt.date_naive();
    let day_id = date.format("%Y-%m-%d").to_string();
    (PeriodType::Day, day_id)
}

/// Period ID and boundaries (converted from internal/anchor/period.go).
pub fn period_id_for_date(period_type: &str, date: NaiveDate) -> String {
    match period_type {
        "day" => date.format("%Y-%m-%d").to_string(),
        "week" => {
            let week = date.iso_week();
            format!("{}-W{:02}", week.year(), week.week())
        }
        "fortnight" => {
            let day_of_year = date.ordinal();
            let fortnight = (day_of_year as i32 + 13) / 14;
            format!("{}-F{:02}", date.year(), fortnight)
        }
        "month" => date.format("%Y-%m").to_string(),
        "year" => date.format("%Y").to_string(),
        _ => date.format("%Y-%m-%d").to_string(),
    }
}

pub fn current_period_id(period_type: &str) -> String {
    let now = Utc::now().date_naive();
    period_id_for_date(period_type, now)
}
