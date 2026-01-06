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
}

use crate::generator::GeneratorColumn;
