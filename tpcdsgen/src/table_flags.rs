/// Table flags indicating special properties (TableFlags)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableFlags {
    /// FL_TYPE_2 in the C code. This dimension keeps history -- rowcount shows unique entities (not including revisions).
    keeps_history: bool,
    /// This table has low rowcount; used by Address.java
    is_small: bool,
    /// This table is date-based for generation
    is_date_based: bool,
}

impl TableFlags {
    /// Create new TableFlags with specified flags (const-compatible)
    pub const fn new(keeps_history: bool, is_small: bool, is_date_based: bool) -> Self {
        Self {
            keeps_history,
            is_small,
            is_date_based,
        }
    }

    /// Check if this table keeps history
    pub fn keeps_history(&self) -> bool {
        self.keeps_history
    }

    /// Check if this is a small table
    pub fn is_small(&self) -> bool {
        self.is_small
    }

    /// Check if this table is date-based
    pub fn is_date_based(&self) -> bool {
        self.is_date_based
    }
}

/// Builder for TableFlags (TableFlagsBuilder)
#[derive(Debug, Default)]
pub struct TableFlagsBuilder {
    keeps_history: bool,
    is_small: bool,
    is_date_based: bool,
}

impl TableFlagsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the keeps_history flag
    pub fn set_keeps_history(mut self) -> Self {
        self.keeps_history = true;
        self
    }

    /// Set the is_small flag
    pub fn set_is_small(mut self) -> Self {
        self.is_small = true;
        self
    }

    /// Set the is_date_based flag  
    pub fn set_is_date_based(mut self) -> Self {
        self.is_date_based = true;
        self
    }

    /// Build the final TableFlags
    pub fn build(self) -> TableFlags {
        TableFlags::new(self.keeps_history, self.is_small, self.is_date_based)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_flags_creation() {
        let flags = TableFlags::new(true, false, true);
        assert!(flags.keeps_history());
        assert!(!flags.is_small());
        assert!(flags.is_date_based());
    }

    #[test]
    fn test_table_flags_default() {
        let flags = TableFlags::default();
        assert!(!flags.keeps_history());
        assert!(!flags.is_small());
        assert!(!flags.is_date_based());
    }

    #[test]
    fn test_table_flags_builder_empty() {
        let flags = TableFlagsBuilder::new().build();
        assert!(!flags.keeps_history());
        assert!(!flags.is_small());
        assert!(!flags.is_date_based());
    }

    #[test]
    fn test_table_flags_builder_all_flags() {
        let flags = TableFlagsBuilder::new()
            .set_keeps_history()
            .set_is_small()
            .set_is_date_based()
            .build();

        assert!(flags.keeps_history());
        assert!(flags.is_small());
        assert!(flags.is_date_based());
    }

    #[test]
    fn test_table_flags_builder_partial() {
        let flags = TableFlagsBuilder::new()
            .set_keeps_history()
            .set_is_date_based()
            .build();

        assert!(flags.keeps_history());
        assert!(!flags.is_small());
        assert!(flags.is_date_based());
    }

    #[test]
    fn test_table_flags_builder_chaining() {
        // Test that builder methods can be chained in any order
        let flags1 = TableFlagsBuilder::new()
            .set_is_small()
            .set_keeps_history()
            .build();

        let flags2 = TableFlagsBuilder::new()
            .set_keeps_history()
            .set_is_small()
            .build();

        assert_eq!(flags1, flags2);
        assert!(flags1.keeps_history());
        assert!(flags1.is_small());
        assert!(!flags1.is_date_based());
    }

    #[test]
    fn test_table_flags_equality() {
        let flags1 = TableFlags::new(true, false, true);
        let flags2 = TableFlags::new(true, false, true);
        let flags3 = TableFlags::new(false, false, true);

        assert_eq!(flags1, flags2);
        assert_ne!(flags1, flags3);
    }

    #[test]
    fn test_table_flags_clone() {
        let flags1 = TableFlags::new(true, true, false);
        let flags2 = flags1.clone();

        assert_eq!(flags1, flags2);
        assert!(flags2.keeps_history());
        assert!(flags2.is_small());
        assert!(!flags2.is_date_based());
    }

    #[test]
    fn test_java_style_usage() {
        // Test usage pattern matching Java: new TableFlagsBuilder().setKeepsHistory().setIsSmall().build()
        let flags = TableFlagsBuilder::new()
            .set_keeps_history()
            .set_is_small()
            .build();

        assert!(flags.keeps_history());
        assert!(flags.is_small());
        assert!(!flags.is_date_based());
    }
}
