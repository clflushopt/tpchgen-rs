use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, WebSalesRowGenerator};

pub struct WebSalesArrow {
    generator: WebSalesRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl WebSalesArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSales);
        Self { generator: WebSalesRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for WebSalesArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for WebSalesArrow {
    type Item = RecordBatch;

    fn next(&mut self) -> Option<RecordBatch> {
        if self.current_row > self.row_count { return None; }

        let mut ws_sold_date: Vec<Option<i64>> = Vec::new();
        let mut ws_sold_time: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_date: Vec<Option<i64>> = Vec::new();
        let mut ws_item: Vec<Option<i64>> = Vec::new();
        let mut ws_bill_customer: Vec<Option<i64>> = Vec::new();
        let mut ws_bill_cdemo: Vec<Option<i64>> = Vec::new();
        let mut ws_bill_hdemo: Vec<Option<i64>> = Vec::new();
        let mut ws_bill_addr: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_customer: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_cdemo: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_hdemo: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_addr: Vec<Option<i64>> = Vec::new();
        let mut ws_web_page: Vec<Option<i64>> = Vec::new();
        let mut ws_web_site: Vec<Option<i64>> = Vec::new();
        let mut ws_ship_mode: Vec<Option<i64>> = Vec::new();
        let mut ws_warehouse: Vec<Option<i64>> = Vec::new();
        let mut ws_promo: Vec<Option<i64>> = Vec::new();
        let mut ws_order_number: Vec<Option<i64>> = Vec::new();
        let mut ws_quantity: Vec<Option<i32>> = Vec::new();
        let mut ws_wholesale_cost: Vec<Option<i128>> = Vec::new();
        let mut ws_list_price: Vec<Option<i128>> = Vec::new();
        let mut ws_sales_price: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_discount_amt: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_sales_price: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_wholesale_cost: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_list_price: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_tax: Vec<Option<i128>> = Vec::new();
        let mut ws_coupon_amt: Vec<Option<i128>> = Vec::new();
        let mut ws_ext_ship_cost: Vec<Option<i128>> = Vec::new();
        let mut ws_net_paid: Vec<Option<i128>> = Vec::new();
        let mut ws_net_paid_inc_tax: Vec<Option<i128>> = Vec::new();
        let mut ws_net_paid_inc_ship: Vec<Option<i128>> = Vec::new();
        let mut ws_net_paid_inc_ship_tax: Vec<Option<i128>> = Vec::new();
        let mut ws_net_profit: Vec<Option<i128>> = Vec::new();

        while self.current_row <= self.row_count && ws_sold_date.len() < self.batch_size {
            let result = self.generator.generate_row_and_child_rows(self.current_row, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::WebSales(r) = g {
                    let nbm = r.null_bit_map();
                    let p = r.get_ws_pricing();
                    ws_sold_date.push(sk_opt(nbm, 0, r.get_ws_sold_date_sk()));
                    ws_sold_time.push(sk_opt(nbm, 1, r.get_ws_sold_time_sk()));
                    ws_ship_date.push(sk_opt(nbm, 2, r.get_ws_ship_date_sk()));
                    ws_item.push(sk_opt(nbm, 3, r.get_ws_item_sk()));
                    ws_bill_customer.push(sk_opt(nbm, 4, r.get_ws_bill_customer_sk()));
                    ws_bill_cdemo.push(sk_opt(nbm, 5, r.get_ws_bill_cdemo_sk()));
                    ws_bill_hdemo.push(sk_opt(nbm, 6, r.get_ws_bill_hdemo_sk()));
                    ws_bill_addr.push(sk_opt(nbm, 7, r.get_ws_bill_addr_sk()));
                    ws_ship_customer.push(sk_opt(nbm, 8, r.get_ws_ship_customer_sk()));
                    ws_ship_cdemo.push(sk_opt(nbm, 9, r.get_ws_ship_cdemo_sk()));
                    ws_ship_hdemo.push(sk_opt(nbm, 10, r.get_ws_ship_hdemo_sk()));
                    ws_ship_addr.push(sk_opt(nbm, 11, r.get_ws_ship_addr_sk()));
                    ws_web_page.push(sk_opt(nbm, 12, r.get_ws_web_page_sk()));
                    ws_web_site.push(sk_opt(nbm, 13, r.get_ws_web_site_sk()));
                    ws_ship_mode.push(sk_opt(nbm, 14, r.get_ws_ship_mode_sk()));
                    ws_warehouse.push(sk_opt(nbm, 15, r.get_ws_warehouse_sk()));
                    ws_promo.push(sk_opt(nbm, 16, r.get_ws_promo_sk()));
                    ws_order_number.push(opt(nbm, 17, r.get_ws_order_number()));
                    ws_quantity.push(opt(nbm, 18, p.get_quantity()));
                    ws_wholesale_cost.push(opt(nbm, 19, decimal_to_i128(p.get_wholesale_cost())));
                    ws_list_price.push(opt(nbm, 20, decimal_to_i128(p.get_list_price())));
                    ws_sales_price.push(opt(nbm, 21, decimal_to_i128(p.get_sales_price())));
                    ws_ext_discount_amt.push(opt(nbm, 22, decimal_to_i128(p.get_ext_discount_amount())));
                    ws_ext_sales_price.push(opt(nbm, 23, decimal_to_i128(p.get_ext_sales_price())));
                    ws_ext_wholesale_cost.push(opt(nbm, 24, decimal_to_i128(p.get_ext_wholesale_cost())));
                    ws_ext_list_price.push(opt(nbm, 25, decimal_to_i128(p.get_ext_list_price())));
                    ws_ext_tax.push(opt(nbm, 26, decimal_to_i128(p.get_ext_tax())));
                    ws_coupon_amt.push(opt(nbm, 27, decimal_to_i128(p.get_coupon_amount())));
                    ws_ext_ship_cost.push(opt(nbm, 28, decimal_to_i128(p.get_ext_ship_cost())));
                    ws_net_paid.push(opt(nbm, 29, decimal_to_i128(p.get_net_paid())));
                    ws_net_paid_inc_tax.push(opt(nbm, 30, decimal_to_i128(p.get_net_paid_including_tax())));
                    ws_net_paid_inc_ship.push(opt(nbm, 31, decimal_to_i128(p.get_net_paid_including_shipping())));
                    ws_net_paid_inc_ship_tax.push(opt(nbm, 32, decimal_to_i128(p.get_net_paid_including_shipping_and_tax())));
                    ws_net_profit.push(opt(nbm, 33, decimal_to_i128(p.get_net_profit())));
                }
            }
            if result.should_end_row() {
                self.generator.consume_remaining_seeds_for_row();
                self.generator.consume_child_seeds();
                self.current_row += 1;
            }
        }
        if ws_sold_date.is_empty() { return None; }

        let dec = |v: Vec<Option<i128>>| Decimal128Array::from(v).with_precision_and_scale(38, 2).unwrap();
        Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
            Arc::new(Int64Array::from(ws_sold_date)),
            Arc::new(Int64Array::from(ws_sold_time)),
            Arc::new(Int64Array::from(ws_ship_date)),
            Arc::new(Int64Array::from(ws_item)),
            Arc::new(Int64Array::from(ws_bill_customer)),
            Arc::new(Int64Array::from(ws_bill_cdemo)),
            Arc::new(Int64Array::from(ws_bill_hdemo)),
            Arc::new(Int64Array::from(ws_bill_addr)),
            Arc::new(Int64Array::from(ws_ship_customer)),
            Arc::new(Int64Array::from(ws_ship_cdemo)),
            Arc::new(Int64Array::from(ws_ship_hdemo)),
            Arc::new(Int64Array::from(ws_ship_addr)),
            Arc::new(Int64Array::from(ws_web_page)),
            Arc::new(Int64Array::from(ws_web_site)),
            Arc::new(Int64Array::from(ws_ship_mode)),
            Arc::new(Int64Array::from(ws_warehouse)),
            Arc::new(Int64Array::from(ws_promo)),
            Arc::new(Int64Array::from(ws_order_number)),
            Arc::new(Int32Array::from(ws_quantity)),
            Arc::new(dec(ws_wholesale_cost)),
            Arc::new(dec(ws_list_price)),
            Arc::new(dec(ws_sales_price)),
            Arc::new(dec(ws_ext_discount_amt)),
            Arc::new(dec(ws_ext_sales_price)),
            Arc::new(dec(ws_ext_wholesale_cost)),
            Arc::new(dec(ws_ext_list_price)),
            Arc::new(dec(ws_ext_tax)),
            Arc::new(dec(ws_coupon_amt)),
            Arc::new(dec(ws_ext_ship_cost)),
            Arc::new(dec(ws_net_paid)),
            Arc::new(dec(ws_net_paid_inc_tax)),
            Arc::new(dec(ws_net_paid_inc_ship)),
            Arc::new(dec(ws_net_paid_inc_ship_tax)),
            Arc::new(dec(ws_net_profit)),
        ]).unwrap())
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| Arc::new(Schema::new(vec![
    Field::new("ws_sold_date_sk", DataType::Int64, true),
    Field::new("ws_sold_time_sk", DataType::Int64, true),
    Field::new("ws_ship_date_sk", DataType::Int64, true),
    Field::new("ws_item_sk", DataType::Int64, true),
    Field::new("ws_bill_customer_sk", DataType::Int64, true),
    Field::new("ws_bill_cdemo_sk", DataType::Int64, true),
    Field::new("ws_bill_hdemo_sk", DataType::Int64, true),
    Field::new("ws_bill_addr_sk", DataType::Int64, true),
    Field::new("ws_ship_customer_sk", DataType::Int64, true),
    Field::new("ws_ship_cdemo_sk", DataType::Int64, true),
    Field::new("ws_ship_hdemo_sk", DataType::Int64, true),
    Field::new("ws_ship_addr_sk", DataType::Int64, true),
    Field::new("ws_web_page_sk", DataType::Int64, true),
    Field::new("ws_web_site_sk", DataType::Int64, true),
    Field::new("ws_ship_mode_sk", DataType::Int64, true),
    Field::new("ws_warehouse_sk", DataType::Int64, true),
    Field::new("ws_promo_sk", DataType::Int64, true),
    Field::new("ws_order_number", DataType::Int64, true),
    Field::new("ws_quantity", DataType::Int32, true),
    Field::new("ws_wholesale_cost", DataType::Decimal128(38, 2), true),
    Field::new("ws_list_price", DataType::Decimal128(38, 2), true),
    Field::new("ws_sales_price", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_discount_amt", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_sales_price", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_wholesale_cost", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_list_price", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_tax", DataType::Decimal128(38, 2), true),
    Field::new("ws_coupon_amt", DataType::Decimal128(38, 2), true),
    Field::new("ws_ext_ship_cost", DataType::Decimal128(38, 2), true),
    Field::new("ws_net_paid", DataType::Decimal128(38, 2), true),
    Field::new("ws_net_paid_inc_tax", DataType::Decimal128(38, 2), true),
    Field::new("ws_net_paid_inc_ship", DataType::Decimal128(38, 2), true),
    Field::new("ws_net_paid_inc_ship_tax", DataType::Decimal128(38, 2), true),
    Field::new("ws_net_profit", DataType::Decimal128(38, 2), true),
])));
