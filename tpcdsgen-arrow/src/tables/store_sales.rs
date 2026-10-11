use crate::conversions::{decimal_7_2_array, int32_array, int32_sk_array, int64_sk_array};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{StoreSalesRow, StoreSalesRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`StoreSalesRow`]s in [`RecordBatch`] format
pub struct StoreSalesArrow {
    inner: StoreSalesRowGenerator,
    batch_size: usize,
    // reused allocation across batches
    scratch: Vec<StoreSalesRow>,
}

impl StoreSalesArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::StoreSales);
        Self {
            inner: StoreSalesRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
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

impl RecordBatchReader for StoreSalesArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for StoreSalesArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scratch.clear();
        self.scratch
            .extend(self.inner.by_ref().take(self.batch_size));
        let rows = &self.scratch;
        if rows.is_empty() {
            return None;
        }

        let nbm = StoreSalesRow::null_bit_map;
        let sk32 = |pos, f: fn(&StoreSalesRow) -> i64| int32_sk_array(rows, pos, nbm, f);
        let dec = |pos, f: fn(&Pricing) -> Decimal| {
            decimal_7_2_array(rows, pos, nbm, |r| f(r.get_ss_pricing()))
        };

        let batch = RecordBatch::try_new(
            self.schema(),
            vec![
                Arc::new(sk32(0, StoreSalesRow::get_ss_sold_date_sk)),
                Arc::new(sk32(1, StoreSalesRow::get_ss_sold_time_sk)),
                Arc::new(sk32(2, StoreSalesRow::get_ss_sold_item_sk)),
                Arc::new(sk32(3, StoreSalesRow::get_ss_sold_customer_sk)),
                Arc::new(sk32(4, StoreSalesRow::get_ss_sold_cdemo_sk)),
                Arc::new(sk32(5, StoreSalesRow::get_ss_sold_hdemo_sk)),
                Arc::new(sk32(6, StoreSalesRow::get_ss_sold_addr_sk)),
                Arc::new(sk32(7, StoreSalesRow::get_ss_sold_store_sk)),
                Arc::new(sk32(8, StoreSalesRow::get_ss_sold_promo_sk)),
                Arc::new(int64_sk_array(
                    rows,
                    9,
                    nbm,
                    StoreSalesRow::get_ss_ticket_number,
                )),
                Arc::new(int32_array(rows, 10, nbm, |r| {
                    r.get_ss_pricing().get_quantity()
                })),
                Arc::new(dec(11, Pricing::get_wholesale_cost)),
                Arc::new(dec(12, Pricing::get_list_price)),
                Arc::new(dec(13, Pricing::get_sales_price)),
                // Java bug: coupon_amount appears at position 14 instead of ext_discount_amount
                Arc::new(dec(14, Pricing::get_coupon_amount)),
                Arc::new(dec(15, Pricing::get_ext_sales_price)),
                Arc::new(dec(16, Pricing::get_ext_wholesale_cost)),
                Arc::new(dec(17, Pricing::get_ext_list_price)),
                Arc::new(dec(18, Pricing::get_ext_tax)),
                // Java bug: coupon_amount appears again at position 19
                Arc::new(dec(14, Pricing::get_coupon_amount)),
                Arc::new(dec(19, Pricing::get_net_paid)),
                Arc::new(dec(20, Pricing::get_net_paid_including_tax)),
                Arc::new(dec(21, Pricing::get_net_profit)),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ss_sold_date_sk", DataType::Int32, true),
        Field::new("ss_sold_time_sk", DataType::Int32, true),
        Field::new("ss_item_sk", DataType::Int32, false),
        Field::new("ss_customer_sk", DataType::Int32, true),
        Field::new("ss_cdemo_sk", DataType::Int32, true),
        Field::new("ss_hdemo_sk", DataType::Int32, true),
        Field::new("ss_addr_sk", DataType::Int32, true),
        Field::new("ss_store_sk", DataType::Int32, true),
        Field::new("ss_promo_sk", DataType::Int32, true),
        Field::new("ss_ticket_number", DataType::Int64, false),
        Field::new("ss_quantity", DataType::Int32, true),
        Field::new("ss_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("ss_list_price", DataType::Decimal128(7, 2), true),
        Field::new("ss_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("ss_ext_discount_amt", DataType::Decimal128(7, 2), true),
        Field::new("ss_ext_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("ss_ext_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("ss_ext_list_price", DataType::Decimal128(7, 2), true),
        Field::new("ss_ext_tax", DataType::Decimal128(7, 2), true),
        Field::new("ss_coupon_amt", DataType::Decimal128(7, 2), true),
        Field::new("ss_net_paid", DataType::Decimal128(7, 2), true),
        Field::new("ss_net_paid_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("ss_net_profit", DataType::Decimal128(7, 2), true),
    ]))
}
