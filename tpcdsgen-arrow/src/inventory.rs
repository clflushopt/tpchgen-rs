use crate::conversions::{opt, sk_opt};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, InventoryRowGenerator, RowGenerator};

pub struct InventoryArrow {
    generator: InventoryRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl InventoryArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Inventory);
        Self { generator: InventoryRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for InventoryArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for InventoryArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }
        let end = (self.current_row + self.batch_size as i64 - 1).min(self.row_count);

        let mut inv_date: Vec<Option<i64>> = Vec::new();
        let mut inv_item: Vec<Option<i64>> = Vec::new();
        let mut inv_warehouse: Vec<Option<i64>> = Vec::new();
        let mut inv_qty: Vec<Option<i32>> = Vec::new();

        for row_number in self.current_row..=end {
            let result = self.generator.generate_row_and_child_rows(row_number, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::Inventory(r) = g {
                    let nbm = r.null_bit_map();
                    inv_date.push(sk_opt(nbm, 0, r.get_inv_date_sk()));
                    inv_item.push(sk_opt(nbm, 1, r.get_inv_item_sk()));
                    inv_warehouse.push(sk_opt(nbm, 2, r.get_inv_warehouse_sk()));
                    inv_qty.push(opt(nbm, 3, r.get_inv_quantity_on_hand()));
                }
            }
            self.generator.consume_remaining_seeds_for_row();
        }
        self.current_row = end + 1;
        if inv_date.is_empty() { return None; }

        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(inv_date)),
            Arc::new(Int64Array::from(inv_item)),
            Arc::new(Int64Array::from(inv_warehouse)),
            Arc::new(Int32Array::from(inv_qty)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("inv_date_sk", DataType::Int64, true),
    Field::new("inv_item_sk", DataType::Int64, true),
    Field::new("inv_warehouse_sk", DataType::Int64, true),
    Field::new("inv_quantity_on_hand", DataType::Int32, true),
])));
