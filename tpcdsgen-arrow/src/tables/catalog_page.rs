use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogPageRow, CatalogPageRowGenerator};

/// Generate [`CatalogPageRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CatalogPageArrow {
    inner: CatalogPageRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CatalogPageRow>,
    cp_catalog_page_sk: PrimitiveArrayRecycler<Int32Type>,
    cp_catalog_page_id: StringViewArrayRecycler,
    cp_start_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cp_end_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cp_department: StringViewArrayRecycler,
    cp_catalog_number: PrimitiveArrayRecycler<Int32Type>,
    cp_catalog_page_number: PrimitiveArrayRecycler<Int32Type>,
    cp_description: StringViewArrayRecycler,
    cp_type: StringViewArrayRecycler,
}

impl CatalogPageArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogPage);
        Self {
            inner: CatalogPageRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            cp_catalog_page_sk: PrimitiveArrayRecycler::new(),
            cp_catalog_page_id: StringViewArrayRecycler::new(),
            cp_start_date_sk: PrimitiveArrayRecycler::new(),
            cp_end_date_sk: PrimitiveArrayRecycler::new(),
            cp_department: StringViewArrayRecycler::new(),
            cp_catalog_number: PrimitiveArrayRecycler::new(),
            cp_catalog_page_number: PrimitiveArrayRecycler::new(),
            cp_description: StringViewArrayRecycler::new(),
            cp_type: StringViewArrayRecycler::new(),
        }
    }
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.inner
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate only source rows `starting_row_number..=ending_row_number`
    /// (1-based, inclusive). The ending row number is clamped to the table's
    /// row count.
    pub fn with_source_row_range(
        mut self,
        starting_row_number: u64,
        ending_row_number: u64,
    ) -> Self {
        self.inner
            .set_source_row_range(starting_row_number, ending_row_number);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchReader for CatalogPageArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CatalogPageArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scratch.clear();
        self.scratch
            .extend(self.inner.by_ref().take(self.batch_size));
        let rows = &self.scratch;
        if rows.is_empty() {
            return None;
        }

        let schema = self.schema();
        let Self {
            cp_catalog_page_sk,
            cp_catalog_page_id,
            cp_start_date_sk,
            cp_end_date_sk,
            cp_department,
            cp_catalog_number,
            cp_catalog_page_number,
            cp_description,
            cp_type,
            ..
        } = self;
        let nbm = CatalogPageRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CatalogPageRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let int =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CatalogPageRow) -> i32| {
                rec.int32(rows, pos, nbm, f)
            };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&CatalogPageRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let cp_catalog_page_sk = sk32(
            cp_catalog_page_sk,
            0,
            CatalogPageRow::get_cp_catalog_page_sk,
        );
        let cp_catalog_page_id = str(
            cp_catalog_page_id,
            1,
            CatalogPageRow::get_cp_catalog_page_id,
        );
        let cp_start_date_sk = sk32(cp_start_date_sk, 2, CatalogPageRow::get_cp_start_date_id);
        let cp_end_date_sk = sk32(cp_end_date_sk, 3, CatalogPageRow::get_cp_end_date_id);
        // CpPromoId occupies global bit 4 but is not in the output schema,
        // so output columns shift: CpDepartment=bit5, ..., CpType=bit9.
        let cp_department = str(cp_department, 5, CatalogPageRow::get_cp_department);
        let cp_catalog_number = int(cp_catalog_number, 6, CatalogPageRow::get_cp_catalog_number);
        let cp_catalog_page_number = int(
            cp_catalog_page_number,
            7,
            CatalogPageRow::get_cp_catalog_page_number,
        );
        let cp_description = str(cp_description, 8, CatalogPageRow::get_cp_description);
        let cp_type = str(cp_type, 9, CatalogPageRow::get_cp_type);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(cp_catalog_page_sk),
                Arc::new(cp_catalog_page_id),
                Arc::new(cp_start_date_sk),
                Arc::new(cp_end_date_sk),
                Arc::new(cp_department),
                Arc::new(cp_catalog_number),
                Arc::new(cp_catalog_page_number),
                Arc::new(cp_description),
                Arc::new(cp_type),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cp_catalog_page_sk", DataType::Int32, false),
        Field::new("cp_catalog_page_id", DataType::Utf8View, false),
        Field::new("cp_start_date_sk", DataType::Int32, true),
        Field::new("cp_end_date_sk", DataType::Int32, true),
        Field::new("cp_department", DataType::Utf8View, true),
        Field::new("cp_catalog_number", DataType::Int32, true),
        Field::new("cp_catalog_page_number", DataType::Int32, true),
        Field::new("cp_description", DataType::Utf8View, true),
        Field::new("cp_type", DataType::Utf8View, true),
    ]))
}
