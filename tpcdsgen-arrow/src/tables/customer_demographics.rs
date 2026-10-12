use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CustomerDemographicsRow, CustomerDemographicsRowGenerator};

/// Generate [`CustomerDemographicsRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CustomerDemographicsArrow {
    inner: CustomerDemographicsRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CustomerDemographicsRow>,
    cd_demo_sk: PrimitiveArrayRecycler<Int32Type>,
    cd_gender: StringViewArrayRecycler,
    cd_marital_status: StringViewArrayRecycler,
    cd_education_status: StringViewArrayRecycler,
    cd_purchase_estimate: PrimitiveArrayRecycler<Int32Type>,
    cd_credit_rating: StringViewArrayRecycler,
    cd_dep_count: PrimitiveArrayRecycler<Int32Type>,
    cd_dep_employed_count: PrimitiveArrayRecycler<Int32Type>,
    cd_dep_college_count: PrimitiveArrayRecycler<Int32Type>,
}

impl CustomerDemographicsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session
            .get_scaling()
            .get_row_count(Table::CustomerDemographics);
        Self {
            inner: CustomerDemographicsRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            cd_demo_sk: PrimitiveArrayRecycler::new(),
            cd_gender: StringViewArrayRecycler::new(),
            cd_marital_status: StringViewArrayRecycler::new(),
            cd_education_status: StringViewArrayRecycler::new(),
            cd_purchase_estimate: PrimitiveArrayRecycler::new(),
            cd_credit_rating: StringViewArrayRecycler::new(),
            cd_dep_count: PrimitiveArrayRecycler::new(),
            cd_dep_employed_count: PrimitiveArrayRecycler::new(),
            cd_dep_college_count: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for CustomerDemographicsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CustomerDemographicsArrow {
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
            cd_demo_sk,
            cd_gender,
            cd_marital_status,
            cd_education_status,
            cd_purchase_estimate,
            cd_credit_rating,
            cd_dep_count,
            cd_dep_employed_count,
            cd_dep_college_count,
            ..
        } = self;
        let nbm = CustomerDemographicsRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>,
                    pos,
                    f: fn(&CustomerDemographicsRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>,
             pos,
             f: fn(&CustomerDemographicsRow) -> i32| { rec.int32(rows, pos, nbm, f) };
        let str =
            |rec: &mut StringViewArrayRecycler, pos, f: fn(&CustomerDemographicsRow) -> &str| {
                rec.str(rows, pos, nbm, f)
            };

        let cd_demo_sk = sk32(cd_demo_sk, 0, CustomerDemographicsRow::get_cd_demo_sk);
        let cd_gender = str(cd_gender, 1, CustomerDemographicsRow::get_cd_gender);
        let cd_marital_status = str(
            cd_marital_status,
            2,
            CustomerDemographicsRow::get_cd_marital_status,
        );
        let cd_education_status = str(
            cd_education_status,
            3,
            CustomerDemographicsRow::get_cd_education_status,
        );
        let cd_purchase_estimate = int(
            cd_purchase_estimate,
            4,
            CustomerDemographicsRow::get_cd_purchase_estimate,
        );
        let cd_credit_rating = str(
            cd_credit_rating,
            5,
            CustomerDemographicsRow::get_cd_credit_rating,
        );
        let cd_dep_count = int(cd_dep_count, 6, CustomerDemographicsRow::get_cd_dep_count);
        let cd_dep_employed_count = int(
            cd_dep_employed_count,
            7,
            CustomerDemographicsRow::get_cd_dep_employed_count,
        );
        let cd_dep_college_count = int(
            cd_dep_college_count,
            8,
            CustomerDemographicsRow::get_cd_dep_college_count,
        );

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(cd_demo_sk),
                Arc::new(cd_gender),
                Arc::new(cd_marital_status),
                Arc::new(cd_education_status),
                Arc::new(cd_purchase_estimate),
                Arc::new(cd_credit_rating),
                Arc::new(cd_dep_count),
                Arc::new(cd_dep_employed_count),
                Arc::new(cd_dep_college_count),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cd_demo_sk", DataType::Int32, false),
        Field::new("cd_gender", DataType::Utf8View, true),
        Field::new("cd_marital_status", DataType::Utf8View, true),
        Field::new("cd_education_status", DataType::Utf8View, true),
        Field::new("cd_purchase_estimate", DataType::Int32, true),
        Field::new("cd_credit_rating", DataType::Utf8View, true),
        Field::new("cd_dep_count", DataType::Int32, true),
        Field::new("cd_dep_employed_count", DataType::Int32, true),
        Field::new("cd_dep_college_count", DataType::Int32, true),
    ]))
}
