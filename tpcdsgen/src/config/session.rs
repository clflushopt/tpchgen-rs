use crate::config::{CompatMode, Scaling, Table};
use crate::error::{InvalidOptionError, Result};

/// Configuration for a TPC-DS data generation run.
///
/// A `Session` defines how TPC-DS data is generated.
#[derive(Debug, Clone)]
pub struct Session {
    scaling: Scaling,
    table: Option<Table>,
    no_sexism: bool,
    chunk_number: i32,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            scaling: Scaling::new_with_compat(Self::DEFAULT_SCALE, Self::DEFAULT_COMPAT),
            table: None,
            no_sexism: Self::DEFAULT_NO_SEXISM,
            chunk_number: Self::DEFAULT_CHUNK_NUMBER,
            compat_mode: Self::DEFAULT_COMPAT,
            command_line_arguments: None,
        }
    }
}

impl Session {
    pub const DEFAULT_SCALE: f64 = 1.0;
    pub const DEFAULT_NO_SEXISM: bool = false;
    pub const DEFAULT_CHUNK_NUMBER: i32 = 1;
    pub const DEFAULT_COMPAT: CompatMode = CompatMode::Trino;

    /// Convert this session into a builder initialized with its current values.
    pub fn into_builder(self) -> SessionBuilder {
        SessionBuilder {
            scale: self.scaling.get_scale(),
            table: self.table,
            no_sexism: self.no_sexism,
            chunk_number: self.chunk_number,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        }
    }

    /// Return the [`Scaling`] settings used for row counts.
    pub fn get_scaling(&self) -> &Scaling {
        &self.scaling
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

    /// Return whether generated manager names should match the reference
    /// implementation's original gendered data.
    pub fn is_sexist(&self) -> bool {
        !self.no_sexism
    }

    /// Return the one-based chunk number represented by this session.
    pub fn get_chunk_number(&self) -> i32 {
        self.chunk_number
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
    table: Option<Table>,
    no_sexism: bool,
    chunk_number: i32,
    compat_mode: CompatMode,
    command_line_arguments: Option<String>,
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self {
            scale: Session::DEFAULT_SCALE,
            table: None,
            no_sexism: Session::DEFAULT_NO_SEXISM,
            chunk_number: Session::DEFAULT_CHUNK_NUMBER,
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

    /// Set whether gender-neutral manager names are enabled.
    pub fn with_no_sexism(mut self, no_sexism: bool) -> Self {
        self.no_sexism = no_sexism;
        self
    }

    /// Set the one-based chunk number represented by this session.
    pub fn with_chunk_number(mut self, chunk_number: i32) -> Self {
        self.chunk_number = chunk_number;
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
            table: self.table,
            no_sexism: self.no_sexism,
            chunk_number: self.chunk_number,
            compat_mode: self.compat_mode,
            command_line_arguments: self.command_line_arguments,
        })
    }

    fn validate(&self) -> Result<()> {
        if !(0.0..=100000.0).contains(&self.scale) {
            return Err(InvalidOptionError::with_message(
                "scale",
                &self.scale.to_string(),
                "Scale must be between 0 and 100000, inclusive",
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
        assert!(!session.generate_only_one_table());
        assert!(session.is_sexist());
        assert_eq!(session.get_chunk_number(), 1);
        assert_eq!(session.command_line_arguments(), None);
    }

    #[test]
    fn test_session_builder() {
        let session = SessionBuilder::new()
            .with_scale_factor(2.0)
            .with_table(Table::CatalogSales)
            .with_no_sexism(true)
            .with_chunk_number(2)
            .with_compat_mode(CompatMode::C)
            .with_command_line_arguments("tpcgen tpcds --scale-factor 2")
            .build()
            .unwrap();

        assert_eq!(session.get_scaling().get_scale(), 2.0);
        assert_eq!(session.get_table(), Some(Table::CatalogSales));
        assert!(!session.is_sexist());
        assert_eq!(session.get_chunk_number(), 2);
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

        assert!(SessionBuilder::new().with_chunk_number(0).build().is_err());
    }

    #[test]
    fn test_into_builder() {
        let session = Session::default();

        let session = session
            .into_builder()
            .with_table(Table::CatalogSales)
            .with_scale_factor(10.0)
            .with_chunk_number(2)
            .with_no_sexism(true)
            .with_command_line_arguments("initial")
            .without_command_line_arguments()
            .build()
            .unwrap();

        assert!(session.generate_only_one_table());
        assert_eq!(session.get_only_table_to_generate(), Table::CatalogSales);
        assert_eq!(session.get_scaling().get_scale(), 10.0);
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
}
