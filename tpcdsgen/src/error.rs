#[derive(Debug, Clone, PartialEq)]
pub struct TpcdsError {
    message: String,
}

impl TpcdsError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for TpcdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TpcdsError {}

pub type Result<T> = std::result::Result<T, TpcdsError>;

/// Specific error for invalid command-line options
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidOptionError {
    option_name: String,
    value: String,
    message: String,
}

impl InvalidOptionError {
    pub fn new(option_name: &str, value: &str) -> Self {
        Self::with_message(option_name, value, "")
    }

    pub fn with_message(option_name: &str, value: &str, message: &str) -> Self {
        Self {
            option_name: option_name.to_string(),
            value: value.to_string(),
            message: message.to_string(),
        }
    }

    pub fn option_name(&self) -> &str {
        &self.option_name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn custom_message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for InvalidOptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid value for {}: '{}'. {}",
            self.option_name, self.value, self.message
        )
    }
}

impl std::error::Error for InvalidOptionError {}

impl From<InvalidOptionError> for TpcdsError {
    fn from(err: InvalidOptionError) -> Self {
        TpcdsError::new(&err.to_string())
    }
}

// Utility macros for argument validation (similar to Java's checkArgument)
#[macro_export]
macro_rules! check_argument {
    ($condition:expr, $message:expr) => {
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !$condition {
            return Err(TpcdsError::new($message));
        }
    };
}

#[macro_export]
macro_rules! check_state {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(TpcdsError::new($message));
        }
    };
}
