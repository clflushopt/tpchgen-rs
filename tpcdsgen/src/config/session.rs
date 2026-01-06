use crate::config::{Options, Scaling, Table};

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
}

impl Default for Session {
    fn default() -> Self {
        Session::get_default_session()
    }
}

impl Session {
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
        )
    }

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
    ) -> Self {
        Session {
            scaling: Scaling::new(scale),
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
        }
    }

    /// Get default session with all default values
    pub fn get_default_session() -> Self {
        Options::new().to_session().unwrap()
    }

    // Builder-like methods (return new Session with updated field)
    pub fn with_table(&self, table: Table) -> Self {
        Session {
            table: Some(table),
            ..self.clone()
        }
    }

    pub fn with_scale(&self, scale: f64) -> Self {
        Session {
            scaling: Scaling::new(scale),
            ..self.clone()
        }
    }

    pub fn with_parallelism(&self, parallelism: i32) -> Self {
        Session {
            parallelism,
            ..self.clone()
        }
    }

    pub fn with_chunk_number(&self, chunk_number: i32) -> Self {
        Session {
            chunk_number,
            ..self.clone()
        }
    }

    pub fn with_no_sexism(&self, no_sexism: bool) -> Self {
        Session {
            no_sexism,
            ..self.clone()
        }
    }

    // Accessor methods
    pub fn get_scaling(&self) -> &Scaling {
        &self.scaling
    }

    pub fn get_target_directory(&self) -> &str {
        &self.target_directory
    }

    pub fn get_suffix(&self) -> &str {
        &self.suffix
    }

    pub fn generate_only_one_table(&self) -> bool {
        self.table.is_some()
    }

    pub fn get_only_table_to_generate(&self) -> Table {
        self.table
            .unwrap_or_else(|| panic!("table not present - call generate_only_one_table() first"))
    }

    pub fn get_table(&self) -> Option<Table> {
        self.table
    }

    pub fn get_null_string(&self) -> &str {
        &self.null_string
    }

    pub fn get_separator(&self) -> char {
        self.separator
    }

    pub fn terminate_rows_with_separator(&self) -> bool {
        !self.do_not_terminate
    }

    pub fn is_sexist(&self) -> bool {
        !self.no_sexism
    }

    pub fn get_parallelism(&self) -> i32 {
        self.parallelism
    }

    pub fn get_chunk_number(&self) -> i32 {
        self.chunk_number
    }

    pub fn should_overwrite(&self) -> bool {
        self.overwrite
    }

    /// Reconstruct command line arguments that would produce this session
    pub fn get_command_line_arguments(&self) -> String {
        let mut output = Vec::new();

        if self.scaling.get_scale() != Options::DEFAULT_SCALE {
            output.push(format!("--scale {}", self.scaling.get_scale()));
        }
        if self.target_directory != Options::DEFAULT_DIRECTORY {
            output.push(format!("--directory {}", self.target_directory));
        }
        if self.suffix != Options::DEFAULT_SUFFIX {
            output.push(format!("--suffix {}", self.suffix));
        }
        if let Some(table) = self.table {
            output.push(format!("--table {}", table.get_name()));
        }
        if self.null_string != Options::DEFAULT_NULL_STRING {
            output.push(format!("--null {}", self.null_string));
        }
        if self.separator != Options::DEFAULT_SEPARATOR {
            output.push(format!("--separator {}", self.separator));
        }
        if self.do_not_terminate != Options::DEFAULT_DO_NOT_TERMINATE {
            output.push("--do-not-terminate".to_string());
        }
        if self.no_sexism != Options::DEFAULT_NO_SEXISM {
            output.push("--no-sexism".to_string());
        }
        if self.parallelism != Options::DEFAULT_PARALLELISM {
            output.push(format!("--parallelism {}", self.parallelism));
        }
        if self.overwrite != Options::DEFAULT_OVERWRITE {
            output.push("--overwrite".to_string());
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
    fn test_default_session() {
        let session = Session::get_default_session();
        assert_eq!(session.get_scaling().get_scale(), 1.0);
        assert_eq!(session.get_target_directory(), ".");
        assert!(!session.generate_only_one_table());
    }

    #[test]
    fn test_with_methods() {
        let session = Session::get_default_session();

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
        let session = Session::get_default_session();
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
        let session = Session::get_default_session();
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
        let session = Session::get_default_session();
        let args = session.get_command_line_arguments();
        assert!(args.is_empty()); // All defaults, so no arguments needed
    }
}
