use crate::conversions::{opt, sk_opt, string_view_array_from_opt_iter};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, HouseholdDemographicsRowGenerator, RowGenerator};

pub struct HouseholdDemographicsArrow {
    generator: HouseholdDemographicsRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl HouseholdDemographicsArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::HouseholdDemographics);
        Self { generator: HouseholdDemographicsRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for HouseholdDemographicsArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for HouseholdDemographicsArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut demo_sk: Vec<Option<i64>> = Vec::new();
        let mut income_band_sk: Vec<Option<i64>> = Vec::new();
        let mut buy_potential: Vec<Option<String>> = Vec::new();
        let mut dep_count: Vec<Option<i32>> = Vec::new();
        let mut vehicle_count: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::HouseholdDemographics(r) = g {
                    let nbm = r.null_bit_map();
                    demo_sk.push(sk_opt(nbm, 0, r.get_hd_demo_sk()));
                    income_band_sk.push(sk_opt(nbm, 1, r.get_hd_income_band_sk()));
                    buy_potential.push(opt(nbm, 2, r.get_hd_buy_potential().to_owned()));
                    dep_count.push(opt(nbm, 3, r.get_hd_dep_count()));
                    vehicle_count.push(opt(nbm, 4, r.get_hd_vehicle_count()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if demo_sk.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(demo_sk)),
            Arc::new(Int64Array::from(income_band_sk)),
            Arc::new(string_view_array_from_opt_iter(buy_potential.iter().map(|s| s.as_deref()))),
            Arc::new(Int32Array::from(dep_count)),
            Arc::new(Int32Array::from(vehicle_count)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("hd_demo_sk", DataType::Int64, true),
    Field::new("hd_income_band_sk", DataType::Int64, true),
    Field::new("hd_buy_potential", DataType::Utf8View, true),
    Field::new("hd_dep_count", DataType::Int32, true),
    Field::new("hd_vehicle_count", DataType::Int32, true),
])));
