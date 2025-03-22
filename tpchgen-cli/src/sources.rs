//! Implementations of [`Source`] for various different TPCH tables

use super::generate::Source;
use std::io::Write;
use tpchgen::csv::{
    CustomerCsv, LineItemCsv, NationCsv, OrderCsv, PartCsv, PartSuppCsv, RegionCsv, SupplierCsv,
};
use tpchgen::generators::{
    CustomerGenerator, LineItemGenerator, NationGenerator, OrderGenerator, PartGenerator,
    PartSupplierGenerator, RegionGenerator, SupplierGenerator,
};

/// Define a Source that writes the table in TBL format
macro_rules! define_tbl_source {
    ($SOURCE_NAME:ident, $GENERATOR_TYPE:ty) => {
        pub struct $SOURCE_NAME {
            inner: $GENERATOR_TYPE,
        }

        impl $SOURCE_NAME {
            pub fn new(inner: $GENERATOR_TYPE) -> Self {
                Self { inner }
            }
        }

        impl Source for $SOURCE_NAME {
            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                for item in self.inner.iter() {
                    // The default Display impl writes TBL format
                    writeln!(&mut buffer, "{item}").expect("writing to memory is infallible");
                }
                buffer
            }
        }
    };
}

/// Define a Source that writes the table in CSV format
macro_rules! define_csv_source {
    ($SOURCE_NAME:ident, $GENERATOR_TYPE:ty, $FORMATTER:ty) => {
        pub struct $SOURCE_NAME {
            inner: $GENERATOR_TYPE,
        }

        impl $SOURCE_NAME {
            pub fn new(inner: $GENERATOR_TYPE) -> Self {
                Self { inner }
            }
        }

        impl Source for $SOURCE_NAME {
            fn create(self, mut buffer: Vec<u8>) -> Vec<u8> {
                writeln!(&mut buffer, "{}", <$FORMATTER>::header())
                    .expect("writing to memory is infallible");
                for item in self.inner.iter() {
                    let formatter = <$FORMATTER>::new(item);
                    writeln!(&mut buffer, "{formatter}").expect("writing to memory is infallible");
                }
                buffer
            }
        }
    };
}

// Define .tbl sources for all tables
define_tbl_source!(NationTblSource, NationGenerator<'static>);
define_tbl_source!(RegionTblSource, RegionGenerator<'static>);
define_tbl_source!(PartTblSource, PartGenerator<'static>);
define_tbl_source!(SupplierTblSource, SupplierGenerator<'static>);
define_tbl_source!(PartSuppTblSource, PartSupplierGenerator<'static>);
define_tbl_source!(CustomerTblSource, CustomerGenerator<'static>);
define_tbl_source!(OrderTblSource, OrderGenerator<'static>);
define_tbl_source!(LineItemTblSource, LineItemGenerator<'static>);

// Define .csv sources for all tables
define_csv_source!(NationCsvSource, NationGenerator<'static>, NationCsv);
define_csv_source!(RegionCsvSource, RegionGenerator<'static>, RegionCsv);
define_csv_source!(PartCsvSource, PartGenerator<'static>, PartCsv);
define_csv_source!(SupplierCsvSource, SupplierGenerator<'static>, SupplierCsv);
define_csv_source!(
    PartSuppCsvSource,
    PartSupplierGenerator<'static>,
    PartSuppCsv
);
define_csv_source!(CustomerCsvSource, CustomerGenerator<'static>, CustomerCsv);
define_csv_source!(OrderCsvSource, OrderGenerator<'static>, OrderCsv);
define_csv_source!(LineItemCsvSource, LineItemGenerator<'static>, LineItemCsv);
