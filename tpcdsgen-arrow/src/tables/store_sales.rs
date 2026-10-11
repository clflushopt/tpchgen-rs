use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{StoreSalesRow, StoreSalesRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`StoreSalesRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next. This only pays off if the consumer drops each [`RecordBatch`] before
/// pulling the next one; a consumer that keeps every batch (e.g. collects them
/// into a `Vec`) never reclaims a buffer and pays a small cost per batch for the
/// extra reference.
pub struct StoreSalesArrow {
    inner: StoreSalesRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<StoreSalesRow>,
    recyclers: Recyclers,
}

/// One buffer recycler per output column
#[derive(Default)]
struct Recyclers {
    /// ss_sold_date_sk .. ss_promo_sk (null bits 0..=8)
    sk: [PrimitiveArrayRecycler<Int32Type>; 9],
    ticket: PrimitiveArrayRecycler<Int64Type>,
    quantity: PrimitiveArrayRecycler<Int32Type>,
    /// ss_wholesale_cost .. ss_net_profit
    dec: [PrimitiveArrayRecycler<Decimal128Type>; 12],
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
            recyclers: Recyclers::default(),
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

        let schema = self.schema();
        let Recyclers {
            sk,
            ticket,
            quantity,
            dec,
        } = &mut self.recyclers;
        let nbm = StoreSalesRow::null_bit_map;
        let mut sk32 =
            |i: usize, f: fn(&StoreSalesRow) -> i64| sk[i].int32_sk(rows, i as u32, nbm, f);
        let mut dec = |i: usize, pos, f: fn(&Pricing) -> Decimal| {
            dec[i].decimal_7_2(rows, pos, nbm, |r| f(r.get_ss_pricing()))
        };

        let batch = RecordBatch::try_new(
            schema,
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
                Arc::new(ticket.int64_sk(rows, 9, nbm, StoreSalesRow::get_ss_ticket_number)),
                Arc::new(quantity.int32(rows, 10, nbm, |r| r.get_ss_pricing().get_quantity())),
                Arc::new(dec(0, 11, Pricing::get_wholesale_cost)),
                Arc::new(dec(1, 12, Pricing::get_list_price)),
                Arc::new(dec(2, 13, Pricing::get_sales_price)),
                // Java bug: coupon_amount appears at position 14 instead of ext_discount_amount
                Arc::new(dec(3, 14, Pricing::get_coupon_amount)),
                Arc::new(dec(4, 15, Pricing::get_ext_sales_price)),
                Arc::new(dec(5, 16, Pricing::get_ext_wholesale_cost)),
                Arc::new(dec(6, 17, Pricing::get_ext_list_price)),
                Arc::new(dec(7, 18, Pricing::get_ext_tax)),
                // Java bug: coupon_amount appears again at position 19
                Arc::new(dec(8, 14, Pricing::get_coupon_amount)),
                Arc::new(dec(9, 19, Pricing::get_net_paid)),
                Arc::new(dec(10, 20, Pricing::get_net_paid_including_tax)),
                Arc::new(dec(11, 21, Pricing::get_net_profit)),
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
