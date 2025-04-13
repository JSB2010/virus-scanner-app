// Error handling
#[derive(Debug, Clone)]
pub enum ErrorCategory {
    ApiError,
    FileSystemError,
    NetworkError,
    ConfigurationError,
    UnknownError,
}

#[derive(Debug, Clone)]
pub struct ErrorHandler {
    pub last_error: Option<String>,
    pub error_category: Option<ErrorCategory>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        ErrorHandler {
            last_error: None,
            error_category: None,
        }
    }

    pub fn set_error(&mut self, error: String, category: ErrorCategory) {
        self.last_error = Some(error);
        self.error_category = Some(category);
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
        self.error_category = None;
    }
}
