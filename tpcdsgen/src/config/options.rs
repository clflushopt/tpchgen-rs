use crate::config::{Session, Table};
use crate::error::{InvalidOptionError, Result};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "tpcdsgen")]
#[command(about = "Rust implementation of TPC-DS data generator")]
pub struct Options {
    /// Volume of data to generate in GB (Default: 1)
    #[arg(long = "scale", short = 's', default_value = "1")]
    pub scale: f64,

    /// Directory to put generated files (Default: .)
    #[arg(long = "directory", short = 'd', default_value = ".")]
    pub directory: String,

    /// Suffix for generated data files (Default: .dat)
    #[arg(long = "suffix", default_value = ".dat")]
    pub suffix: String,

    /// Build only the specified table. If not specified, all tables will be generated
    #[arg(long = "table", short = 't')]
    pub table: Option<String>,

    /// String representation for null values (Default: the empty string)
    #[arg(long = "null", default_value = "")]
    pub null_string: String,

    /// Separator between columns (Default: |)
    #[arg(long = "separator", default_value = "|")]
    pub separator: String,

    /// Do not terminate each row with a separator (Default: false)
    #[arg(long = "do-not-terminate")]
    pub do_not_terminate: bool,

    /// Use gender-neutral manager names.
    /// This diverges from C implementation but is supported by the Java one (i need to check the latest spec)
    #[arg(long = "no-sexism")]
    pub no_sexism: bool,

    /// Build data in `n` separate chunks (Default: 1)
    #[arg(long = "parallelism", default_value = "1")]
    pub parallelism: i32,

    /// Overwrite existing data files for tables
    #[arg(long = "overwrite")]
    pub overwrite: bool,
}

impl Options {
    // Default constants (matching Java implementation)
    pub const DEFAULT_SCALE: f64 = 1.0;
    pub const DEFAULT_DIRECTORY: &'static str = ".";
    pub const DEFAULT_SUFFIX: &'static str = ".dat";
    pub const DEFAULT_NULL_STRING: &'static str = "";
    pub const DEFAULT_SEPARATOR: char = '|';
    pub const DEFAULT_DO_NOT_TERMINATE: bool = false;
    pub const DEFAULT_NO_SEXISM: bool = false;
    pub const DEFAULT_PARALLELISM: i32 = 1;
    pub const DEFAULT_OVERWRITE: bool = false;

    pub fn new() -> Self {
        Self {
            scale: Self::DEFAULT_SCALE,
            directory: Self::DEFAULT_DIRECTORY.to_string(),
            suffix: Self::DEFAULT_SUFFIX.to_string(),
            table: None,
            null_string: Self::DEFAULT_NULL_STRING.to_string(),
            separator: Self::DEFAULT_SEPARATOR.to_string(),
            do_not_terminate: Self::DEFAULT_DO_NOT_TERMINATE,
            no_sexism: Self::DEFAULT_NO_SEXISM,
            parallelism: Self::DEFAULT_PARALLELISM,
            overwrite: Self::DEFAULT_OVERWRITE,
        }
    }

    /// Convert Options to Session, performing validation
    pub fn to_session(&self) -> Result<Session> {
        self.validate_properties()?;

        let table_option = if let Some(table_str) = &self.table {
            Some(self.parse_table(table_str)?)
        } else {
            None
        };

        // Parse separator (should be single character)
        let separator_char = if self.separator.len() == 1 {
            self.separator.chars().next().unwrap()
        } else {
            return Err(InvalidOptionError::with_message(
                "separator",
                &self.separator,
                "Separator must be a single character",
            )
            .into());
        };

        Ok(Session::new(
            self.scale,
            self.directory.clone(),
            self.suffix.clone(),
            table_option,
            self.null_string.clone(),
            separator_char,
            self.do_not_terminate,
            self.no_sexism,
            self.parallelism,
            self.overwrite,
        ))
    }

    /// Parse table name to Table enum (case-insensitive)
    fn parse_table(&self, table_str: &str) -> Result<Table> {
        table_str
            .parse::<Table>()
            .map_err(|_| InvalidOptionError::new("table", table_str).into())
    }

    /// Validate all properties (matching Java validation rules)
    fn validate_properties(&self) -> Result<()> {
        // Scale validation
        if self.scale < 0.0 || self.scale > 100000.0 {
            return Err(InvalidOptionError::with_message(
                "scale",
                &self.scale.to_string(),
                "Scale must be greater than 0 and less than 100000",
            )
            .into());
        }

        // Directory validation
        if self.directory.is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                &self.directory,
                "Directory cannot be an empty string",
            )
            .into());
        }

        // Suffix validation
        if self.suffix.is_empty() {
            return Err(InvalidOptionError::with_message(
                "suffix",
                &self.suffix,
                "Suffix cannot be an empty string",
            )
            .into());
        }

        // Parallelism validation
        if self.parallelism < 1 {
            return Err(InvalidOptionError::with_message(
                "parallelism",
                &self.parallelism.to_string(),
                "Parallelism must be >= 1",
            )
            .into());
        }

        // Separator validation
        if self.separator.len() != 1 {
            return Err(InvalidOptionError::with_message(
                "separator",
                &self.separator,
                "Separator must be a single character",
            )
            .into());
        }

        Ok(())
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_defaults() {
        let options = Options::new();
        assert_eq!(options.scale, 1.0);
        assert_eq!(options.directory, ".");
        assert_eq!(options.suffix, ".dat");
        assert_eq!(options.table, None);
        assert_eq!(options.null_string, "");
        assert_eq!(options.separator, "|");
        assert!(!options.do_not_terminate);
        assert!(!options.no_sexism);
        assert_eq!(options.parallelism, 1);
        assert!(!options.overwrite);
    }

    #[test]
    fn test_valid_options_to_session() {
        let options = Options::new();
        let session = options.to_session().unwrap();
        assert_eq!(session.get_scaling().get_scale(), 1.0);
        assert_eq!(session.get_target_directory(), ".");
        assert_eq!(session.get_suffix(), ".dat");
        assert!(!session.generate_only_one_table());
    }

    #[test]
    fn test_table_parsing() {
        let mut options = Options::new();
        options.table = Some("catalog_sales".to_string());
        let session = options.to_session().unwrap();
        assert!(session.generate_only_one_table());
        assert_eq!(session.get_only_table_to_generate(), Table::CatalogSales);
    }

    #[test]
    fn test_invalid_table() {
        let mut options = Options::new();
        options.table = Some("invalid_table".to_string());
        assert!(options.to_session().is_err());
    }

    #[test]
    fn test_scale_validation() {
        let mut options = Options::new();

        // Valid scale
        options.scale = 10.0;
        assert!(options.validate_properties().is_ok());

        // Invalid scale - too large
        options.scale = 200000.0;
        assert!(options.validate_properties().is_err());

        // Invalid scale - negative
        options.scale = -1.0;
        assert!(options.validate_properties().is_err());
    }

    #[test]
    fn test_directory_validation() {
        let mut options = Options::new();

        // Valid directory
        options.directory = "/tmp".to_string();
        assert!(options.validate_properties().is_ok());

        // Invalid directory - empty
        options.directory = "".to_string();
        assert!(options.validate_properties().is_err());
    }

    #[test]
    fn test_suffix_validation() {
        let mut options = Options::new();

        // Valid suffix
        options.suffix = ".csv".to_string();
        assert!(options.validate_properties().is_ok());

        // Invalid suffix - empty
        options.suffix = "".to_string();
        assert!(options.validate_properties().is_err());
    }

    #[test]
    fn test_parallelism_validation() {
        let mut options = Options::new();

        // Valid parallelism
        options.parallelism = 4;
        assert!(options.validate_properties().is_ok());

        // Invalid parallelism - too small
        options.parallelism = 0;
        assert!(options.validate_properties().is_err());
    }

    #[test]
    fn test_separator_validation() {
        let mut options = Options::new();

        // Valid separator
        options.separator = ",".to_string();
        assert!(options.validate_properties().is_ok());

        // Invalid separator - too long
        options.separator = "||".to_string();
        assert!(options.validate_properties().is_err());

        // Invalid separator - empty
        options.separator = "".to_string();
        assert!(options.validate_properties().is_err());
    }
}
