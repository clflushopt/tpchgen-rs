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
use crate::row::TableRow;
use crate::types::Pricing;

/// Row structure for web_returns table
#[derive(Debug, Clone)]
pub struct WebReturnsRow {
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
