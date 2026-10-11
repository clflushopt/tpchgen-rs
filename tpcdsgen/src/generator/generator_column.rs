use crate::column::Table;

/// GeneratorColumns are columns that are used only within the context of the
/// generator logic. The Enums that implement this interface may include columns
/// that are not user visible and will sometimes omit columns that are user visible
/// (because those get derived from other columns).
///
/// GeneratorColumn
///
/// The trait is sealed: [`AbstractRowGenerator`] relies on `values()` listing
/// every value of the type with contiguous global column numbers.
///
/// [`AbstractRowGenerator`]: crate::row::AbstractRowGenerator
pub trait GeneratorColumn: sealed::Sealed + Send + Sync + 'static {
    /// All generator columns of the table, in global column number order
    fn values() -> &'static [Self]
    where
        Self: Sized;

    /// Get the table this generator column belongs to
    fn get_table(&self) -> Table;

    /// Get the global column number for this generator column
    fn get_global_column_number(&self) -> i32;

    /// Get the number of seeds per row for this generator column  
    fn get_seeds_per_row(&self) -> u32;
}

pub(crate) mod sealed {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a simple test implementation
    struct TestGeneratorColumn {
        table: Table,
        global_column_number: i32,
        seeds_per_row: u32,
    }

    impl sealed::Sealed for TestGeneratorColumn {}

    impl GeneratorColumn for TestGeneratorColumn {
        fn values() -> &'static [Self] {
            &[]
        }

        fn get_table(&self) -> Table {
            self.table
        }

        fn get_global_column_number(&self) -> i32 {
            self.global_column_number
        }

        fn get_seeds_per_row(&self) -> u32 {
            self.seeds_per_row
        }
    }

    #[test]
    fn test_generator_column_trait() {
        let test_column = TestGeneratorColumn {
            table: Table::CallCenter,
            global_column_number: 5,
            seeds_per_row: 10,
        };

        assert_eq!(test_column.get_table(), Table::CallCenter);
        assert_eq!(test_column.get_global_column_number(), 5);
        assert_eq!(test_column.get_seeds_per_row(), 10);
    }
}
