use std::cmp::Ordering;
use std::cmp::Ord;

use pyo3::prelude::*;
use pyo3::types::PyDateAccess;
use pyo3::types::PyDateTime;
use pyo3::types::PyTimeAccess;

/// Extract the timezone from a Python datetime using the `tzinfo` attribute.
pub fn extract_timezone<'py>(py: Python<'py>, datetime: &'py PyDateTime) -> PyResult<Option<fastobo::ast::IsoTimezone>> {
    use fastobo::ast::IsoTimezone::*;
    let tzinfo = datetime.to_object(py).getattr(py, "tzinfo")?;
    if !tzinfo.is_none() {
        let timedelta = tzinfo.call_method1(py, "utcoffset", (datetime,))?;
        let total_seconds = timedelta.call_method0(py, "total_seconds")?.extract::<f64>(py)? as i64;
        let hh = total_seconds / 3600;
        let mm = (total_seconds / 60) % 60;
        match total_seconds.cmp(&0) {
            Ordering::Equal => Ok(Some(Utc)),
            Ordering::Less => Ok(Some(Minus((-hh) as u8, Some(((mm + 60) % 60) as u8)))),
            Ordering::Greater => Ok(Some(Plus(hh as u8, Some(mm as u8)))),
        }
    } else {
        Ok(None)
    }
}

/// Convert a Python `datetime.datetime` to a `fastobo::ast::IsoDateTime`.
pub fn datetime_to_isodate<'py>(py: Python<'py>, datetime: &'py PyDateTime) -> PyResult<fastobo::ast::IsoDateTime> {
    let mut dt = fastobo::ast::IsoDateTime::new(
        datetime.get_day(),
        datetime.get_month(),
        datetime.get_year() as u16,
        datetime.get_hour(),
        datetime.get_minute(),
        datetime.get_second()
    );

    if let Some(timezone) = extract_timezone(py, datetime)? {
        dt = dt.with_timezone(timezone);
    }

    Ok(dt)
}

/// Convert a `fastobo::ast::IsoDateTime` to a Python `datetime.datetime`.
pub fn isodate_to_datetime<'py>(py: Python<'py>, datetime: &fastobo::ast::IsoDateTime) -> PyResult<&'py PyDateTime> {
    use fastobo::ast::IsoTimezone::*;

    // Extract the timezone if there is any
    let tz = if let Some(tz) = datetime.timezone() {
        let datetime = py.import("datetime")?;
        let timezone = datetime.get("timezone")?.to_object(py);
        let timedelta = datetime.get("timedelta")?.to_object(py);
        match tz {
            Utc => Some(timezone.getattr(py, "utc")?),
            Plus(hh, mm) => {
                let args = (0u8, 0u8, 0u8, 0u8, mm.unwrap_or(0), *hh);
                Some(timezone.call1(py, (timedelta.call1(py, args)?,))?)
            }
            Minus(hh, mm) => {
                let args = (0u8, 0u8, 0u8, 0u8, -(mm.unwrap_or(0) as i8), -(*hh as i8));
                Some(timezone.call1(py, (timedelta.call1(py, args)?,))?)
            }
        }
    } else {
        None
    };

    // Create the `datetime.datetime` instance
    PyDateTime::new(
        py,
        datetime.year() as i32,
        datetime.month(),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second(),
        datetime.fraction().map(|f| (f*1000.0) as u32).unwrap_or(0),
        tz.as_ref(),
    )
}
