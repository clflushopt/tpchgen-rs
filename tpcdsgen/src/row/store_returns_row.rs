/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Store returns row data structure

use crate::generator::{GeneratorColumn, StoreReturnsGeneratorColumn};
use crate::row::TableRow;
use crate::types::Pricing;

/// Row data structure for the store_returns table
#[derive(Debug, Clone)]
pub struct StoreReturnsRow {
    null_bit_map: i64,
    sr_returned_date_sk: i64,
    sr_returned_time_sk: i64,
    sr_item_sk: i64,
    sr_customer_sk: i64,
    sr_cdemo_sk: i64,
    sr_hdemo_sk: i64,
    sr_addr_sk: i64,
    sr_store_sk: i64,
    sr_reason_sk: i64,
    sr_ticket_number: i64,
    sr_pricing: Pricing,
}

impl StoreReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        sr_returned_date_sk: i64,
        sr_returned_time_sk: i64,
        sr_item_sk: i64,
        sr_customer_sk: i64,
        sr_cdemo_sk: i64,
        sr_hdemo_sk: i64,
        sr_addr_sk: i64,
        sr_store_sk: i64,
        sr_reason_sk: i64,
        sr_ticket_number: i64,
        sr_pricing: Pricing,
    ) -> Self {
        StoreReturnsRow {
            null_bit_map,
            sr_returned_date_sk,
            sr_returned_time_sk,
            sr_item_sk,
            sr_customer_sk,
            sr_cdemo_sk,
            sr_hdemo_sk,
            sr_addr_sk,
            sr_store_sk,
            sr_reason_sk,
            sr_ticket_number,
            sr_pricing,
        }
    }

    fn get_string_or_null_for_key(&self, key: i64, column: StoreReturnsGeneratorColumn) -> String {
        if key == -1 || self.is_null_at(column) {
            String::new()
        } else {
            key.to_string()
        }
    }

    fn get_string_or_null_int(&self, value: i32, column: StoreReturnsGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_decimal(
        &self,
        value: &crate::types::Decimal,
        column: StoreReturnsGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn is_null_at(&self, column: StoreReturnsGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - StoreReturnsGeneratorColumn::SrReturnedDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }
}

impl TableRow for StoreReturnsRow {
    fn get_values(&self) -> Vec<String> {
        use StoreReturnsGeneratorColumn::*;

        vec![
            self.get_string_or_null_for_key(self.sr_returned_date_sk, SrReturnedDateSk),
            self.get_string_or_null_for_key(self.sr_returned_time_sk, SrReturnedTimeSk),
            self.get_string_or_null_for_key(self.sr_item_sk, SrItemSk),
            self.get_string_or_null_for_key(self.sr_customer_sk, SrCustomerSk),
            self.get_string_or_null_for_key(self.sr_cdemo_sk, SrCdemoSk),
            self.get_string_or_null_for_key(self.sr_hdemo_sk, SrHdemoSk),
            self.get_string_or_null_for_key(self.sr_addr_sk, SrAddrSk),
            self.get_string_or_null_for_key(self.sr_store_sk, SrStoreSk),
            self.get_string_or_null_for_key(self.sr_reason_sk, SrReasonSk),
            self.get_string_or_null_for_key(self.sr_ticket_number, SrTicketNumber),
            self.get_string_or_null_int(self.sr_pricing.get_quantity(), SrPricingQuantity),
            self.get_string_or_null_decimal(&self.sr_pricing.get_net_paid(), SrPricingNetPaid),
            self.get_string_or_null_decimal(&self.sr_pricing.get_ext_tax(), SrPricingExtTax),
            self.get_string_or_null_decimal(
                &self.sr_pricing.get_net_paid_including_tax(),
                SrPricingNetPaidIncTax,
            ),
            self.get_string_or_null_decimal(&self.sr_pricing.get_fee(), SrPricingFee),
            self.get_string_or_null_decimal(
                &self.sr_pricing.get_ext_ship_cost(),
                SrPricingExtShipCost,
            ),
            self.get_string_or_null_decimal(
                &self.sr_pricing.get_refunded_cash(),
                SrPricingRefundedCash,
            ),
            self.get_string_or_null_decimal(
                &self.sr_pricing.get_reversed_charge(),
                SrPricingReversedCharge,
            ),
            self.get_string_or_null_decimal(
                &self.sr_pricing.get_store_credit(),
                SrPricingStoreCredit,
            ),
            self.get_string_or_null_decimal(&self.sr_pricing.get_net_loss(), SrPricingNetLoss),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Decimal;

    fn create_test_pricing() -> Pricing {
        Pricing::new(
            Decimal::new(1000, 2).unwrap(), // wholesale_cost: 10.00
            Decimal::new(1500, 2).unwrap(), // list_price: 15.00
            Decimal::new(1200, 2).unwrap(), // sales_price: 12.00
            5,                              // quantity
            Decimal::new(300, 2).unwrap(),  // ext_discount_amount: 3.00
            Decimal::new(6000, 2).unwrap(), // ext_sales_price: 60.00
            Decimal::new(5000, 2).unwrap(), // ext_wholesale_cost: 50.00
            Decimal::new(7500, 2).unwrap(), // ext_list_price: 75.00
            Decimal::new(8, 2).unwrap(),    // tax_percent: 0.08
            Decimal::new(480, 2).unwrap(),  // ext_tax: 4.80
            Decimal::new(100, 2).unwrap(),  // coupon_amount: 1.00
            Decimal::new(200, 2).unwrap(),  // ship_cost: 2.00
            Decimal::new(1000, 2).unwrap(), // ext_ship_cost: 10.00
            Decimal::new(5900, 2).unwrap(), // net_paid: 59.00
            Decimal::new(6380, 2).unwrap(), // net_paid_including_tax: 63.80
            Decimal::new(6900, 2).unwrap(), // net_paid_including_shipping: 69.00
            Decimal::new(7380, 2).unwrap(), // net_paid_including_shipping_and_tax: 73.80
            Decimal::new(900, 2).unwrap(),  // net_profit: 9.00
            Decimal::new(2000, 2).unwrap(), // refunded_cash: 20.00
            Decimal::new(1000, 2).unwrap(), // reversed_charge: 10.00
            Decimal::new(2900, 2).unwrap(), // store_credit: 29.00
            Decimal::new(500, 2).unwrap(),  // fee: 5.00
            Decimal::new(1580, 2).unwrap(), // net_loss: 15.80
        )
    }

    #[test]
    fn test_store_returns_row_creation() {
        let pricing = create_test_pricing();
        let row = StoreReturnsRow::new(
            0,       // null_bit_map
            2451545, // sr_returned_date_sk
            36000,   // sr_returned_time_sk
            1,       // sr_item_sk
            100,     // sr_customer_sk
            200,     // sr_cdemo_sk
            300,     // sr_hdemo_sk
            400,     // sr_addr_sk
            500,     // sr_store_sk
            600,     // sr_reason_sk
            1,       // sr_ticket_number
            pricing,
        );

        let values = row.get_values();
        assert_eq!(values.len(), 20);
        assert_eq!(values[0], "2451545"); // sr_returned_date_sk
        assert_eq!(values[9], "1"); // sr_ticket_number
    }

    #[test]
    fn test_store_returns_row_null_handling() {
        let pricing = create_test_pricing();
        // Set bit for sr_returned_time_sk (position 1)
        let row = StoreReturnsRow::new(
            0b10, // null_bit_map - second bit set
            2451545, 36000, 1, 100, 200, 300, 400, 500, 600, 1, pricing,
        );

        let values = row.get_values();
        assert_eq!(values[0], "2451545"); // not null
        assert_eq!(values[1], ""); // null (bit 1 set)
    }
}
