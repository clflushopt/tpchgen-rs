use crate::conversions::PrimitiveArrayRecycler;
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Decimal128Type, Field, Int32Type, Int64Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{CatalogSalesRow, CatalogSalesRowGenerator};
use tpcdsgen::types::{Decimal, Pricing};

/// Generate [`CatalogSalesRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct CatalogSalesArrow {
    inner: CatalogSalesRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<CatalogSalesRow>,
    cs_sold_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_sold_time_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_date_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_bill_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_bill_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_bill_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_bill_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_customer_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_cdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_hdemo_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_addr_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_call_center_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_catalog_page_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_ship_mode_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_warehouse_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_item_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_promo_sk: PrimitiveArrayRecycler<Int32Type>,
    cs_order_number: PrimitiveArrayRecycler<Int64Type>,
    cs_quantity: PrimitiveArrayRecycler<Int32Type>,
    cs_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    cs_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    cs_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_discount_amt: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_sales_price: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_list_price: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_tax: PrimitiveArrayRecycler<Decimal128Type>,
    cs_coupon_amt: PrimitiveArrayRecycler<Decimal128Type>,
    cs_ext_ship_cost: PrimitiveArrayRecycler<Decimal128Type>,
    cs_net_paid: PrimitiveArrayRecycler<Decimal128Type>,
    cs_net_paid_inc_tax: PrimitiveArrayRecycler<Decimal128Type>,
    cs_net_paid_inc_ship: PrimitiveArrayRecycler<Decimal128Type>,
    cs_net_paid_inc_ship_tax: PrimitiveArrayRecycler<Decimal128Type>,
    cs_net_profit: PrimitiveArrayRecycler<Decimal128Type>,
}

impl CatalogSalesArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::CatalogSales);
        Self {
            inner: CatalogSalesRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            cs_sold_date_sk: PrimitiveArrayRecycler::new(),
            cs_sold_time_sk: PrimitiveArrayRecycler::new(),
            cs_ship_date_sk: PrimitiveArrayRecycler::new(),
            cs_bill_customer_sk: PrimitiveArrayRecycler::new(),
            cs_bill_cdemo_sk: PrimitiveArrayRecycler::new(),
            cs_bill_hdemo_sk: PrimitiveArrayRecycler::new(),
            cs_bill_addr_sk: PrimitiveArrayRecycler::new(),
            cs_ship_customer_sk: PrimitiveArrayRecycler::new(),
            cs_ship_cdemo_sk: PrimitiveArrayRecycler::new(),
            cs_ship_hdemo_sk: PrimitiveArrayRecycler::new(),
            cs_ship_addr_sk: PrimitiveArrayRecycler::new(),
            cs_call_center_sk: PrimitiveArrayRecycler::new(),
            cs_catalog_page_sk: PrimitiveArrayRecycler::new(),
            cs_ship_mode_sk: PrimitiveArrayRecycler::new(),
            cs_warehouse_sk: PrimitiveArrayRecycler::new(),
            cs_item_sk: PrimitiveArrayRecycler::new(),
            cs_promo_sk: PrimitiveArrayRecycler::new(),
            cs_order_number: PrimitiveArrayRecycler::new(),
            cs_quantity: PrimitiveArrayRecycler::new(),
            cs_wholesale_cost: PrimitiveArrayRecycler::new(),
            cs_list_price: PrimitiveArrayRecycler::new(),
            cs_sales_price: PrimitiveArrayRecycler::new(),
            cs_ext_discount_amt: PrimitiveArrayRecycler::new(),
            cs_ext_sales_price: PrimitiveArrayRecycler::new(),
            cs_ext_wholesale_cost: PrimitiveArrayRecycler::new(),
            cs_ext_list_price: PrimitiveArrayRecycler::new(),
            cs_ext_tax: PrimitiveArrayRecycler::new(),
            cs_coupon_amt: PrimitiveArrayRecycler::new(),
            cs_ext_ship_cost: PrimitiveArrayRecycler::new(),
            cs_net_paid: PrimitiveArrayRecycler::new(),
            cs_net_paid_inc_tax: PrimitiveArrayRecycler::new(),
            cs_net_paid_inc_ship: PrimitiveArrayRecycler::new(),
            cs_net_paid_inc_ship_tax: PrimitiveArrayRecycler::new(),
            cs_net_profit: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for CatalogSalesArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for CatalogSalesArrow {
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
            cs_sold_date_sk,
            cs_sold_time_sk,
            cs_ship_date_sk,
            cs_bill_customer_sk,
            cs_bill_cdemo_sk,
            cs_bill_hdemo_sk,
            cs_bill_addr_sk,
            cs_ship_customer_sk,
            cs_ship_cdemo_sk,
            cs_ship_hdemo_sk,
            cs_ship_addr_sk,
            cs_call_center_sk,
            cs_catalog_page_sk,
            cs_ship_mode_sk,
            cs_warehouse_sk,
            cs_item_sk,
            cs_promo_sk,
            cs_order_number,
            cs_quantity,
            cs_wholesale_cost,
            cs_list_price,
            cs_sales_price,
            cs_ext_discount_amt,
            cs_ext_sales_price,
            cs_ext_wholesale_cost,
            cs_ext_list_price,
            cs_ext_tax,
            cs_coupon_amt,
            cs_ext_ship_cost,
            cs_net_paid,
            cs_net_paid_inc_tax,
            cs_net_paid_inc_ship,
            cs_net_paid_inc_ship_tax,
            cs_net_profit,
            ..
        } = self;
        let nbm = CatalogSalesRow::null_bit_map;
        let sk32 =
            |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&CatalogSalesRow) -> i64| {
                rec.int32_sk(rows, pos, nbm, f)
            };
        let dec =
            |rec: &mut PrimitiveArrayRecycler<Decimal128Type>, pos, f: fn(&Pricing) -> Decimal| {
                rec.decimal_7_2(rows, pos, nbm, |r| f(r.get_cs_pricing()))
            };

        let cs_sold_date_sk = sk32(cs_sold_date_sk, 0, CatalogSalesRow::get_cs_sold_date_sk);
        let cs_sold_time_sk = sk32(cs_sold_time_sk, 1, CatalogSalesRow::get_cs_sold_time_sk);
        let cs_ship_date_sk = sk32(cs_ship_date_sk, 2, CatalogSalesRow::get_cs_ship_date_sk);
        let cs_bill_customer_sk = sk32(
            cs_bill_customer_sk,
            3,
            CatalogSalesRow::get_cs_bill_customer_sk,
        );
        let cs_bill_cdemo_sk = sk32(cs_bill_cdemo_sk, 4, CatalogSalesRow::get_cs_bill_cdemo_sk);
        let cs_bill_hdemo_sk = sk32(cs_bill_hdemo_sk, 5, CatalogSalesRow::get_cs_bill_hdemo_sk);
        let cs_bill_addr_sk = sk32(cs_bill_addr_sk, 6, CatalogSalesRow::get_cs_bill_addr_sk);
        let cs_ship_customer_sk = sk32(
            cs_ship_customer_sk,
            7,
            CatalogSalesRow::get_cs_ship_customer_sk,
        );
        let cs_ship_cdemo_sk = sk32(cs_ship_cdemo_sk, 8, CatalogSalesRow::get_cs_ship_cdemo_sk);
        let cs_ship_hdemo_sk = sk32(cs_ship_hdemo_sk, 9, CatalogSalesRow::get_cs_ship_hdemo_sk);
        let cs_ship_addr_sk = sk32(cs_ship_addr_sk, 10, CatalogSalesRow::get_cs_ship_addr_sk);
        let cs_call_center_sk = sk32(
            cs_call_center_sk,
            11,
            CatalogSalesRow::get_cs_call_center_sk,
        );
        let cs_catalog_page_sk = sk32(
            cs_catalog_page_sk,
            12,
            CatalogSalesRow::get_cs_catalog_page_sk,
        );
        let cs_ship_mode_sk = sk32(cs_ship_mode_sk, 13, CatalogSalesRow::get_cs_ship_mode_sk);
        let cs_warehouse_sk = sk32(cs_warehouse_sk, 14, CatalogSalesRow::get_cs_warehouse_sk);
        let cs_item_sk = sk32(cs_item_sk, 15, CatalogSalesRow::get_cs_sold_item_sk);
        let cs_promo_sk = sk32(cs_promo_sk, 16, CatalogSalesRow::get_cs_promo_sk);
        let cs_order_number =
            cs_order_number.int64(rows, 17, nbm, CatalogSalesRow::get_cs_order_number);
        let cs_quantity = cs_quantity.int32(rows, 18, nbm, |r| r.get_cs_pricing().get_quantity());
        let cs_wholesale_cost = dec(cs_wholesale_cost, 19, Pricing::get_wholesale_cost);
        let cs_list_price = dec(cs_list_price, 20, Pricing::get_list_price);
        let cs_sales_price = dec(cs_sales_price, 21, Pricing::get_sales_price);
        let cs_ext_discount_amt = dec(cs_ext_discount_amt, 24, Pricing::get_ext_discount_amount);
        let cs_ext_sales_price = dec(cs_ext_sales_price, 23, Pricing::get_ext_sales_price);
        let cs_ext_wholesale_cost = dec(cs_ext_wholesale_cost, 25, Pricing::get_ext_wholesale_cost);
        let cs_ext_list_price = dec(cs_ext_list_price, 26, Pricing::get_ext_list_price);
        let cs_ext_tax = dec(cs_ext_tax, 27, Pricing::get_ext_tax);
        let cs_coupon_amt = dec(cs_coupon_amt, 22, Pricing::get_coupon_amount);
        let cs_ext_ship_cost = dec(cs_ext_ship_cost, 28, Pricing::get_ext_ship_cost);
        let cs_net_paid = dec(cs_net_paid, 29, Pricing::get_net_paid);
        let cs_net_paid_inc_tax = dec(cs_net_paid_inc_tax, 30, Pricing::get_net_paid_including_tax);
        let cs_net_paid_inc_ship = dec(
            cs_net_paid_inc_ship,
            31,
            Pricing::get_net_paid_including_shipping,
        );
        let cs_net_paid_inc_ship_tax = dec(
            cs_net_paid_inc_ship_tax,
            32,
            Pricing::get_net_paid_including_shipping_and_tax,
        );
        let cs_net_profit = dec(cs_net_profit, 33, Pricing::get_net_profit);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(cs_sold_date_sk),
                Arc::new(cs_sold_time_sk),
                Arc::new(cs_ship_date_sk),
                Arc::new(cs_bill_customer_sk),
                Arc::new(cs_bill_cdemo_sk),
                Arc::new(cs_bill_hdemo_sk),
                Arc::new(cs_bill_addr_sk),
                Arc::new(cs_ship_customer_sk),
                Arc::new(cs_ship_cdemo_sk),
                Arc::new(cs_ship_hdemo_sk),
                Arc::new(cs_ship_addr_sk),
                Arc::new(cs_call_center_sk),
                Arc::new(cs_catalog_page_sk),
                Arc::new(cs_ship_mode_sk),
                Arc::new(cs_warehouse_sk),
                Arc::new(cs_item_sk),
                Arc::new(cs_promo_sk),
                Arc::new(cs_order_number),
                Arc::new(cs_quantity),
                Arc::new(cs_wholesale_cost),
                Arc::new(cs_list_price),
                Arc::new(cs_sales_price),
                Arc::new(cs_ext_discount_amt),
                Arc::new(cs_ext_sales_price),
                Arc::new(cs_ext_wholesale_cost),
                Arc::new(cs_ext_list_price),
                Arc::new(cs_ext_tax),
                Arc::new(cs_coupon_amt),
                Arc::new(cs_ext_ship_cost),
                Arc::new(cs_net_paid),
                Arc::new(cs_net_paid_inc_tax),
                Arc::new(cs_net_paid_inc_ship),
                Arc::new(cs_net_paid_inc_ship_tax),
                Arc::new(cs_net_profit),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("cs_sold_date_sk", DataType::Int32, true),
        Field::new("cs_sold_time_sk", DataType::Int32, true),
        Field::new("cs_ship_date_sk", DataType::Int32, true),
        Field::new("cs_bill_customer_sk", DataType::Int32, true),
        Field::new("cs_bill_cdemo_sk", DataType::Int32, true),
        Field::new("cs_bill_hdemo_sk", DataType::Int32, true),
        Field::new("cs_bill_addr_sk", DataType::Int32, true),
        Field::new("cs_ship_customer_sk", DataType::Int32, true),
        Field::new("cs_ship_cdemo_sk", DataType::Int32, true),
        Field::new("cs_ship_hdemo_sk", DataType::Int32, true),
        Field::new("cs_ship_addr_sk", DataType::Int32, true),
        Field::new("cs_call_center_sk", DataType::Int32, true),
        Field::new("cs_catalog_page_sk", DataType::Int32, true),
        Field::new("cs_ship_mode_sk", DataType::Int32, true),
        Field::new("cs_warehouse_sk", DataType::Int32, true),
        Field::new("cs_item_sk", DataType::Int32, false),
        Field::new("cs_promo_sk", DataType::Int32, true),
        Field::new("cs_order_number", DataType::Int64, false),
        Field::new("cs_quantity", DataType::Int32, true),
        Field::new("cs_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("cs_list_price", DataType::Decimal128(7, 2), true),
        Field::new("cs_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_discount_amt", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_sales_price", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_list_price", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_tax", DataType::Decimal128(7, 2), true),
        Field::new("cs_coupon_amt", DataType::Decimal128(7, 2), true),
        Field::new("cs_ext_ship_cost", DataType::Decimal128(7, 2), true),
        Field::new("cs_net_paid", DataType::Decimal128(7, 2), true),
        Field::new("cs_net_paid_inc_tax", DataType::Decimal128(7, 2), true),
        Field::new("cs_net_paid_inc_ship", DataType::Decimal128(7, 2), true),
        Field::new("cs_net_paid_inc_ship_tax", DataType::Decimal128(7, 2), true),
        Field::new("cs_net_profit", DataType::Decimal128(7, 2), true),
    ]))
}
