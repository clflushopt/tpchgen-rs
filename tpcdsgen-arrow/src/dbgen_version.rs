use crate::conversions::{opt, string_view_array_from_opt_iter};
use crate::{RecordBatchIterator, RowIter, DEFAULT_BATCH_SIZE};
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{DbgenVersionRowGenerator, GeneratedRow};

pub struct DbgenVersionArrow {
    inner: RowIter<DbgenVersionRowGenerator>,
    batch_size: usize,
}

impl DbgenVersionArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::DbgenVersion);
        Self {
            inner: RowIter::new(DbgenVersionRowGenerator::new(), session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchIterator for DbgenVersionArrow {
    fn schema(&self) -> &SchemaRef {
        &SCHEMA
    }
}

impl Iterator for DbgenVersionArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        let rows: Vec<_> = self
            .inner
            .by_ref()
            .map(|g| match g {
                GeneratedRow::DbgenVersion(r) => r,
                _ => unreachable!(),
            })
            .take(self.batch_size)
            .collect();
        if rows.is_empty() {
            return None;
        }

        let mut version: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut create_date: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut create_time: Vec<Option<String>> = Vec::with_capacity(rows.len());
        let mut cmdline: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for r in &rows {
            let nbm = r.null_bit_map();
            version.push(opt(nbm, 0, r.get_dv_version().to_owned()));
            create_date.push(opt(nbm, 1, r.get_dv_create_date().to_owned()));
            create_time.push(opt(nbm, 2, r.get_dv_create_time().to_owned()));
            cmdline.push(opt(nbm, 3, r.get_dv_cmdline_args().to_owned()));
        }

        Some(
            RecordBatch::try_new(
                Arc::clone(self.schema()),
                vec![
                    Arc::new(string_view_array_from_opt_iter(
                        version.iter().map(|s| s.as_deref()),
                    )),
                    Arc::new(string_view_array_from_opt_iter(
                        create_date.iter().map(|s| s.as_deref()),
                    )),
                    Arc::new(string_view_array_from_opt_iter(
                        create_time.iter().map(|s| s.as_deref()),
                    )),
                    Arc::new(string_view_array_from_opt_iter(
                        cmdline.iter().map(|s| s.as_deref()),
                    )),
                ],
            )
            .unwrap(),
        )
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("dv_version", DataType::Utf8View, true),
        Field::new("dv_create_date", DataType::Utf8View, true),
        Field::new("dv_create_time", DataType::Utf8View, true),
        Field::new("dv_cmdline_args", DataType::Utf8View, true),
    ]))
}
