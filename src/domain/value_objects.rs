#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainId(String);

impl DomainId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainValidationError::Empty("id"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(
        value: impl Into<String>,
        field: &'static str,
    ) -> Result<Self, DomainValidationError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(DomainValidationError::Empty(field));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into().trim().to_ascii_lowercase();
        if !value.contains('@') || value.starts_with('@') || value.ends_with('@') {
            return Err(DomainValidationError::Invalid("email"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtcDateTime(String);

impl UtcDateTime {
    pub fn new_iso8601(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainValidationError::Empty("utc_datetime"));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainValidationError {
    Empty(&'static str),
    Invalid(&'static str),
}
