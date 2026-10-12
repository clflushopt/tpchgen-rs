use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{WebSalesRow, WebSalesRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`WebSalesRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct WebSalesArrow {
    inner: WebSalesRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<WebSalesRow>,
    ws_sold_date_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_sold_time_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_date_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_item_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_bill_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_bill_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_bill_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_bill_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_web_page_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_web_site_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_ship_mode_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_warehouse_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_promo_sk: PrimitiveArrayRecycler<Int32Type>,
    ws_order_number: PrimitiveArrayRecycler<Int64Type>,
    ws_quantity: PrimitiveArrayRecycler<Int32Type>,
    ws_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    ws_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    ws_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_discount_amt: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_tax: PrimitiveArrayRecycler<Decimal128Type>,
    ws_coupon_amt: PrimitiveArrayRecycler<Decimal128Type>,
    ws_ext_ship_cost: PrimitiveArrayRecycler<Decimal128Type>,
    ws_net_paid: PrimitiveArrayRecycler<Decimal128Type>,
    ws_net_paid_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    ws_net_paid_inc_ship: PrimitiveArrayRecycler<Decimal128Type>,
    ws_net_paid_inc_ship_tax: PrimitiveArrayRecycler<Decimal128Type>,
    ws_net_profit: PrimitiveArrayRecycler<Decimal128Type>,
}

impl WebSalesArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSales);
        Self {
            inner: WebSalesRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            ws_sold_date_sk: PrimitiveArrayRecycler::new(),
            ws_sold_time_sk: PrimitiveArrayRecycler::new(),
            ws_ship_date_sk: PrimitiveArrayRecycler::new(),
            ws_item_sk: PrimitiveArrayRecycler::new(),
            ws_bill_customer_sk: PrimitiveArrayRecycler::new(),
            ws_bill_cdemo_sk: PrimitiveArrayRecycler::new(),
            ws_bill_hdemo_sk: PrimitiveArrayRecycler::new(),
            ws_bill_addr_sk: PrimitiveArrayRecycler::new(),
            ws_ship_customer_sk: PrimitiveArrayRecycler::new(),
            ws_ship_cdemo_sk: PrimitiveArrayRecycler::new(),
            ws_ship_hdemo_sk: PrimitiveArrayRecycler::new(),
            ws_ship_addr_sk: PrimitiveArrayRecycler::new(),
            ws_web_page_sk: PrimitiveArrayRecycler::new(),
            ws_web_site_sk: PrimitiveArrayRecycler::new(),
            ws_ship_mode_sk: PrimitiveArrayRecycler::new(),
            ws_warehouse_sk: PrimitiveArrayRecycler::new(),
            ws_promo_sk: PrimitiveArrayRecycler::new(),
            ws_order_number: PrimitiveArrayRecycler::new(),
            ws_quantity: PrimitiveArrayRecycler::new(),
            ws_wholesale_cost: PrimitiveArrayRecycler::new(),
            ws_list_price: PrimitiveArrayRecycler::new(),
            ws_sales_price: PrimitiveArrayRecycler::new(),
            ws_ext_discount_amt: PrimitiveArrayRecycler::new(),
            ws_ext_sales_price: PrimitiveArrayRecycler::new(),
            ws_ext_wholesale_cost: PrimitiveArrayRecycler::new(),
            ws_ext_list_price: PrimitiveArrayRecycler::new(),
            ws_ext_tax: PrimitiveArrayRecycler::new(),
            ws_coupon_amt: PrimitiveArrayRecycler::new(),
            ws_ext_ship_cost: PrimitiveArrayRecycler::new(),
            ws_net_paid: PrimitiveArrayRecycler::new(),
            ws_net_paid_inc_tax: PrimitiveArrayRecycler::new(),
            ws_net_paid_inc_ship: PrimitiveArrayRecycler::new(),
            ws_net_paid_inc_ship_tax: PrimitiveArrayRecycler::new(),
            ws_net_profit: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for WebSalesArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WebSalesArrow {
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
            ws_sold_date_sk,
            ws_sold_time_sk,
            ws_ship_date_sk,
            ws_item_sk,
            ws_bill_customer_sk,
            ws_bill_cdemo_sk,
            ws_bill_hdemo_sk,
            ws_bill_addr_sk,
            ws_ship_customer_sk,
            ws_ship_cdemo_sk,
            ws_ship_hdemo_sk,
            ws_ship_addr_sk,
            ws_web_page_sk,
            ws_web_site_sk,
            ws_ship_mode_sk,
            ws_warehouse_sk,
            ws_promo_sk,
            ws_order_number,
            ws_quantity,
            ws_wholesale_cost,
            ws_list_price,
            ws_sales_price,
            ws_ext_discount_amt,
            ws_ext_sales_price,
            ws_ext_wholesale_cost,
            ws_ext_list_price,
            ws_ext_tax,
            ws_coupon_amt,
            ws_ext_ship_cost,
            ws_net_paid,
            ws_net_paid_inc_tax,
            ws_net_paid_inc_ship,
            ws_net_paid_inc_ship_tax,
            ws_net_profit,
            ..
        } = self;
        let nbm = WebSalesRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebSalesRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_ws_pricing()))
            };

        let ws_sold_date_sk = sk32(ws_sold_date_sk, 0, WebSalesRow::get_ws_sold_date_sk);
        let ws_sold_time_sk = sk32(ws_sold_time_sk, 1, WebSalesRow::get_ws_sold_time_sk);
        let ws_ship_date_sk = sk32(ws_ship_date_sk, 2, WebSalesRow::get_ws_ship_date_sk);
        let ws_item_sk = sk32(ws_item_sk, 3, WebSalesRow::get_ws_item_sk);
        let ws_bill_customer_sk =
            sk32(ws_bill_customer_sk, 4, WebSalesRow::get_ws_bill_customer_sk);
        let ws_bill_cdemo_sk = sk32(ws_bill_cdemo_sk, 5, WebSalesRow::get_ws_bill_cdemo_sk);
        let ws_bill_hdemo_sk = sk32(ws_bill_hdemo_sk, 6, WebSalesRow::get_ws_bill_hdemo_sk);
        let ws_bill_addr_sk = sk32(ws_bill_addr_sk, 7, WebSalesRow::get_ws_bill_addr_sk);
        let ws_ship_customer_sk =
            sk32(ws_ship_customer_sk, 8, WebSalesRow::get_ws_ship_customer_sk);
        let ws_ship_cdemo_sk = sk32(ws_ship_cdemo_sk, 9, WebSalesRow::get_ws_ship_cdemo_sk);
        let ws_ship_hdemo_sk = sk32(ws_ship_hdemo_sk, 10, WebSalesRow::get_ws_ship_hdemo_sk);
        let ws_ship_addr_sk = sk32(ws_ship_addr_sk, 11, WebSalesRow::get_ws_ship_addr_sk);
        let ws_web_page_sk = sk32(ws_web_page_sk, 12, WebSalesRow::get_ws_web_page_sk);
        let ws_web_site_sk = sk32(ws_web_site_sk, 13, WebSalesRow::get_ws_web_site_sk);
        let ws_ship_mode_sk = sk32(ws_ship_mode_sk, 14, WebSalesRow::get_ws_ship_mode_sk);
        let ws_warehouse_sk = sk32(ws_warehouse_sk, 15, WebSalesRow::get_ws_warehouse_sk);
        let ws_promo_sk = sk32(ws_promo_sk, 16, WebSalesRow::get_ws_promo_sk);
        let ws_order_number =
            ws_order_number.int64(rows, 17, nbm, WebSalesRow::get_ws_order_number);
        let ws_quantity = ws_quantity.int32(rows, 18, nbm, |r| r.get_ws_pricing().get_quantity());
        let ws_wholesale_cost = dec(ws_wholesale_cost, 19, Pricing::get_wholesale_cost);
        let ws_list_price = dec(ws_list_price, 20, Pricing::get_list_price);
        let ws_sales_price = dec(ws_sales_price, 21, Pricing::get_sales_price);
        let ws_ext_discount_amt = dec(ws_ext_discount_amt, 22, Pricing::get_ext_discount_amount);
        let ws_ext_sales_price = dec(ws_ext_sales_price, 23, Pricing::get_ext_sales_price);
        let ws_ext_wholesale_cost = dec(ws_ext_wholesale_cost, 24, Pricing::get_ext_wholesale_cost);
        let ws_ext_list_price = dec(ws_ext_list_price, 25, Pricing::get_ext_list_price);
        let ws_ext_tax = dec(ws_ext_tax, 26, Pricing::get_ext_tax);
        let ws_coupon_amt = dec(ws_coupon_amt, 27, Pricing::get_coupon_amount);
        let ws_ext_ship_cost = dec(ws_ext_ship_cost, 28, Pricing::get_ext_ship_cost);
        let ws_net_paid = dec(ws_net_paid, 29, Pricing::get_net_paid);
        let ws_net_paid_inc_tax = dec(ws_net_paid_inc_tax, 30, Pricing::get_net_paid_including_tax);
        let ws_net_paid_inc_ship = dec(
            ws_net_paid_inc_ship,
            31,
            Pricing::get_net_paid_including_shipping,
        );
        let ws_net_paid_inc_ship_tax = dec(
            ws_net_paid_inc_ship_tax,
            32,
            Pricing::get_net_paid_including_shipping_and_tax,
        );
        let ws_net_profit = dec(ws_net_profit, 33, Pricing::get_net_profit);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(ws_sold_date_sk),
                Arc::new(ws_sold_time_sk),
                Arc::new(ws_ship_date_sk),
                Arc::new(ws_item_sk),
                Arc::new(ws_bill_customer_sk),
                Arc::new(ws_bill_cdemo_sk),
                Arc::new(ws_bill_hdemo_sk),
                Arc::new(ws_bill_addr_sk),
                Arc::new(ws_ship_customer_sk),
                Arc::new(ws_ship_cdemo_sk),
                Arc::new(ws_ship_hdemo_sk),
                Arc::new(ws_ship_addr_sk),
                Arc::new(ws_web_page_sk),
                Arc::new(ws_web_site_sk),
                Arc::new(ws_ship_mode_sk),
                Arc::new(ws_warehouse_sk),
                Arc::new(ws_promo_sk),
                Arc::new(ws_order_number),
                Arc::new(ws_quantity),
                Arc::new(ws_wholesale_cost),
                Arc::new(ws_list_price),
                Arc::new(ws_sales_price),
                Arc::new(ws_ext_discount_amt),
                Arc::new(ws_ext_sales_price),
                Arc::new(ws_ext_wholesale_cost),
                Arc::new(ws_ext_list_price),
                Arc::new(ws_ext_tax),
                Arc::new(ws_coupon_amt),
                Arc::new(ws_ext_ship_cost),
                Arc::new(ws_net_paid),
                Arc::new(ws_net_paid_inc_tax),
                Arc::new(ws_net_paid_inc_ship),
                Arc::new(ws_net_paid_inc_ship_tax),
                Arc::new(ws_net_profit),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("ws_sold_date_sk", DataType::Int32, true),
        Field::new("ws_sold_time_sk", DataType::Int32, true),
        Field::new("ws_ship_date_sk", DataType::Int32, true),
        Field::new("ws_item_sk", DataType::Int32, false),
        Field::new("ws_bill_customer_sk", DataType::Int32, true),
        Field::new("ws_bill_cdemo_sk", DataType::Int32, true),
        Field::new("ws_bill_hdemo_sk", DataType::Int32, true),
        Field::new("ws_bill_addr_sk", DataType::Int32, true),
        Field::new("ws_ship_customer_sk", DataType::Int32, true),
        Field::new("ws_ship_cdemo_sk", DataType::Int32, true),
        Field::new("ws_ship_hdemo_sk", DataType::Int32, true),
        Field::new("ws_ship_addr_sk", DataType::Int32, true),
        Field::new("ws_web_page_sk", DataType::Int32, true),
        Field::new("ws_web_site_sk", DataType::Int32, true),
        Field::new("ws_ship_mode_sk", DataType::Int32, true),
        Field::new("ws_warehouse_sk", DataType::Int32, true),
        Field::new("ws_promo_sk", DataType::Int32, true),
        Field::new("ws_order_number", DataType::Int64, false),
        Field::new("ws_quantity", DataType::Int32, true),
        Field::new("ws_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("ws_list_price", DataType::Decimal128(7, 2), true),
        Field::new("ws_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_discount_amt", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_list_price", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_tax", DataType::Decimal128(7, 2), true),
        Field::new("ws_coupon_amt", DataType::Decimal128(7, 2), true),
        Field::new("ws_ext_ship_cost", DataType::Decimal128(7, 2), true),
        Field::new("ws_net_paid", DataType::Decimal128(7, 2), true),
        Field::new("ws_net_paid_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("ws_net_paid_inc_ship", DataType::Decimal128(7, 2), true),
        Field::new("ws_net_paid_inc_ship_tax", DataType::Decimal128(7, 2), true),
        Field::new("ws_net_profit", DataType::Decimal128(7, 2), true),
    ]))
}
