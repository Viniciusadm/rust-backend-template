use chrono::NaiveDateTime;

use crate::domain::{DomainId, Email, NonEmptyString, UtcDateTime};
use crate::errors::AppError;

pub fn id(value: String) -> Result<DomainId, AppError> {
    Ok(DomainId::new(value)?)
}

pub fn non_empty(value: String, field: &'static str) -> Result<NonEmptyString, AppError> {
    Ok(NonEmptyString::new(value, field)?)
}

pub fn email(value: String) -> Result<Email, AppError> {
    Ok(Email::new(value)?)
}

pub fn datetime(value: NaiveDateTime) -> Result<UtcDateTime, AppError> {
    Ok(UtcDateTime::new_iso8601(value.to_string())?)
}

pub fn opt_datetime(value: Option<NaiveDateTime>) -> Result<Option<UtcDateTime>, AppError> {
    value.map(datetime).transpose()
}
