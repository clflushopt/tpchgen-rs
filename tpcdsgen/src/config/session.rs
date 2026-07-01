use crate::config::{CompatMode, Scaling, Table};
use crate::error::{InvalidOptionError, Result};

/// Configuration for a TPC-DS data generation run.
///
/// A `Session` defines how TPC-DS data is generated.
#[derive(Debug, Clone)]
pub struct Session {
    scaling: Scaling,
    target_directory: String,
    suffix: String,
    table: Option<Table>,
    null_string: String,
    separator: char,
    do_not_terminate: bool,
    no_sexism: bool,
    parallelism: i32,
    chunk_number: i32,
    overwrite: bool,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            scaling: Scaling::new_with_compat(Self::DEFAULT_SCALE, Self::DEFAULT_COMPAT),
            target_directory: Self::DEFAULT_DIRECTORY.to_string(),
            suffix: Self::DEFAULT_SUFFIX.to_string(),
            table: None,
            null_string: Self::DEFAULT_NULL_STRING.to_string(),
            separator: Self::DEFAULT_SEPARATOR,
            do_not_terminate: Self::DEFAULT_DO_NOT_TERMINATE,
            no_sexism: Self::DEFAULT_NO_SEXISM,
            parallelism: Self::DEFAULT_PARALLELISM,
            chunk_number: 1,
            overwrite: Self::DEFAULT_OVERWRITE,
            compat_mode: Self::DEFAULT_COMPAT,
            command_line_arguments: None,
        }
    }
}

impl Session {
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

    /// Convert this session into a builder initialized with its current values.
    pub fn into_builder(self) -> SessionBuilder {
        SessionBuilder {
            scale: self.scaling.get_scale(),
            target_directory: self.target_directory,
            suffix: self.suffix,
            table: self.table,
            null_string: self.null_string,
            separator: self.separator,
            do_not_terminate: self.do_not_terminate,
            no_sexism: self.no_sexism,
            parallelism: self.parallelism,
            chunk_number: self.chunk_number,
            overwrite: self.overwrite,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        }
    }

    /// Return the [`Scaling`] settings used for row counts.
    pub fn get_scaling(&self) -> &Scaling {
        &self.scaling
    }

    /// Return the directory where generated files are written.
    pub fn get_target_directory(&self) -> &str {
        &self.target_directory
    }

    /// Return the suffix appended to generated data file names.
    pub fn get_suffix(&self) -> &str {
        &self.suffix
    }

    /// Return `true` if this session should generate a single table.
    pub fn generate_only_one_table(&self) -> bool {
        self.table.is_some()
    }

    /// Return the single table selected for generation.
    ///
    /// # Panics
    ///
    /// Panics if no single table was configured. Call
    /// [`Session::generate_only_one_table`] before using this method.
    pub fn get_only_table_to_generate(&self) -> Table {
        self.table
            .unwrap_or_else(|| panic!("table not present - call generate_only_one_table() first"))
    }

    /// Return the optional single-table selection.
    pub fn get_table(&self) -> Option<Table> {
        self.table
    }

    /// Return the string emitted for null values.
    pub fn get_null_string(&self) -> &str {
        &self.null_string
    }

    /// Return the column separator emitted between fields.
    pub fn get_separator(&self) -> char {
        self.separator
    }

    /// Return whether rows should end with the configured column separator.
    pub fn terminate_rows_with_separator(&self) -> bool {
        !self.do_not_terminate
    }

    /// Return whether generated manager names should match the reference
    /// implementation's original gendered data.
    pub fn is_sexist(&self) -> bool {
        !self.no_sexism
    }

    /// Return the total number of chunks requested for generation.
    pub fn get_parallelism(&self) -> i32 {
        self.parallelism
    }

    /// Return the one-based chunk number represented by this session.
    pub fn get_chunk_number(&self) -> i32 {
        self.chunk_number
    }

    /// Return whether existing output files may be overwritten.
    pub fn should_overwrite(&self) -> bool {
        self.overwrite
    }

    /// Return the reference implementation compatibility mode.
    pub fn get_compat_mode(&self) -> CompatMode {
        self.compat_mode
    }

    /// Return the actual command line arguments used to create this session, if known.
    pub fn command_line_arguments(&self) -> Option<&str> {
        self.command_line_arguments.as_deref()
    }
}

/// Builder for validated [`Session`] construction.
#[derive(Debug, Clone)]
pub struct SessionBuilder {
    scale: f64,
    target_directory: String,
    suffix: String,
    table: Option<Table>,
    null_string: String,
    separator: char,
    do_not_terminate: bool,
    no_sexism: bool,
    parallelism: i32,
    chunk_number: i32,
    overwrite: bool,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self {
            scale: Session::DEFAULT_SCALE,
            target_directory: Session::DEFAULT_DIRECTORY.to_string(),
            suffix: Session::DEFAULT_SUFFIX.to_string(),
            table: None,
            null_string: Session::DEFAULT_NULL_STRING.to_string(),
            separator: Session::DEFAULT_SEPARATOR,
            do_not_terminate: Session::DEFAULT_DO_NOT_TERMINATE,
            no_sexism: Session::DEFAULT_NO_SEXISM,
            parallelism: Session::DEFAULT_PARALLELISM,
            chunk_number: 1,
            overwrite: Session::DEFAULT_OVERWRITE,
            compat_mode: Session::DEFAULT_COMPAT,
            command_line_arguments: None,
        }
    }
}

impl SessionBuilder {
    /// Create a builder initialized with the default session values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the scale factor to generate.
    pub fn with_scale_factor(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Set the directory where generated files are written.
    pub fn with_target_directory(mut self, target_directory: impl Into<String>) -> Self {
        self.target_directory = target_directory.into();
        self
    }

    /// Set the suffix appended to generated data file names.
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Restrict generation to a single table.
    pub fn with_table(mut self, table: Table) -> Self {
        self.table = Some(table);
        self
    }

    /// Clear any single-table restriction.
    pub fn without_table(mut self) -> Self {
        self.table = None;
        self
    }

    /// Set the string emitted for null values.
    pub fn with_null_string(mut self, null_string: impl Into<String>) -> Self {
        self.null_string = null_string.into();
        self
    }

    /// Set the column separator emitted between fields.
    pub fn with_separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Set whether generated rows should omit the trailing separator.
    pub fn with_do_not_terminate(mut self, do_not_terminate: bool) -> Self {
        self.do_not_terminate = do_not_terminate;
        self
    }

    /// Set whether gender-neutral manager names are enabled.
    pub fn with_no_sexism(mut self, no_sexism: bool) -> Self {
        self.no_sexism = no_sexism;
        self
    }

    /// Set the total number of chunks to generate.
    pub fn with_parallelism(mut self, parallelism: i32) -> Self {
        self.parallelism = parallelism;
        self
    }

    /// Set the one-based chunk number represented by this session.
    pub fn with_chunk_number(mut self, chunk_number: i32) -> Self {
        self.chunk_number = chunk_number;
        self
    }

    /// Set whether existing output files may be overwritten.
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// Set the reference implementation compatibility mode.
    pub fn with_compat_mode(mut self, compat_mode: CompatMode) -> Self {
        self.compat_mode = compat_mode;
        self
    }

    /// Set the actual command line arguments used to create the session.
    pub fn with_command_line_arguments(
        mut self,
        command_line_arguments: impl Into<String>,
    ) -> Self {
        self.command_line_arguments = Some(command_line_arguments.into());
        self
    }

    /// Clear any command line arguments associated with the session.
    pub fn without_command_line_arguments(mut self) -> Self {
        self.command_line_arguments = None;
        self
    }

    /// Build a validated [`Session`].
    pub fn build(self) -> Result<Session> {
        self.validate()?;

        Ok(Session {
            scaling: Scaling::new_with_compat(self.scale, self.compat_mode),
            target_directory: self.target_directory,
            suffix: self.suffix,
            table: self.table,
            null_string: self.null_string,
            separator: self.separator,
            do_not_terminate: self.do_not_terminate,
            no_sexism: self.no_sexism,
            parallelism: self.parallelism,
            chunk_number: self.chunk_number,
            overwrite: self.overwrite,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        })
    }

    fn validate(&self) -> Result<()> {
        if !(0.0..=100000.0).contains(&self.scale) {
            return Err(InvalidOptionError::with_message(
                "scale",
                &self.scale.to_string(),
                "Scale must be greater than 0 and less than 100000",
            )
            .into());
        }

        if self.target_directory.is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                &self.target_directory,
                "Directory cannot be an empty string",
            )
            .into());
        }

        if self.suffix.is_empty() {
            return Err(InvalidOptionError::with_message(
                "suffix",
                &self.suffix,
                "Suffix cannot be an empty string",
            )
            .into());
        }

        if self.parallelism < 1 {
            return Err(InvalidOptionError::with_message(
                "parallelism",
                &self.parallelism.to_string(),
                "Parallelism must be >= 1",
            )
            .into());
        }

        if self.chunk_number < 1 {
            return Err(InvalidOptionError::with_message(
                "chunk_number",
                &self.chunk_number.to_string(),
                "Chunk number must be >= 1",
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_session() {
        let session = Session::default();

        assert_eq!(session.get_scaling().get_scale(), 1.0);
        assert_eq!(session.get_target_directory(), ".");
        assert_eq!(session.get_suffix(), ".dat");
        assert!(!session.generate_only_one_table());
        assert_eq!(session.get_null_string(), "");
        assert_eq!(session.get_separator(), '|');
        assert!(session.terminate_rows_with_separator());
        assert!(session.is_sexist());
        assert_eq!(session.get_parallelism(), 1);
        assert_eq!(session.get_chunk_number(), 1);
        assert!(!session.should_overwrite());
        assert_eq!(session.command_line_arguments(), None);
    }

    #[test]
    fn test_session_builder() {
        let session = SessionBuilder::new()
            .with_scale_factor(2.0)
            .with_target_directory("/tmp")
            .with_suffix(".csv")
            .with_table(Table::CatalogSales)
            .with_null_string("NULL")
            .with_separator(',')
            .with_do_not_terminate(true)
            .with_no_sexism(true)
            .with_parallelism(4)
            .with_chunk_number(2)
            .with_overwrite(true)
            .with_compat_mode(CompatMode::C)
            .with_command_line_arguments("tpcgen tpcds --scale-factor 2")
            .build()
            .unwrap();

        assert_eq!(session.get_scaling().get_scale(), 2.0);
        assert_eq!(session.get_target_directory(), "/tmp");
        assert_eq!(session.get_suffix(), ".csv");
        assert_eq!(session.get_table(), Some(Table::CatalogSales));
        assert_eq!(session.get_null_string(), "NULL");
        assert_eq!(session.get_separator(), ',');
        assert!(!session.terminate_rows_with_separator());
        assert!(!session.is_sexist());
        assert_eq!(session.get_parallelism(), 4);
        assert_eq!(session.get_chunk_number(), 2);
        assert!(session.should_overwrite());
        assert_eq!(session.get_compat_mode(), CompatMode::C);
        assert_eq!(
            session.command_line_arguments(),
            Some("tpcgen tpcds --scale-factor 2")
        );
    }

    #[test]
    fn test_session_builder_validation() {
        assert!(SessionBuilder::new()
            .with_scale_factor(10.0)
            .build()
            .is_ok());

        assert!(SessionBuilder::new()
            .with_scale_factor(-1.0)
            .build()
            .is_err());

        assert!(SessionBuilder::new()
            .with_scale_factor(f64::NAN)
            .build()
            .is_err());

        assert!(SessionBuilder::new()
            .with_target_directory("")
            .build()
            .is_err());

        assert!(SessionBuilder::new().with_suffix("").build().is_err());

        assert!(SessionBuilder::new().with_parallelism(0).build().is_err());

        assert!(SessionBuilder::new().with_chunk_number(0).build().is_err());
    }

    #[test]
    fn test_into_builder() {
        let session = Session::default();

        let session = session
            .into_builder()
            .with_table(Table::CatalogSales)
            .with_scale_factor(10.0)
            .with_parallelism(4)
            .with_chunk_number(2)
            .with_no_sexism(true)
            .with_command_line_arguments("initial")
            .without_command_line_arguments()
            .build()
            .unwrap();

        assert!(session.generate_only_one_table());
        assert_eq!(session.get_only_table_to_generate(), Table::CatalogSales);
        assert_eq!(session.get_scaling().get_scale(), 10.0);
        assert_eq!(session.get_parallelism(), 4);
        assert_eq!(session.get_chunk_number(), 2);
        assert!(!session.is_sexist());
        assert_eq!(session.command_line_arguments(), None);
    }

    #[test]
    fn test_generate_only_one_table() {
        let session = Session::default();
        assert!(!session.generate_only_one_table());

        let session_with_table = session
            .into_builder()
            .with_table(Table::StoreSales)
            .build()
            .unwrap();
        assert!(session_with_table.generate_only_one_table());
        assert_eq!(
            session_with_table.get_only_table_to_generate(),
            Table::StoreSales
        );
    }

    #[test]
    #[should_panic(expected = "table not present")]
    fn test_get_only_table_when_none() {
        let session = Session::default();
        session.get_only_table_to_generate();
    }

    #[test]
    fn test_boolean_accessors() {
        let session = SessionBuilder::new()
            .with_do_not_terminate(true)
            .with_no_sexism(true)
            .build()
            .unwrap();

        assert!(!session.terminate_rows_with_separator()); // negation of do_not_terminate
        assert!(!session.is_sexist()); // negation of no_sexism
    }
}
