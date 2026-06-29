use crate::config::{CompatMode, Session, Table};
use crate::error::{InvalidOptionError, Result, TpcdsError};
use std::fmt;

/// TPC-DS generation options.
///
/// Use [`Options::to_session`] to
/// convert these values into a validated [`Session`] for row generation.
#[derive(Debug, Clone)]
pub struct Options {
    /// Volume of data to generate in GB (Default: 1)
    pub scale: f64,

    /// Directory to put generated files (Default: .)
    pub directory: String,

    /// Suffix for generated data files (Default: .dat)
    pub suffix: String,

    /// Build only the specified table. If not specified, all tables will be generated
    pub table: Option<String>,

    /// String representation for null values (Default: the empty string)
    pub null_string: String,

    /// Separator between columns (Default: |)
    pub separator: String,

    /// Do not terminate each row with a separator (Default: false)
    pub do_not_terminate: bool,

    /// Use gender-neutral manager names.
    /// This diverges from C implementation but is supported by the Java one (i need to check the latest spec)
    pub no_sexism: bool,

    /// Build data in `n` separate chunks (Default: 1)
    pub parallelism: i32,

    /// Overwrite existing data files for tables
    pub overwrite: bool,

    /// Reference implementation to match (Default: trino)
    ///
    /// 'trino' produces byte-for-byte output compatible with the Trino Java library.
    /// 'c' corrects known divergences in the Java port to match the original C dsdgen.
    pub compat: CompatMode,
}

impl Options {
    pub const DEFAULT_SCALE: f64 = 1.0;
    pub const DEFAULT_DIRECTORY: &'static str = ".";
    pub const DEFAULT_SUFFIX: &'static str = ".dat";
    pub const DEFAULT_NULL_STRING: &'static str = "";
    pub const DEFAULT_SEPARATOR: char = '|';
    pub const DEFAULT_DO_NOT_TERMINATE: bool = false;
    pub const DEFAULT_NO_SEXISM: bool = false;
    pub const DEFAULT_PARALLELISM: i32 = 1;
    pub const DEFAULT_OVERWRITE: bool = false;
    pub const DEFAULT_COMPAT: CompatMode = CompatMode::Trino;

    /// Create options using the generator defaults.
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
            compat: Self::DEFAULT_COMPAT,
        }
    }

    /// Convert options into a validated [`Session`].
    pub fn to_session(&self) -> Result<Session> {
        Session::try_from(self.clone())
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

impl TryFrom<Options> for Session {
    type Error = TpcdsError;

    fn try_from(options: Options) -> Result<Self> {
        options.validate_properties()?;

        let table_option = if let Some(table_str) = &options.table {
            Some(options.parse_table(table_str)?)
        } else {
            None
        };

        // Parse separator (should be single character)
        let separator_char = if options.separator.len() == 1 {
            options.separator.chars().next().unwrap()
        } else {
            return Err(InvalidOptionError::with_message(
                "separator",
                &options.separator,
                "Separator must be a single character",
            )
            .into());
        };

        Ok(Session::new(
            options.scale,
            options.directory,
            options.suffix,
            table_option,
            options.null_string,
            separator_char,
            options.do_not_terminate,
            options.no_sexism,
            options.parallelism,
            options.overwrite,
            options.compat,
        ))
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = Vec::new();
        let default = Options::new();

        if self.scale != default.scale {
            output.push(format!("--scale {}", self.scale));
        }
        if self.directory != default.directory {
            output.push(format!("--directory {}", self.directory));
        }
        if self.suffix != default.suffix {
            output.push(format!("--suffix {}", self.suffix));
        }
        if let Some(table) = &self.table {
            output.push(format!("--table {}", table));
        }
        if self.null_string != default.null_string {
            output.push(format!("--null {}", self.null_string));
        }
        if self.separator != default.separator {
            output.push(format!("--separator {}", self.separator));
        }
        if self.do_not_terminate != default.do_not_terminate {
            output.push("--do-not-terminate".to_string());
        }
        if self.no_sexism != default.no_sexism {
            output.push("--no-sexism".to_string());
        }
        if self.parallelism != default.parallelism {
            output.push(format!("--parallelism {}", self.parallelism));
        }
        if self.overwrite != default.overwrite {
            output.push("--overwrite".to_string());
        }
        if self.compat != default.compat {
            output.push(format!(
                "--compat {}",
                match self.compat {
                    CompatMode::Trino => "trino",
                    CompatMode::C => "c",
                }
            ));
        }

        write!(f, "{}", output.join(" "))
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
    fn test_try_from_options_to_session() {
        let mut options = Options::new();
        options.scale = 2.0;
        options.table = Some("catalog_sales".to_string());

        let session = Session::try_from(options).unwrap();

        assert_eq!(session.get_scaling().get_scale(), 2.0);
        assert_eq!(session.get_table(), Some(Table::CatalogSales));
    }

    #[test]
    fn test_display_defaults() {
        let options = Options::new();
        assert_eq!(options.to_string(), "");
    }

    #[test]
    fn test_display_non_defaults() {
        let options = Options {
            scale: 2.0,
            directory: "/tmp".to_string(),
            suffix: ".csv".to_string(),
            table: Some("catalog_sales".to_string()),
            null_string: "NULL".to_string(),
            separator: ",".to_string(),
            do_not_terminate: true,
            no_sexism: true,
            parallelism: 4,
            overwrite: true,
            compat: CompatMode::C,
        };

        let expected = "--scale 2 --directory /tmp --suffix .csv --table catalog_sales --null NULL --separator , --do-not-terminate --no-sexism --parallelism 4 --overwrite --compat c";
        assert_eq!(options.to_string(), expected);
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
