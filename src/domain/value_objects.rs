use serde::{Deserialize, Serialize};

/// Email value object - ensures email validity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, &'static str> {
        if email.contains('@') && email.contains('.') {
            Ok(Self(email))
        } else {
            Err("Invalid email format")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Grade value object - represents FSRS grading (0-5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grade(u8);

impl Grade {
    pub fn new(value: u8) -> Result<Self, &'static str> {
        if value <= 5 {
            Ok(Self(value))
        } else {
            Err("Grade must be between 0 and 5")
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(Email::new("user@example.com".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        assert!(Email::new("invalid".to_string()).is_err());
    }

    #[test]
    fn test_valid_grade() {
        assert!(Grade::new(5).is_ok());
        assert!(Grade::new(0).is_ok());
    }

    #[test]
    fn test_invalid_grade() {
        assert!(Grade::new(6).is_err());
    }
}
