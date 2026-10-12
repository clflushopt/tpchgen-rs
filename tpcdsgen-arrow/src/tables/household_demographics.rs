use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{HouseholdDemographicsRow, HouseholdDemographicsRowGenerator};

/// Generate [`HouseholdDemographicsRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct HouseholdDemographicsArrow {
    inner: HouseholdDemographicsRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<HouseholdDemographicsRow>,
    hd_demo_sk: PrimitiveArrayRecycler<Int32Type>,
    hd_income_band_sk: PrimitiveArrayRecycler<Int32Type>,
    hd_buy_potential: StringViewArrayRecycler,
    hd_dep_count: PrimitiveArrayRecycler<Int32Type>,
    hd_vehicle_count: PrimitiveArrayRecycler<Int32Type>,
}

impl HouseholdDemographicsArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session
            .get_scaling()
            .get_row_count(Table::HouseholdDemographics);
        Self {
            inner: HouseholdDemographicsRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            hd_demo_sk: PrimitiveArrayRecycler::new(),
            hd_income_band_sk: PrimitiveArrayRecycler::new(),
            hd_buy_potential: StringViewArrayRecycler::new(),
            hd_dep_count: PrimitiveArrayRecycler::new(),
            hd_vehicle_count: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for HouseholdDemographicsArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for HouseholdDemographicsArrow {
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
            hd_demo_sk,
            hd_income_band_sk,
            hd_buy_potential,
            hd_dep_count,
            hd_vehicle_count,
            ..
        } = self;
        let nbm = HouseholdDemographicsRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>,
                    pos,
                    f: fn(&HouseholdDemographicsRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>,
             pos,
             f: fn(&HouseholdDemographicsRow) -> i32| { rec.int32(rows, pos, nbm, f) };
        let str =
            |rec: &mut StringViewArrayRecycler, pos, f: fn(&HouseholdDemographicsRow) -> &str| {
                rec.str(rows, pos, nbm, f)
            };

        let hd_demo_sk = sk32(hd_demo_sk, 0, HouseholdDemographicsRow::get_hd_demo_sk);
        let hd_income_band_sk = sk32(
            hd_income_band_sk,
            1,
            HouseholdDemographicsRow::get_hd_income_band_sk,
        );
        let hd_buy_potential = str(
            hd_buy_potential,
            2,
            HouseholdDemographicsRow::get_hd_buy_potential,
        );
        let hd_dep_count = int(hd_dep_count, 3, HouseholdDemographicsRow::get_hd_dep_count);
        let hd_vehicle_count = int(
            hd_vehicle_count,
            4,
            HouseholdDemographicsRow::get_hd_vehicle_count,
        );

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(hd_demo_sk),
                Arc::new(hd_income_band_sk),
                Arc::new(hd_buy_potential),
                Arc::new(hd_dep_count),
                Arc::new(hd_vehicle_count),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("hd_demo_sk", DataType::Int32, false),
        Field::new("hd_income_band_sk", DataType::Int32, true),
        Field::new("hd_buy_potential", DataType::Utf8View, true),
        Field::new("hd_dep_count", DataType::Int32, true),
        Field::new("hd_vehicle_count", DataType::Int32, true),
    ]))
}
