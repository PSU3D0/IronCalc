use chrono::DateTime;
use chrono::Datelike;
use chrono::Months;
use chrono::Timelike;

use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::dates::date_to_serial_number;
use crate::formatter::dates::permissive_date_to_serial_number;
use crate::model::get_milliseconds_since_epoch;
use crate::{
    calc_result::CalcResult, constants::EXCEL_DATE_BASE, expressions::parser::Node,
    expressions::token::Error, formatter::dates::from_excel_date, model::Model,
};

impl Model {
    pub(crate) fn fn_day(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let day = date.day() as f64;
        CalcResult::Number(day)
    }

    pub(crate) fn fn_month(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let month = date.month() as f64;
        CalcResult::Number(month)
    }

    pub(crate) fn fn_eomonth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Function EOMONTH parameter 1 value is negative. It should be positive or zero.".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        if serial_number > MAXIMUM_DATE_SERIAL_NUMBER as i64 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Function DAY parameter 1 value is too large.".to_string(),
            };
        }

        let months = match self.get_number_no_bools(&args[1], cell) {
            Ok(c) => {
                let t = c.trunc();
                t as i32
            }
            Err(s) => return s,
        };

        let months_abs = months.unsigned_abs();

        let native_date = if months > 0 {
            date + Months::new(months_abs)
        } else {
            date - Months::new(months_abs)
        };

        // Instead of calculating the end of month we compute the first day of the following month
        // and take one day.
        let mut month = native_date.month() + 1;
        let mut year = native_date.year();
        if month == 13 {
            month = 1;
            year += 1;
        }
        match date_to_serial_number(1, month, year) {
            Ok(serial_number) => CalcResult::Number(serial_number as f64 - 1.0),
            Err(message) => CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message,
            },
        }
    }

    // year, month, day
    pub(crate) fn fn_date(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let year = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i32;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let month = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.floor();
                t as i32
            }
            Err(s) => return s,
        };
        let day = match self.get_number(&args[2], cell) {
            Ok(c) => {
                let t = c.floor();
                t as i32
            }
            Err(s) => return s,
        };
        match permissive_date_to_serial_number(day, month, year) {
            Ok(serial_number) => CalcResult::Number(serial_number as f64),
            Err(message) => CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message,
            },
        }
    }

    pub(crate) fn fn_year(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };
        let year = date.year() as f64;
        CalcResult::Number(year)
    }

    // date, months
    pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let date = match from_excel_date(serial_number) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                }
            }
        };

        let months = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.trunc();
                t as i32
            }
            Err(s) => return s,
        };

        let months_abs = months.unsigned_abs();

        let native_date = if months > 0 {
            date + Months::new(months_abs)
        } else {
            date - Months::new(months_abs)
        };

        let serial_number = native_date.num_days_from_ce() - EXCEL_DATE_BASE;
        if serial_number < MINIMUM_DATE_SERIAL_NUMBER {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "EDATE out of bounds".to_string(),
            };
        }
        CalcResult::Number(serial_number as f64)
    }

    pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 0 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        // milliseconds since January 1, 1970 00:00:00 UTC.
        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let local_time = match DateTime::from_timestamp(seconds, 0) {
            Some(dt) => dt.with_timezone(&self.tz),
            None => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Invalid date".to_string(),
                }
            }
        };
        // 693_594 is computed as:
        // NaiveDate::from_ymd(1900, 1, 1).num_days_from_ce() - 2
        // The 2 days offset is because of Excel 1900 bug
        let days_from_1900 = local_time.num_days_from_ce() - 693_594;

        CalcResult::Number(days_from_1900 as f64)
    }

    pub(crate) fn fn_now(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 0 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        // milliseconds since January 1, 1970 00:00:00 UTC.
        let milliseconds = get_milliseconds_since_epoch();
        let seconds = milliseconds / 1000;
        let local_time = match DateTime::from_timestamp(seconds, 0) {
            Some(dt) => dt.with_timezone(&self.tz),
            None => {
                return CalcResult::Error {
                    error: Error::ERROR,
                    origin: cell,
                    message: "Invalid date".to_string(),
                }
            }
        };
        // 693_594 is computed as:
        // NaiveDate::from_ymd(1900, 1, 1).num_days_from_ce() - 2
        // The 2 days offset is because of Excel 1900 bug
        let days_from_1900 = local_time.num_days_from_ce() - 693_594;
        let days = (local_time.num_seconds_from_midnight() as f64) / (60.0 * 60.0 * 24.0);

        CalcResult::Number(days_from_1900 as f64 + days.fract())
    }

    // DATEDIF(start_date, end_date, unit)
    // Note: This function is hidden in Excel but widely used and documented.
    // It mimics the behavior of Excel's DATEDIF.
    pub(crate) fn fn_datedif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let start_date_serial = match self.get_number(&args[0], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };
        let end_date_serial = match self.get_number(&args[1], cell) {
            Ok(c) => c.floor() as i64,
            Err(s) => return s,
        };

        let unit = match self.get_string(&args[2], cell) {
            Ok(s) => s.to_uppercase(),
            Err(e) => return e,
        };

        if start_date_serial > end_date_serial {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Function DATEDIF requires start_date <= end_date".to_string(),
            };
        }

        let start_date = match from_excel_date(start_date_serial) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Function DATEDIF parameter 1 has incorrect date value".to_string(),
                }
            }
        };

        let end_date = match from_excel_date(end_date_serial) {
            Ok(date) => date,
            Err(_) => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Function DATEDIF parameter 2 has incorrect date value".to_string(),
                }
            }
        };

        match unit.as_str() {
            "Y" => {
                // Calculate full years
                let mut years = end_date.year() - start_date.year();
                // Check if the end date's month/day is earlier than the start date's month/day
                if end_date.month() < start_date.month()
                    || (end_date.month() == start_date.month() && end_date.day() < start_date.day())
                {
                    years -= 1;
                }
                CalcResult::Number(years as f64)
            }
            "M" => {
                // Calculate full months
                let mut months = (end_date.year() * 12 + end_date.month() as i32)
                    - (start_date.year() * 12 + start_date.month() as i32);
                // Adjust if the end date's day is earlier than the start date's day
                if end_date.day() < start_date.day() {
                    months -= 1;
                }
                CalcResult::Number(months as f64)
            }
            "D" => {
                // Calculate days
                CalcResult::Number((end_date_serial - start_date_serial) as f64)
            }
            "MD" => {
                // Calculate days ignoring months and years
                let mut day_diff = end_date.day() as i32 - start_date.day() as i32;
                if day_diff < 0 {
                    // Find the last day of the month *before* the end_date's month
                    // This handles borrowing correctly across month boundaries and leap years.
                    let prev_month_last_day = match end_date.with_day(1) {
                        Some(first_of_month) => (first_of_month - chrono::Duration::days(1)).day(),
                        None => {
                            // Should not happen for valid dates
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Internal error calculating MD in DATEDIF".to_string(),
                            };
                        }
                    };
                    day_diff += prev_month_last_day as i32;
                }
                CalcResult::Number(day_diff as f64)
            }
            "YM" => {
                // Calculate months ignoring years
                let mut months = end_date.month() as i32 - start_date.month() as i32;
                // Adjust if the end date's day is earlier than the start date's day
                if end_date.day() < start_date.day() {
                    months -= 1;
                }
                // Ensure the result is always positive (0-11 range)
                CalcResult::Number(months.rem_euclid(12) as f64)
            }
            "YD" => {
                // Calculate days ignoring years
                // Find the date corresponding to the start month/day but in the end year (or year before)
                let target_year = if start_date.month() > end_date.month()
                    || (start_date.month() == end_date.month() && start_date.day() > end_date.day())
                {
                    end_date.year() - 1 // Use previous year if start date month/day hasn't occurred yet in end year
                } else {
                    end_date.year()
                };

                let start_date_in_target_year = match chrono::NaiveDate::from_ymd_opt(
                    target_year,
                    start_date.month(),
                    start_date.day(),
                ) {
                    Some(d) => d,
                    None => {
                        // Handle Feb 29 in non-leap year for target_year
                        if start_date.month() == 2 && start_date.day() == 29 {
                            match chrono::NaiveDate::from_ymd_opt(target_year, 2, 28) {
                                Some(d) => d,
                                None => {
                                    return CalcResult::Error {
                                        error: Error::VALUE,
                                        origin: cell,
                                        message: "Internal error adjusting Feb 29 in DATEDIF YD"
                                            .to_string(),
                                    }
                                }
                            }
                        } else {
                            return CalcResult::Error {
                                error: Error::VALUE,
                                origin: cell,
                                message: "Internal error creating date in DATEDIF YD".to_string(),
                            };
                        }
                    }
                };

                let days = end_date
                    .signed_duration_since(start_date_in_target_year)
                    .num_days();
                CalcResult::Number(days as f64)
            }
            _ => CalcResult::Error {
                error: Error::NUM, // Excel returns #NUM! for invalid unit
                origin: cell,
                message: format!("Function DATEDIF has invalid unit: {}", unit),
            },
        }
    }
}
