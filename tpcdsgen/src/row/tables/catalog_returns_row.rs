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

//! Catalog returns row structure

use crate::generator::CatalogReturnsGeneratorColumn;
use crate::output::Line;
use crate::row::TableRow;
use crate::types::Pricing;

/// Row structure for catalog_returns table
#[derive(Debug, Clone)]
pub struct CatalogReturnsRow {
    null_bit_map: i64,
    cr_returned_date_sk: i64,
    cr_returned_time_sk: i64,
    cr_item_sk: i64,
    cr_refunded_customer_sk: i64,
    cr_refunded_cdemo_sk: i64,
    cr_refunded_hdemo_sk: i64,
    cr_refunded_addr_sk: i64,
    cr_returning_customer_sk: i64,
    cr_returning_cdemo_sk: i64,
    cr_returning_hdemo_sk: i64,
    cr_returning_addr_sk: i64,
    cr_call_center_sk: i64,
    cr_catalog_page_sk: i64,
    cr_ship_mode_sk: i64,
    cr_warehouse_sk: i64,
    cr_reason_sk: i64,
    cr_order_number: i64,
    cr_pricing: Pricing,
}

impl CatalogReturnsRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cr_returned_date_sk: i64,
        cr_returned_time_sk: i64,
        cr_item_sk: i64,
        cr_refunded_customer_sk: i64,
        cr_refunded_cdemo_sk: i64,
        cr_refunded_hdemo_sk: i64,
        cr_refunded_addr_sk: i64,
        cr_returning_customer_sk: i64,
        cr_returning_cdemo_sk: i64,
        cr_returning_hdemo_sk: i64,
        cr_returning_addr_sk: i64,
        cr_call_center_sk: i64,
        cr_catalog_page_sk: i64,
        cr_ship_mode_sk: i64,
        cr_warehouse_sk: i64,
        cr_reason_sk: i64,
        cr_order_number: i64,
        cr_pricing: Pricing,
    ) -> Self {
        CatalogReturnsRow {
            null_bit_map,
            cr_returned_date_sk,
            cr_returned_time_sk,
            cr_item_sk,
            cr_refunded_customer_sk,
            cr_refunded_cdemo_sk,
            cr_refunded_hdemo_sk,
            cr_refunded_addr_sk,
            cr_returning_customer_sk,
            cr_returning_cdemo_sk,
            cr_returning_hdemo_sk,
            cr_returning_addr_sk,
            cr_call_center_sk,
            cr_catalog_page_sk,
            cr_ship_mode_sk,
            cr_warehouse_sk,
            cr_reason_sk,
            cr_order_number,
            cr_pricing,
        }
    }

    fn is_null(&self, column: &CatalogReturnsGeneratorColumn) -> bool {
        let column_number = column.get_global_column_number();
        let first_column =
            CatalogReturnsGeneratorColumn::CrReturnedDateSk.get_global_column_number();
        let bit_position = column_number - first_column;
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(
        &self,
        value: i64,
        column: &CatalogReturnsGeneratorColumn,
    ) -> String {
        if self.is_null(column) || value < 0 {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null<T: ToString>(
        &self,
        value: T,
        column: &CatalogReturnsGeneratorColumn,
    ) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn push_key(
        &self,
        out: &mut Line,
        sep: &[u8],
        value: i64,
        column: &CatalogReturnsGeneratorColumn,
    ) {
        if !self.is_null(column) && value >= 0 {
            out.push_int(value);
        }
        out.push_bytes(sep);
    }

    fn push_int_field(
        &self,
        out: &mut Line,
        sep: &[u8],
        value: i64,
        column: &CatalogReturnsGeneratorColumn,
    ) {
        if !self.is_null(column) {
            out.push_int(value);
        }
        out.push_bytes(sep);
    }

    fn push_int(
        &self,
        out: &mut Line,
        sep: &[u8],
        value: i32,
        column: &CatalogReturnsGeneratorColumn,
    ) {
        if !self.is_null(column) {
            out.push_int(value);
        }
        out.push_bytes(sep);
    }

    fn push_decimal(
        &self,
        out: &mut Line,
        sep: &[u8],
        value: &crate::types::Decimal,
        column: &CatalogReturnsGeneratorColumn,
    ) {
        if !self.is_null(column) {
            value.append_dat(out);
        }
        out.push_bytes(sep);
    }

    pub fn null_bit_map(&self) -> i64 {
        self.null_bit_map
    }

    pub fn get_cr_returned_date_sk(&self) -> i64 {
        self.cr_returned_date_sk
    }

    pub fn get_cr_returned_time_sk(&self) -> i64 {
        self.cr_returned_time_sk
    }

    pub fn get_cr_item_sk(&self) -> i64 {
        self.cr_item_sk
    }

    pub fn get_cr_refunded_customer_sk(&self) -> i64 {
        self.cr_refunded_customer_sk
    }

    pub fn get_cr_refunded_cdemo_sk(&self) -> i64 {
        self.cr_refunded_cdemo_sk
    }

    pub fn get_cr_refunded_hdemo_sk(&self) -> i64 {
        self.cr_refunded_hdemo_sk
    }

    pub fn get_cr_refunded_addr_sk(&self) -> i64 {
        self.cr_refunded_addr_sk
    }

    pub fn get_cr_returning_customer_sk(&self) -> i64 {
        self.cr_returning_customer_sk
    }

    pub fn get_cr_returning_cdemo_sk(&self) -> i64 {
        self.cr_returning_cdemo_sk
    }

    pub fn get_cr_returning_hdemo_sk(&self) -> i64 {
        self.cr_returning_hdemo_sk
    }

    pub fn get_cr_returning_addr_sk(&self) -> i64 {
        self.cr_returning_addr_sk
    }

    pub fn get_cr_call_center_sk(&self) -> i64 {
        self.cr_call_center_sk
    }

    pub fn get_cr_catalog_page_sk(&self) -> i64 {
        self.cr_catalog_page_sk
    }

    pub fn get_cr_ship_mode_sk(&self) -> i64 {
        self.cr_ship_mode_sk
    }

    pub fn get_cr_warehouse_sk(&self) -> i64 {
        self.cr_warehouse_sk
    }

    pub fn get_cr_reason_sk(&self) -> i64 {
        self.cr_reason_sk
    }

    pub fn get_cr_order_number(&self) -> i64 {
        self.cr_order_number
    }

    pub fn get_cr_pricing(&self) -> &Pricing {
        &self.cr_pricing
    }
}

impl TableRow for CatalogReturnsRow {
    fn get_values(&self) -> Vec<String> {
        use CatalogReturnsGeneratorColumn::*;

        vec![
            self.get_string_or_null_for_key(self.cr_returned_date_sk, &CrReturnedDateSk),
            self.get_string_or_null_for_key(self.cr_returned_time_sk, &CrReturnedTimeSk),
            self.get_string_or_null_for_key(self.cr_item_sk, &CrItemSk),
            self.get_string_or_null_for_key(self.cr_refunded_customer_sk, &CrRefundedCustomerSk),
            self.get_string_or_null_for_key(self.cr_refunded_cdemo_sk, &CrRefundedCdemoSk),
            self.get_string_or_null_for_key(self.cr_refunded_hdemo_sk, &CrRefundedHdemoSk),
            self.get_string_or_null_for_key(self.cr_refunded_addr_sk, &CrRefundedAddrSk),
            self.get_string_or_null_for_key(self.cr_returning_customer_sk, &CrReturningCustomerSk),
            self.get_string_or_null_for_key(self.cr_returning_cdemo_sk, &CrReturningCdemoSk),
            self.get_string_or_null_for_key(self.cr_returning_hdemo_sk, &CrReturningHdemoSk),
            self.get_string_or_null_for_key(self.cr_returning_addr_sk, &CrReturningAddrSk),
            self.get_string_or_null_for_key(self.cr_call_center_sk, &CrCallCenterSk),
            self.get_string_or_null_for_key(self.cr_catalog_page_sk, &CrCatalogPageSk),
            self.get_string_or_null_for_key(self.cr_ship_mode_sk, &CrShipModeSk),
            self.get_string_or_null_for_key(self.cr_warehouse_sk, &CrWarehouseSk),
            self.get_string_or_null_for_key(self.cr_reason_sk, &CrReasonSk),
            self.get_string_or_null(self.cr_order_number, &CrOrderNumber),
            self.get_string_or_null(self.cr_pricing.get_quantity(), &CrPricingQuantity),
            self.get_string_or_null(self.cr_pricing.get_net_paid(), &CrPricingNetPaid),
            self.get_string_or_null(self.cr_pricing.get_ext_tax(), &CrPricingExtTax),
            self.get_string_or_null(
                self.cr_pricing.get_net_paid_including_tax(),
                &CrPricingNetPaidIncTax,
            ),
            self.get_string_or_null(self.cr_pricing.get_fee(), &CrPricingFee),
            self.get_string_or_null(self.cr_pricing.get_ext_ship_cost(), &CrPricingExtShipCost),
            self.get_string_or_null(self.cr_pricing.get_refunded_cash(), &CrPricingRefundedCash),
            self.get_string_or_null(
                self.cr_pricing.get_reversed_charge(),
                &CrPricingReversedCharge,
            ),
            self.get_string_or_null(self.cr_pricing.get_store_credit(), &CrPricingStoreCredit),
            self.get_string_or_null(self.cr_pricing.get_net_loss(), &CrPricingNetLoss),
        ]
    }

    fn append_line(&self, out: &mut Line, separator: char) {
        use CatalogReturnsGeneratorColumn::*;
        let mut sep_buf = [0u8; 4];
        let sep = separator.encode_utf8(&mut sep_buf).as_bytes();

        self.push_key(out, sep, self.cr_returned_date_sk, &CrReturnedDateSk);
        self.push_key(out, sep, self.cr_returned_time_sk, &CrReturnedTimeSk);
        self.push_key(out, sep, self.cr_item_sk, &CrItemSk);
        self.push_key(
            out,
            sep,
            self.cr_refunded_customer_sk,
            &CrRefundedCustomerSk,
        );
        self.push_key(out, sep, self.cr_refunded_cdemo_sk, &CrRefundedCdemoSk);
        self.push_key(out, sep, self.cr_refunded_hdemo_sk, &CrRefundedHdemoSk);
        self.push_key(out, sep, self.cr_refunded_addr_sk, &CrRefundedAddrSk);
        self.push_key(
            out,
            sep,
            self.cr_returning_customer_sk,
            &CrReturningCustomerSk,
        );
        self.push_key(out, sep, self.cr_returning_cdemo_sk, &CrReturningCdemoSk);
        self.push_key(out, sep, self.cr_returning_hdemo_sk, &CrReturningHdemoSk);
        self.push_key(out, sep, self.cr_returning_addr_sk, &CrReturningAddrSk);
        self.push_key(out, sep, self.cr_call_center_sk, &CrCallCenterSk);
        self.push_key(out, sep, self.cr_catalog_page_sk, &CrCatalogPageSk);
        self.push_key(out, sep, self.cr_ship_mode_sk, &CrShipModeSk);
        self.push_key(out, sep, self.cr_warehouse_sk, &CrWarehouseSk);
        self.push_key(out, sep, self.cr_reason_sk, &CrReasonSk);
        self.push_int_field(out, sep, self.cr_order_number, &CrOrderNumber);
        self.push_int(out, sep, self.cr_pricing.get_quantity(), &CrPricingQuantity);
        self.push_decimal(out, sep, &self.cr_pricing.get_net_paid(), &CrPricingNetPaid);
        self.push_decimal(out, sep, &self.cr_pricing.get_ext_tax(), &CrPricingExtTax);
        self.push_decimal(
            out,
            sep,
            &self.cr_pricing.get_net_paid_including_tax(),
            &CrPricingNetPaidIncTax,
        );
        self.push_decimal(out, sep, &self.cr_pricing.get_fee(), &CrPricingFee);
        self.push_decimal(
            out,
            sep,
            &self.cr_pricing.get_ext_ship_cost(),
            &CrPricingExtShipCost,
        );
        self.push_decimal(
            out,
            sep,
            &self.cr_pricing.get_refunded_cash(),
            &CrPricingRefundedCash,
        );
        self.push_decimal(
            out,
            sep,
            &self.cr_pricing.get_reversed_charge(),
            &CrPricingReversedCharge,
        );
        self.push_decimal(
            out,
            sep,
            &self.cr_pricing.get_store_credit(),
            &CrPricingStoreCredit,
        );
        self.push_decimal(out, sep, &self.cr_pricing.get_net_loss(), &CrPricingNetLoss);
        out.newline();
    }
}

use crate::generator::GeneratorColumn;
