use std::cmp::Ord;
use std::cmp::Ordering;

use fastobo::ast::Date;
use fastobo::ast::Time;

use pyo3::prelude::*;
use pyo3::types::PyDate;
use pyo3::types::PyDateAccess;
use pyo3::types::PyDateTime;
use pyo3::types::PyTimeAccess;
use pyo3::types::PyTzInfo;

/// Extract the timezone from a Python datetime using the `tzinfo` attribute.
pub fn extract_timezone<'py>(
    py: Python<'py>,
    datetime: &Bound<'py, PyDateTime>,
) -> PyResult<Option<fastobo::ast::IsoTimezone>> {
    use fastobo::ast::IsoTimezone::*;
    let tzinfo = datetime.getattr("tzinfo")?;
    if !tzinfo.is_none() {
        let timedelta = tzinfo.call_method1("utcoffset", (datetime,))?;
        let total_seconds = timedelta.call_method0("total_seconds")?.extract::<f64>()? as i64;
        let hh = total_seconds / 3600;
        let mm = (total_seconds / 60) % 60;
        match total_seconds.cmp(&0) {
            Ordering::Equal => Ok(Some(Utc)),
            Ordering::Less => Ok(Some(Minus((-hh) as u8, ((mm + 60) % 60) as u8))),
            Ordering::Greater => Ok(Some(Plus(hh as u8, mm as u8))),
        }
    } else {
        Ok(None)
    }
}

/// Convert a Python `datetime.datetime` to a `fastobo::ast::IsoDateTime`.
pub fn datetime_to_isodatetime<'py>(
    py: Python<'py>,
    datetime: &Bound<'py, PyDateTime>,
) -> PyResult<fastobo::ast::IsoDateTime> {
    let date = fastobo::ast::IsoDate::new(
        datetime.get_year() as u16,
        datetime.get_month(),
        datetime.get_day(),
    );
    let mut time = fastobo::ast::IsoTime::new(
        datetime.get_hour(),
        datetime.get_minute(),
        datetime.get_second(),
    );
    if let Some(timezone) = extract_timezone(py, datetime)? {
        time = time.with_timezone(timezone);
    }
    Ok(fastobo::ast::IsoDateTime::new(date, time))
}

/// Convert a `fastobo::ast::IsoDateTime` to a Python `datetime.datetime`.
pub fn isodatetime_to_datetime<'py>(
    py: Python<'py>,
    datetime: &fastobo::ast::IsoDateTime,
) -> PyResult<Bound<'py, PyDateTime>> {
    use fastobo::ast::IsoTimezone::*;

    // Extract the timezone if there is any
    let tz = if let Some(tz) = datetime.time().timezone() {
        let datetime = py.import("datetime")?;
        let timezone = datetime.getattr("timezone")?;
        let timedelta = datetime.getattr("timedelta")?;
        match tz {
            Utc => Some(timezone.getattr("utc")?),
            Plus(hh, mm) => {
                let args = (0u8, 0u8, 0u8, 0u8, *mm, *hh);
                Some(timezone.call1((timedelta.call1(args)?,))?)
            }
            Minus(hh, mm) => {
                let args = (0u8, 0u8, 0u8, 0u8, -(*mm as i8), -(*hh as i8));
                Some(timezone.call1((timedelta.call1(args)?,))?)
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
        datetime
            .time()
            .fraction()
            .map(|f| (f * 1000.0) as u32)
            .unwrap_or(0),
        tz.as_ref()
            .map(|obj| obj.downcast::<PyTzInfo>())
            .transpose()?,
    )
}

/// Convert a Python `datetime.date` to a `fastobo::ast::IsoDate`.
pub fn date_to_isodate<'py>(
    py: Python<'py>,
    date: &Bound<'py, PyDate>,
) -> PyResult<fastobo::ast::IsoDate> {
    Ok(fastobo::ast::IsoDate::new(
        date.get_year() as u16,
        date.get_month(),
        date.get_day(),
    ))
}

/// Convert a `fastobo::ast::IsoDateTime` to a Python `datetime.datetime`.
pub fn isodate_to_date<'py>(
    py: Python<'py>,
    date: &fastobo::ast::IsoDate,
) -> PyResult<Bound<'py, PyDate>> {
    // Create the `datetime.datetime` instance
    PyDate::new(py, date.year() as i32, date.month(), date.day())
}
