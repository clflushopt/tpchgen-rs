//! Verifies the correctness of the Arrow TPCH generator by parsing the canonical TBL format
//! and comparing with the generated Arrow RecordBatches

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use std::io::Write;
use std::sync::Arc;
use tpchgen::csv::LineItemCsv;
use tpchgen::generators::{LineItem, LineItemGenerator};
use tpchgen_arrow::{LineItemArrow, RecordBatchIterator};

#[test]
fn test_tpchgen_lineitem_tbl() {
    do_test_tpchgen_lineitem(0.1, TestFormat::TBL)
}

#[test]
fn test_tpchgen_lineitem_csv() {
    do_test_tpchgen_lineitem(0.1, TestFormat::CSV)
}

/// Generates LineItem's using the specified format and compares the results of
/// parsing with the Arrow CSV parser with directly generated the batches
fn do_test_tpchgen_lineitem(scale_factor: f64, format: TestFormat) {
    let batch_size = 1000;

    // TPCH scale factor 1
    let lineitem_generator = LineItemGenerator::new(scale_factor, 1, 1);
    let mut lineitem_iter = lineitem_generator.clone().iter();
    let mut arrow_iter = LineItemArrow::new(lineitem_generator).with_batch_size(batch_size);

    let mut batch_num = 0;
    while let Some(arrow_batch) = arrow_iter.next() {
        println!("Batch {}", batch_num);
        batch_num += 1;
        let mut text_data = Vec::new();
        format.write_lineitem_header(&mut text_data);
        lineitem_iter.by_ref().take(batch_size).for_each(|item| {
            format.write_lineitem(item, &mut text_data);
        });
        let tbl_batch = format.parse(&text_data, arrow_iter.schema(), batch_size);
        assert_eq!(tbl_batch, arrow_batch);
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum TestFormat {
    /// Generate and parse data as TBL format ('|' delimited)
    TBL,
    /// Generate and parse data as CSV format
    CSV,
}

impl TestFormat {
    /// Write the header for the LineItem format into the provided buffer
    fn write_lineitem_header(&self, text_data: &mut Vec<u8>) {
        match self {
            TestFormat::TBL => {}
            TestFormat::CSV => {
                writeln!(text_data, "{}\n", LineItemCsv::header()).unwrap();
            }
        }
    }

    /// Write a LineItem into the provided buffer
    fn write_lineitem(&self, line_item: LineItem<'_>, text_data: &mut Vec<u8>) {
        match self {
            TestFormat::TBL => {
                write!(text_data, "{}", line_item).unwrap();
                // Note: TBL lines end with '|' which the arrow csv parser treats as a
                // delimiter for a new column so replace the last '|' with a newline
                let end_offset = text_data.len() - 1;
                text_data[end_offset] = b'\n';
            }
            TestFormat::CSV => {
                writeln!(text_data, "{}", LineItemCsv::new(line_item)).unwrap();
            }
        }
    }

    /// Parse the provided data into an Arrow RecordBatch
    fn parse(&self, data: &[u8], schema: &SchemaRef, batch_size: usize) -> RecordBatch {
        let builder =
            arrow_csv::reader::ReaderBuilder::new(Arc::clone(schema)).with_batch_size(batch_size);

        let builder = match self {
            TestFormat::TBL => builder.with_header(false).with_delimiter(b'|'),
            TestFormat::CSV => builder.with_header(true),
        };

        let mut parser = builder.build(data).unwrap();

        let batch = parser
            .next()
            .expect("should have a batch")
            .expect("should have no errors parsing");
        assert!(parser.next().is_none(), "should have only one batch");
        batch
    }
}
