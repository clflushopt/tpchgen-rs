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

//! Web returns row definition

use crate::generator::{GeneratorColumn, WebReturnsGeneratorColumn};
use crate::row::table_row::DatField;
use crate::row::TableRow;
use crate::types::Pricing;
use std::fmt;

/// Row structure for web_returns table
#[derive(Debug, Clone)]
pub struct WebReturnsRow {
    null_bit_map: i64,
    pub(crate) wr_returned_date_sk: i64,
    pub(crate) wr_returned_time_sk: i64,
    pub(crate) wr_item_sk: i64,
    pub(crate) wr_refunded_customer_sk: i64,
    pub(crate) wr_refunded_cdemo_sk: i64,
    pub(crate) wr_refunded_hdemo_sk: i64,
    pub(crate) wr_refunded_addr_sk: i64,
    pub(crate) wr_returning_customer_sk: i64,
    pub(crate) wr_returning_cdemo_sk: i64,
    pub(crate) wr_returning_hdemo_sk: i64,
    pub(crate) wr_returning_addr_sk: i64,
    pub(crate) wr_web_page_sk: i64,
    pub(crate) wr_reason_sk: i64,
    pub(crate) wr_order_number: i64,
    pub(crate) wr_pricing: Pricing,
}

impl WebReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        wr_returned_date_sk: i64,
        wr_returned_time_sk: i64,
        wr_item_sk: i64,
        wr_refunded_customer_sk: i64,
        wr_refunded_cdemo_sk: i64,
        wr_refunded_hdemo_sk: i64,
        wr_refunded_addr_sk: i64,
        wr_returning_customer_sk: i64,
        wr_returning_cdemo_sk: i64,
        wr_returning_hdemo_sk: i64,
        wr_returning_addr_sk: i64,
        wr_web_page_sk: i64,
        wr_reason_sk: i64,
        wr_order_number: i64,
        wr_pricing: Pricing,
    ) -> Self {
        WebReturnsRow {
            null_bit_map,
            wr_returned_date_sk,
            wr_returned_time_sk,
            wr_item_sk,
            wr_refunded_customer_sk,
            wr_refunded_cdemo_sk,
            wr_refunded_hdemo_sk,
            wr_refunded_addr_sk,
            wr_returning_customer_sk,
            wr_returning_cdemo_sk,
            wr_returning_hdemo_sk,
            wr_returning_addr_sk,
            wr_web_page_sk,
            wr_reason_sk,
            wr_order_number,
            wr_pricing,
        }
    }

    fn is_null(&self, column: WebReturnsGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - WebReturnsGeneratorColumn::WrReturnedDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(&self, value: i64, column: WebReturnsGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null<T: std::fmt::Display>(
        &self,
        value: T,
        column: WebReturnsGeneratorColumn,
    ) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_wr_returned_date_sk(&self) -> i64 {
        self.wr_returned_date_sk
    }

    pub fn get_wr_returned_time_sk(&self) -> i64 {
        self.wr_returned_time_sk
    }

    pub fn get_wr_item_sk(&self) -> i64 {
        self.wr_item_sk
    }

    pub fn get_wr_refunded_customer_sk(&self) -> i64 {
        self.wr_refunded_customer_sk
    }

    pub fn get_wr_refunded_cdemo_sk(&self) -> i64 {
        self.wr_refunded_cdemo_sk
    }

    pub fn get_wr_refunded_hdemo_sk(&self) -> i64 {
        self.wr_refunded_hdemo_sk
    }

    pub fn get_wr_refunded_addr_sk(&self) -> i64 {
        self.wr_refunded_addr_sk
    }

    pub fn get_wr_returning_customer_sk(&self) -> i64 {
        self.wr_returning_customer_sk
    }

    pub fn get_wr_returning_cdemo_sk(&self) -> i64 {
        self.wr_returning_cdemo_sk
    }

    pub fn get_wr_returning_hdemo_sk(&self) -> i64 {
        self.wr_returning_hdemo_sk
    }

    pub fn get_wr_returning_addr_sk(&self) -> i64 {
        self.wr_returning_addr_sk
    }

    pub fn get_wr_web_page_sk(&self) -> i64 {
        self.wr_web_page_sk
    }

    pub fn get_wr_reason_sk(&self) -> i64 {
        self.wr_reason_sk
    }

    pub fn get_wr_order_number(&self) -> i64 {
        self.wr_order_number
    }

    pub fn get_wr_pricing(&self) -> &Pricing {
        &self.wr_pricing
    }
}

/// DAT field helper mirroring `get_string_or_null`/`get_string_or_null_for_key`
/// (web rows apply no key sentinel check, only the null bit).
impl WebReturnsRow {
    pub(crate) fn field<T>(&self, value: T, column: WebReturnsGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for WebReturnsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WebReturnsGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.wr_returned_date_sk, WrReturnedDateSk),
            self.field(self.wr_returned_time_sk, WrReturnedTimeSk),
            self.field(self.wr_item_sk, WrItemSk),
            self.field(self.wr_refunded_customer_sk, WrRefundedCustomerSk),
            self.field(self.wr_refunded_cdemo_sk, WrRefundedCdemoSk),
            self.field(self.wr_refunded_hdemo_sk, WrRefundedHdemoSk),
            self.field(self.wr_refunded_addr_sk, WrRefundedAddrSk),
            self.field(self.wr_returning_customer_sk, WrReturningCustomerSk),
            self.field(self.wr_returning_cdemo_sk, WrReturningCdemoSk),
            self.field(self.wr_returning_hdemo_sk, WrReturningHdemoSk),
            self.field(self.wr_returning_addr_sk, WrReturningAddrSk),
            self.field(self.wr_web_page_sk, WrWebPageSk),
            self.field(self.wr_reason_sk, WrReasonSk),
            self.field(self.wr_order_number, WrOrderNumber),
            self.field(self.wr_pricing.get_quantity(), WrPricingQuantity),
            self.field(self.wr_pricing.get_net_paid(), WrPricingNetPaid),
            self.field(self.wr_pricing.get_ext_tax(), WrPricingExtTax),
            self.field(
                self.wr_pricing.get_net_paid_including_tax(),
                WrPricingNetPaidIncTax
            ),
            self.field(self.wr_pricing.get_fee(), WrPricingFee),
            self.field(self.wr_pricing.get_ext_ship_cost(), WrPricingExtShipCost),
            self.field(self.wr_pricing.get_refunded_cash(), WrPricingRefundedCash),
            self.field(
                self.wr_pricing.get_reversed_charge(),
                WrPricingReversedCharge
            ),
            self.field(self.wr_pricing.get_store_credit(), WrPricingStoreCredit),
            self.field(self.wr_pricing.get_net_loss(), WrPricingNetLoss),
        )
    }
}

impl TableRow for WebReturnsRow {
    fn get_values(&self) -> Vec<String> {
        use WebReturnsGeneratorColumn::*;
        vec![
            self.get_string_or_null_for_key(self.wr_returned_date_sk, WrReturnedDateSk),
            self.get_string_or_null_for_key(self.wr_returned_time_sk, WrReturnedTimeSk),
            self.get_string_or_null_for_key(self.wr_item_sk, WrItemSk),
            self.get_string_or_null_for_key(self.wr_refunded_customer_sk, WrRefundedCustomerSk),
            self.get_string_or_null_for_key(self.wr_refunded_cdemo_sk, WrRefundedCdemoSk),
            self.get_string_or_null_for_key(self.wr_refunded_hdemo_sk, WrRefundedHdemoSk),
            self.get_string_or_null_for_key(self.wr_refunded_addr_sk, WrRefundedAddrSk),
            self.get_string_or_null_for_key(self.wr_returning_customer_sk, WrReturningCustomerSk),
            self.get_string_or_null_for_key(self.wr_returning_cdemo_sk, WrReturningCdemoSk),
            self.get_string_or_null_for_key(self.wr_returning_hdemo_sk, WrReturningHdemoSk),
            self.get_string_or_null_for_key(self.wr_returning_addr_sk, WrReturningAddrSk),
            self.get_string_or_null_for_key(self.wr_web_page_sk, WrWebPageSk),
            self.get_string_or_null_for_key(self.wr_reason_sk, WrReasonSk),
            self.get_string_or_null_for_key(self.wr_order_number, WrOrderNumber),
            self.get_string_or_null(self.wr_pricing.get_quantity(), WrPricingQuantity),
            self.get_string_or_null(self.wr_pricing.get_net_paid(), WrPricingNetPaid),
            self.get_string_or_null(self.wr_pricing.get_ext_tax(), WrPricingExtTax),
            self.get_string_or_null(
                self.wr_pricing.get_net_paid_including_tax(),
                WrPricingNetPaidIncTax,
            ),
            self.get_string_or_null(self.wr_pricing.get_fee(), WrPricingFee),
            self.get_string_or_null(self.wr_pricing.get_ext_ship_cost(), WrPricingExtShipCost),
            self.get_string_or_null(self.wr_pricing.get_refunded_cash(), WrPricingRefundedCash),
            self.get_string_or_null(
                self.wr_pricing.get_reversed_charge(),
                WrPricingReversedCharge,
            ),
            self.get_string_or_null(self.wr_pricing.get_store_credit(), WrPricingStoreCredit),
            self.get_string_or_null(self.wr_pricing.get_net_loss(), WrPricingNetLoss),
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
    fn test_display_matches_get_values() {
        // Null bit on wr_returned_time_sk; web keys have no -1 sentinel handling.
        let row = WebReturnsRow::new(
            0b10,
            2451545,
            36000,
            1000,
            100,
            200,
            300,
            400,
            101,
            201,
            301,
            401,
            10,
            6,
            42,
            create_test_pricing(),
        );

        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
