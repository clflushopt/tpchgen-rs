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
}

impl Default for Session {
    fn default() -> Self {
        Self::new_with_chunk_number(
            Self::DEFAULT_SCALE,
            Self::DEFAULT_DIRECTORY.to_string(),
            Self::DEFAULT_SUFFIX.to_string(),
            None,
            Self::DEFAULT_NULL_STRING.to_string(),
            Self::DEFAULT_SEPARATOR,
            Self::DEFAULT_DO_NOT_TERMINATE,
            Self::DEFAULT_NO_SEXISM,
            Self::DEFAULT_PARALLELISM,
            1,
            Self::DEFAULT_OVERWRITE,
            Self::DEFAULT_COMPAT,
        )
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

    /// Create a session for a single generation chunk.
    ///
    /// This is equivalent to calling [`Session::new_with_chunk_number`] with
    /// `chunk_number` set to `1`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        scale: f64,
        target_directory: String,
        suffix: String,
        table: Option<Table>,
        null_string: String,
        separator: char,
        do_not_terminate: bool,
        no_sexism: bool,
        parallelism: i32,
        overwrite: bool,
        compat_mode: CompatMode,
    ) -> Self {
        Self::new_with_chunk_number(
            scale,
            target_directory,
            suffix,
            table,
            null_string,
            separator,
            do_not_terminate,
            no_sexism,
            parallelism,
            1, // Default chunk number
            overwrite,
            compat_mode,
        )
    }

    /// Create and validate a session for a single generation chunk.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        scale: f64,
        target_directory: String,
        suffix: String,
        table: Option<Table>,
        null_string: String,
        separator: char,
        do_not_terminate: bool,
        no_sexism: bool,
        parallelism: i32,
        overwrite: bool,
        compat_mode: CompatMode,
    ) -> Result<Self> {
        Self::validate(scale, &target_directory, &suffix, parallelism)?;

        Ok(Self::new(
            scale,
            target_directory,
            suffix,
            table,
            null_string,
            separator,
            do_not_terminate,
            no_sexism,
            parallelism,
            overwrite,
            compat_mode,
        ))
    }

    fn validate(scale: f64, target_directory: &str, suffix: &str, parallelism: i32) -> Result<()> {
        if scale < 0.0 || scale > 100000.0 {
            return Err(InvalidOptionError::with_message(
                "scale",
                &scale.to_string(),
                "Scale must be greater than 0 and less than 100000",
            )
            .into());
        }

        if target_directory.is_empty() {
            return Err(InvalidOptionError::with_message(
                "directory",
                target_directory,
                "Directory cannot be an empty string",
            )
            .into());
        }

        if suffix.is_empty() {
            return Err(InvalidOptionError::with_message(
                "suffix",
                suffix,
                "Suffix cannot be an empty string",
            )
            .into());
        }

        if parallelism < 1 {
            return Err(InvalidOptionError::with_message(
                "parallelism",
                &parallelism.to_string(),
                "Parallelism must be >= 1",
            )
            .into());
        }

        Ok(())
    }

    /// Create a session for a specific generation chunk.
    ///
    /// `parallelism` is the total number of chunks to generate, and
    /// `chunk_number` identifies the chunk represented by this session. The
    /// caller is responsible for passing values that have already been
    /// validated.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_chunk_number(
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
    ) -> Self {
        Session {
            scaling: Scaling::new_with_compat(scale, compat_mode),
            target_directory,
            suffix,
            table,
            null_string,
            separator,
            do_not_terminate,
            no_sexism,
            parallelism,
            chunk_number,
            overwrite,
            compat_mode,
        }
    }

    /// Return a copy of this session that only generates `table`.
    pub fn with_table(&self, table: Table) -> Self {
        Session {
            table: Some(table),
            ..self.clone()
        }
    }

    /// Return a copy of this session with a different scale factor.
    ///
    /// The existing compatibility mode is preserved when rebuilding the
    /// [`Scaling`] configuration.
    pub fn with_scale(&self, scale: f64) -> Self {
        Session {
            scaling: Scaling::new_with_compat(scale, self.compat_mode),
            ..self.clone()
        }
    }

    /// Return a copy of this session with a different [`CompatMode`].
    ///
    /// The scale factor is preserved and the [`Scaling`] configuration is
    /// rebuilt so row-count calculations use the new mode.
    pub fn with_compat_mode(&self, compat_mode: CompatMode) -> Self {
        Session {
            scaling: Scaling::new_with_compat(self.scaling.get_scale(), compat_mode),
            compat_mode,
            ..self.clone()
        }
    }

    /// Return a copy of this session with a different total chunk count.
    pub fn with_parallelism(&self, parallelism: i32) -> Self {
        Session {
            parallelism,
            ..self.clone()
        }
    }

    /// Return a copy of this session for a different chunk number.
    pub fn with_chunk_number(&self, chunk_number: i32) -> Self {
        Session {
            chunk_number,
            ..self.clone()
        }
    }

    /// Return a copy of this session with gender-neutral manager names enabled
    /// or disabled.
    ///
    /// When enabled, first names are picked from the general first-name
    /// distribution instead of the male-only distribution used by the C
    /// reference implementation. This matches the behavior documented by the
    /// Trino Java implementation's [`--no-sexism`] option.
    ///
    /// [`--no-sexism`]: https://github.com/trinodb/tpcds/blob/8a02abbba864feedc2afd078c8153d66a95bb2d4/src/main/java/io/trino/tpcds/Options.java#L55-L60
    pub fn with_no_sexism(&self, no_sexism: bool) -> Self {
        Session {
            no_sexism,
            ..self.clone()
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

    /// Reconstruct command line arguments that would produce this session.
    ///
    /// Default-valued options are omitted from the returned string.
    pub fn get_command_line_arguments(&self) -> String {
        let mut output = Vec::new();

        if self.scaling.get_scale() != Self::DEFAULT_SCALE {
            output.push(format!("--scale {}", self.scaling.get_scale()));
        }
        if self.target_directory != Self::DEFAULT_DIRECTORY {
            output.push(format!("--directory {}", self.target_directory));
        }
        if self.suffix != Self::DEFAULT_SUFFIX {
            output.push(format!("--suffix {}", self.suffix));
        }
        if let Some(table) = self.table {
            output.push(format!("--table {}", table.get_name()));
        }
        if self.null_string != Self::DEFAULT_NULL_STRING {
            output.push(format!("--null {}", self.null_string));
        }
        if self.separator != Self::DEFAULT_SEPARATOR {
            output.push(format!("--separator {}", self.separator));
        }
        if self.do_not_terminate != Self::DEFAULT_DO_NOT_TERMINATE {
            output.push("--do-not-terminate".to_string());
        }
        if self.no_sexism != Self::DEFAULT_NO_SEXISM {
            output.push("--no-sexism".to_string());
        }
        if self.parallelism != Self::DEFAULT_PARALLELISM {
            output.push(format!("--parallelism {}", self.parallelism));
        }
        if self.overwrite != Self::DEFAULT_OVERWRITE {
            output.push("--overwrite".to_string());
        }
        if self.compat_mode != Self::DEFAULT_COMPAT {
            output.push(format!(
                "--compat {}",
                match self.compat_mode {
                    CompatMode::Trino => "trino",
                    CompatMode::C => "c",
                }
            ));
        }

        output.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            1.0,
            ".".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            1,
            false,
            CompatMode::Trino,
        );

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
    }

    #[test]
    fn test_try_new_validation() {
        assert!(Session::try_new(
            10.0,
            "/tmp".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            1,
            false,
            CompatMode::Trino,
        )
        .is_ok());

        assert!(Session::try_new(
            -1.0,
            ".".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            1,
            false,
            CompatMode::Trino,
        )
        .is_err());

        assert!(Session::try_new(
            1.0,
            "".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            1,
            false,
            CompatMode::Trino,
        )
        .is_err());

        assert!(Session::try_new(
            1.0,
            ".".to_string(),
            "".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            1,
            false,
            CompatMode::Trino,
        )
        .is_err());

        assert!(Session::try_new(
            1.0,
            ".".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            false,
            false,
            0,
            false,
            CompatMode::Trino,
        )
        .is_err());
    }

    #[test]
    fn test_default_session() {
        let session = Session::default();
        assert_eq!(session.get_scaling().get_scale(), 1.0);
        assert_eq!(session.get_target_directory(), ".");
        assert!(!session.generate_only_one_table());
    }

    #[test]
    fn test_with_methods() {
        let session = Session::default();

        let session_with_table = session.with_table(Table::CatalogSales);
        assert!(session_with_table.generate_only_one_table());
        assert_eq!(
            session_with_table.get_only_table_to_generate(),
            Table::CatalogSales
        );

        let session_with_scale = session.with_scale(10.0);
        assert_eq!(session_with_scale.get_scaling().get_scale(), 10.0);

        let session_with_parallelism = session.with_parallelism(4);
        assert_eq!(session_with_parallelism.get_parallelism(), 4);

        let session_with_chunk = session.with_chunk_number(2);
        assert_eq!(session_with_chunk.get_chunk_number(), 2);

        let session_with_no_sexism = session.with_no_sexism(true);
        assert!(!session_with_no_sexism.is_sexist());
    }

    #[test]
    fn test_generate_only_one_table() {
        let session = Session::default();
        assert!(!session.generate_only_one_table());

        let session_with_table = session.with_table(Table::StoreSales);
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
        let session = Session::new(
            1.0,
            ".".to_string(),
            ".dat".to_string(),
            None,
            "".to_string(),
            '|',
            true, // do_not_terminate = true
            true, // no_sexism = true
            1,
            false,
            CompatMode::Trino,
        );

        assert!(!session.terminate_rows_with_separator()); // negation of do_not_terminate
        assert!(!session.is_sexist()); // negation of no_sexism
    }

    #[test]
    fn test_command_line_arguments() {
        let session = Session::new(
            2.0,
            "/tmp".to_string(),
            ".csv".to_string(),
            Some(Table::CatalogSales),
            "NULL".to_string(),
            ',',
            true,
            true,
            4,
            true,
            CompatMode::Trino,
        );

        let args = session.get_command_line_arguments();
        assert!(args.contains("--scale 2"));
        assert!(args.contains("--directory /tmp"));
        assert!(args.contains("--suffix .csv"));
        assert!(args.contains("--table catalog_sales"));
        assert!(args.contains("--null NULL"));
        assert!(args.contains("--separator ,"));
        assert!(args.contains("--do-not-terminate"));
        assert!(args.contains("--no-sexism"));
        assert!(args.contains("--parallelism 4"));
        assert!(args.contains("--overwrite"));
    }

    #[test]
    fn test_command_line_arguments_defaults() {
        let session = Session::default();
        let args = session.get_command_line_arguments();
        assert!(args.is_empty()); // All defaults, so no arguments needed
    }
}
