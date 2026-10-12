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
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct StoreSalesArrow {
    inner: StoreSalesRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<StoreSalesRow>,
    ss_sold_date_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_sold_time_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_item_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_store_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_promo_sk: PrimitiveArrayRecycler<Int32Type>,
    ss_ticket_number: PrimitiveArrayRecycler<Int64Type>,
    ss_quantity: PrimitiveArrayRecycler<Int32Type>,
    ss_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    ss_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    ss_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    ss_ext_discount_amt: PrimitiveArrayRecycler<Decimal128Type>,
    ss_ext_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    ss_ext_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    ss_ext_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    ss_ext_tax: PrimitiveArrayRecycler<Decimal128Type>,
    ss_coupon_amt: PrimitiveArrayRecycler<Decimal128Type>,
    ss_net_paid: PrimitiveArrayRecycler<Decimal128Type>,
    ss_net_paid_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    ss_net_profit: PrimitiveArrayRecycler<Decimal128Type>,
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
            ss_sold_date_sk: PrimitiveArrayRecycler::new(),
            ss_sold_time_sk: PrimitiveArrayRecycler::new(),
            ss_item_sk: PrimitiveArrayRecycler::new(),
            ss_customer_sk: PrimitiveArrayRecycler::new(),
            ss_cdemo_sk: PrimitiveArrayRecycler::new(),
            ss_hdemo_sk: PrimitiveArrayRecycler::new(),
            ss_addr_sk: PrimitiveArrayRecycler::new(),
            ss_store_sk: PrimitiveArrayRecycler::new(),
            ss_promo_sk: PrimitiveArrayRecycler::new(),
            ss_ticket_number: PrimitiveArrayRecycler::new(),
            ss_quantity: PrimitiveArrayRecycler::new(),
            ss_wholesale_cost: PrimitiveArrayRecycler::new(),
            ss_list_price: PrimitiveArrayRecycler::new(),
            ss_sales_price: PrimitiveArrayRecycler::new(),
            ss_ext_discount_amt: PrimitiveArrayRecycler::new(),
            ss_ext_sales_price: PrimitiveArrayRecycler::new(),
            ss_ext_wholesale_cost: PrimitiveArrayRecycler::new(),
            ss_ext_list_price: PrimitiveArrayRecycler::new(),
            ss_ext_tax: PrimitiveArrayRecycler::new(),
            ss_coupon_amt: PrimitiveArrayRecycler::new(),
            ss_net_paid: PrimitiveArrayRecycler::new(),
            ss_net_paid_inc_tax: PrimitiveArrayRecycler::new(),
            ss_net_profit: PrimitiveArrayRecycler::new(),
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
        let Self {
            ss_sold_date_sk,
            ss_sold_time_sk,
            ss_item_sk,
            ss_customer_sk,
            ss_cdemo_sk,
            ss_hdemo_sk,
            ss_addr_sk,
            ss_store_sk,
            ss_promo_sk,
            ss_ticket_number,
            ss_quantity,
            ss_wholesale_cost,
            ss_list_price,
            ss_sales_price,
            ss_ext_discount_amt,
            ss_ext_sales_price,
            ss_ext_wholesale_cost,
            ss_ext_list_price,
            ss_ext_tax,
            ss_coupon_amt,
            ss_net_paid,
            ss_net_paid_inc_tax,
            ss_net_profit,
            ..
        } = self;
        let nbm = StoreSalesRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&StoreSalesRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let sk64 =
            |rec: &mut PrimitiveArrayRecycler<Int64Type>, pos, f: fn(&StoreSalesRow) -> i64| {
                rec.int64_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_ss_pricing()))
            };

        let ss_sold_date_sk = sk32(ss_sold_date_sk, 0, StoreSalesRow::get_ss_sold_date_sk);
        let ss_sold_time_sk = sk32(ss_sold_time_sk, 1, StoreSalesRow::get_ss_sold_time_sk);
        let ss_item_sk = sk32(ss_item_sk, 2, StoreSalesRow::get_ss_sold_item_sk);
        let ss_customer_sk = sk32(ss_customer_sk, 3, StoreSalesRow::get_ss_sold_customer_sk);
        let ss_cdemo_sk = sk32(ss_cdemo_sk, 4, StoreSalesRow::get_ss_sold_cdemo_sk);
        let ss_hdemo_sk = sk32(ss_hdemo_sk, 5, StoreSalesRow::get_ss_sold_hdemo_sk);
        let ss_addr_sk = sk32(ss_addr_sk, 6, StoreSalesRow::get_ss_sold_addr_sk);
        let ss_store_sk = sk32(ss_store_sk, 7, StoreSalesRow::get_ss_sold_store_sk);
        let ss_promo_sk = sk32(ss_promo_sk, 8, StoreSalesRow::get_ss_sold_promo_sk);
        let ss_ticket_number = sk64(ss_ticket_number, 9, StoreSalesRow::get_ss_ticket_number);
        let ss_quantity = ss_quantity.int32(rows, 10, nbm, |r| r.get_ss_pricing().get_quantity());
        let ss_wholesale_cost = dec(ss_wholesale_cost, 11, Pricing::get_wholesale_cost);
        let ss_list_price = dec(ss_list_price, 12, Pricing::get_list_price);
        let ss_sales_price = dec(ss_sales_price, 13, Pricing::get_sales_price);
        // Java bug: coupon_amount appears at position 14 instead of ext_discount_amount
        let ss_ext_discount_amt = dec(ss_ext_discount_amt, 14, Pricing::get_coupon_amount);
        let ss_ext_sales_price = dec(ss_ext_sales_price, 15, Pricing::get_ext_sales_price);
        let ss_ext_wholesale_cost = dec(ss_ext_wholesale_cost, 16, Pricing::get_ext_wholesale_cost);
        let ss_ext_list_price = dec(ss_ext_list_price, 17, Pricing::get_ext_list_price);
        let ss_ext_tax = dec(ss_ext_tax, 18, Pricing::get_ext_tax);
        // Java bug: coupon_amount appears again at position 19
        let ss_coupon_amt = dec(ss_coupon_amt, 14, Pricing::get_coupon_amount);
        let ss_net_paid = dec(ss_net_paid, 19, Pricing::get_net_paid);
        let ss_net_paid_inc_tax = dec(ss_net_paid_inc_tax, 20, Pricing::get_net_paid_including_tax);
        let ss_net_profit = dec(ss_net_profit, 21, Pricing::get_net_profit);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ss_sold_date_sk),
                Arc::new(ss_sold_time_sk),
                Arc::new(ss_item_sk),
                Arc::new(ss_customer_sk),
                Arc::new(ss_cdemo_sk),
                Arc::new(ss_hdemo_sk),
                Arc::new(ss_addr_sk),
                Arc::new(ss_store_sk),
                Arc::new(ss_promo_sk),
                Arc::new(ss_ticket_number),
                Arc::new(ss_quantity),
                Arc::new(ss_wholesale_cost),
                Arc::new(ss_list_price),
                Arc::new(ss_sales_price),
                Arc::new(ss_ext_discount_amt),
                Arc::new(ss_ext_sales_price),
                Arc::new(ss_ext_wholesale_cost),
                Arc::new(ss_ext_list_price),
                Arc::new(ss_ext_tax),
                Arc::new(ss_coupon_amt),
                Arc::new(ss_net_paid),
                Arc::new(ss_net_paid_inc_tax),
                Arc::new(ss_net_profit),
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
