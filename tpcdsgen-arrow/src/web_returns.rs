use crate::conversions::{decimal_to_i128, opt, sk_opt};
use crate::{DEFAULT_BATCH_SIZE, RecordBatchIterator};
use arrow::array::{Decimal128Array, Int32Array, Int64Array, RecordBatch};
use arrow::error::ArrowError;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{GeneratedRow, RowGenerator, WebSalesRowGenerator};

pub struct WebReturnsArrow {
    generator: WebSalesRowGenerator,
    session: Session,
    row_count: i64,
    current_row: i64,
    batch_size: usize,
}

impl WebReturnsArrow {
    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSales);
        Self { generator: WebSalesRowGenerator::new(), session, row_count, current_row: 1, batch_size: DEFAULT_BATCH_SIZE }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self { self.batch_size = batch_size; self }
}

impl RecordBatchIterator for WebReturnsArrow {
    fn schema(&self) -> &SchemaRef { &SCHEMA }
}

impl Iterator for WebReturnsArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Result<RecordBatch, ArrowError>> {
        loop {
        if self.current_row > self.row_count { return None; }

        let mut wr_returned_date: Vec<Option<i64>> = Vec::new();
        let mut wr_returned_time: Vec<Option<i64>> = Vec::new();
        let mut wr_item: Vec<Option<i64>> = Vec::new();
        let mut wr_refunded_customer: Vec<Option<i64>> = Vec::new();
        let mut wr_refunded_cdemo: Vec<Option<i64>> = Vec::new();
        let mut wr_refunded_hdemo: Vec<Option<i64>> = Vec::new();
        let mut wr_refunded_addr: Vec<Option<i64>> = Vec::new();
        let mut wr_returning_customer: Vec<Option<i64>> = Vec::new();
        let mut wr_returning_cdemo: Vec<Option<i64>> = Vec::new();
        let mut wr_returning_hdemo: Vec<Option<i64>> = Vec::new();
        let mut wr_returning_addr: Vec<Option<i64>> = Vec::new();
        let mut wr_web_page: Vec<Option<i64>> = Vec::new();
        let mut wr_reason: Vec<Option<i64>> = Vec::new();
        let mut wr_order_number: Vec<Option<i64>> = Vec::new();
        let mut wr_quantity: Vec<Option<i32>> = Vec::new();
        let mut wr_return_amt: Vec<Option<i128>> = Vec::new();
        let mut wr_return_tax: Vec<Option<i128>> = Vec::new();
        let mut wr_return_amt_inc_tax: Vec<Option<i128>> = Vec::new();
        let mut wr_fee: Vec<Option<i128>> = Vec::new();
        let mut wr_return_ship_cost: Vec<Option<i128>> = Vec::new();
        let mut wr_refunded_cash: Vec<Option<i128>> = Vec::new();
        let mut wr_reversed_charge: Vec<Option<i128>> = Vec::new();
        let mut wr_store_credit: Vec<Option<i128>> = Vec::new();
        let mut wr_net_loss: Vec<Option<i128>> = Vec::new();

        while self.current_row <= self.row_count && wr_returned_date.len() < self.batch_size {
            let result = self.generator.generate_row_and_child_rows(self.current_row, &self.session, None, None).expect("row gen");
            for g in result.get_rows() {
                if let GeneratedRow::WebReturns(r) = g {
                    let nbm = r.null_bit_map();
                    let p = r.get_wr_pricing();
                    wr_returned_date.push(sk_opt(nbm, 0, r.get_wr_returned_date_sk()));
                    wr_returned_time.push(sk_opt(nbm, 1, r.get_wr_returned_time_sk()));
                    wr_item.push(sk_opt(nbm, 2, r.get_wr_item_sk()));
                    wr_refunded_customer.push(sk_opt(nbm, 3, r.get_wr_refunded_customer_sk()));
                    wr_refunded_cdemo.push(sk_opt(nbm, 4, r.get_wr_refunded_cdemo_sk()));
                    wr_refunded_hdemo.push(sk_opt(nbm, 5, r.get_wr_refunded_hdemo_sk()));
                    wr_refunded_addr.push(sk_opt(nbm, 6, r.get_wr_refunded_addr_sk()));
                    wr_returning_customer.push(sk_opt(nbm, 7, r.get_wr_returning_customer_sk()));
                    wr_returning_cdemo.push(sk_opt(nbm, 8, r.get_wr_returning_cdemo_sk()));
                    wr_returning_hdemo.push(sk_opt(nbm, 9, r.get_wr_returning_hdemo_sk()));
                    wr_returning_addr.push(sk_opt(nbm, 10, r.get_wr_returning_addr_sk()));
                    wr_web_page.push(sk_opt(nbm, 11, r.get_wr_web_page_sk()));
                    wr_reason.push(sk_opt(nbm, 12, r.get_wr_reason_sk()));
                    wr_order_number.push(opt(nbm, 13, r.get_wr_order_number()));
                    wr_quantity.push(opt(nbm, 14, p.get_quantity()));
                    wr_return_amt.push(opt(nbm, 15, decimal_to_i128(p.get_net_paid())));
                    wr_return_tax.push(opt(nbm, 16, decimal_to_i128(p.get_ext_tax())));
                    wr_return_amt_inc_tax.push(opt(nbm, 17, decimal_to_i128(p.get_net_paid_including_tax())));
                    wr_fee.push(opt(nbm, 18, decimal_to_i128(p.get_fee())));
                    wr_return_ship_cost.push(opt(nbm, 19, decimal_to_i128(p.get_ext_ship_cost())));
                    wr_refunded_cash.push(opt(nbm, 20, decimal_to_i128(p.get_refunded_cash())));
                    wr_reversed_charge.push(opt(nbm, 21, decimal_to_i128(p.get_reversed_charge())));
                    wr_store_credit.push(opt(nbm, 22, decimal_to_i128(p.get_store_credit())));
                    wr_net_loss.push(opt(nbm, 23, decimal_to_i128(p.get_net_loss())));
                }
            }
            if result.should_end_row() {
                self.generator.consume_remaining_seeds_for_row();
                self.generator.consume_child_seeds();
                self.current_row += 1;
            }
        }
        if !wr_returned_date.is_empty() {
            let dec = |v: Vec<Option<i128>>| Decimal128Array::from(v).with_precision_and_scale(38, 2).unwrap();
            return Some(RecordBatch::try_new(Arc::clone(self.schema()), vec![
                Arc::new(Int64Array::from(wr_returned_date)),
                Arc::new(Int64Array::from(wr_returned_time)),
                Arc::new(Int64Array::from(wr_item)),
                Arc::new(Int64Array::from(wr_refunded_customer)),
                Arc::new(Int64Array::from(wr_refunded_cdemo)),
                Arc::new(Int64Array::from(wr_refunded_hdemo)),
                Arc::new(Int64Array::from(wr_refunded_addr)),
                Arc::new(Int64Array::from(wr_returning_customer)),
                Arc::new(Int64Array::from(wr_returning_cdemo)),
                Arc::new(Int64Array::from(wr_returning_hdemo)),
                Arc::new(Int64Array::from(wr_returning_addr)),
                Arc::new(Int64Array::from(wr_web_page)),
                Arc::new(Int64Array::from(wr_reason)),
                Arc::new(Int64Array::from(wr_order_number)),
                Arc::new(Int32Array::from(wr_quantity)),
                Arc::new(dec(wr_return_amt)),
                Arc::new(dec(wr_return_tax)),
                Arc::new(dec(wr_return_amt_inc_tax)),
                Arc::new(dec(wr_fee)),
                Arc::new(dec(wr_return_ship_cost)),
                Arc::new(dec(wr_refunded_cash)),
                Arc::new(dec(wr_reversed_charge)),
                Arc::new(dec(wr_store_credit)),
                Arc::new(dec(wr_net_loss)),
            ]));
        }
        } // loop
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
    Field::new("wr_returned_date_sk", DataType::Int64, true),
    Field::new("wr_returned_time_sk", DataType::Int64, true),
    Field::new("wr_item_sk", DataType::Int64, true),
    Field::new("wr_refunded_customer_sk", DataType::Int64, true),
    Field::new("wr_refunded_cdemo_sk", DataType::Int64, true),
    Field::new("wr_refunded_hdemo_sk", DataType::Int64, true),
    Field::new("wr_refunded_addr_sk", DataType::Int64, true),
    Field::new("wr_returning_customer_sk", DataType::Int64, true),
    Field::new("wr_returning_cdemo_sk", DataType::Int64, true),
    Field::new("wr_returning_hdemo_sk", DataType::Int64, true),
    Field::new("wr_returning_addr_sk", DataType::Int64, true),
    Field::new("wr_web_page_sk", DataType::Int64, true),
    Field::new("wr_reason_sk", DataType::Int64, true),
    Field::new("wr_order_number", DataType::Int64, true),
    Field::new("wr_return_quantity", DataType::Int32, true),
    Field::new("wr_return_amt", DataType::Decimal128(38, 2), true),
    Field::new("wr_return_tax", DataType::Decimal128(38, 2), true),
    Field::new("wr_return_amt_inc_tax", DataType::Decimal128(38, 2), true),
    Field::new("wr_fee", DataType::Decimal128(38, 2), true),
    Field::new("wr_return_ship_cost", DataType::Decimal128(38, 2), true),
    Field::new("wr_refunded_cash", DataType::Decimal128(38, 2), true),
    Field::new("wr_reversed_charge", DataType::Decimal128(38, 2), true),
    Field::new("wr_store_credit", DataType::Decimal128(38, 2), true),
    Field::new("wr_net_loss", DataType::Decimal128(38, 2), true),
]))
}
