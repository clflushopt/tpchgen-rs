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
use crate::row::table_row::DatField;
use crate::row::TableRow;
use crate::types::Pricing;
use std::fmt;

/// Row structure for catalog_returns table
#[derive(Debug, Clone)]
pub struct CatalogReturnsRow {
    pub(crate) null_bit_map: i64,
    pub(crate) cr_returned_date_sk: i64,
    pub(crate) cr_returned_time_sk: i64,
    pub(crate) cr_item_sk: i64,
    pub(crate) cr_refunded_customer_sk: i64,
    pub(crate) cr_refunded_cdemo_sk: i64,
    pub(crate) cr_refunded_hdemo_sk: i64,
    pub(crate) cr_refunded_addr_sk: i64,
    pub(crate) cr_returning_customer_sk: i64,
    pub(crate) cr_returning_cdemo_sk: i64,
    pub(crate) cr_returning_hdemo_sk: i64,
    pub(crate) cr_returning_addr_sk: i64,
    pub(crate) cr_call_center_sk: i64,
    pub(crate) cr_catalog_page_sk: i64,
    pub(crate) cr_ship_mode_sk: i64,
    pub(crate) cr_warehouse_sk: i64,
    pub(crate) cr_reason_sk: i64,
    pub(crate) cr_order_number: i64,
    pub(crate) cr_pricing: Pricing,
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

    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative (mirrors `get_string_or_null_for_key`).
    pub(crate) fn key_field(
        &self,
        value: i64,
        column: &CatalogReturnsGeneratorColumn,
    ) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set
    /// (mirrors `get_string_or_null`).
    pub(crate) fn field<T>(&self, value: T, column: &CatalogReturnsGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for CatalogReturnsRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CatalogReturnsGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.cr_returned_date_sk, &CrReturnedDateSk),
            self.key_field(self.cr_returned_time_sk, &CrReturnedTimeSk),
            self.key_field(self.cr_item_sk, &CrItemSk),
            self.key_field(self.cr_refunded_customer_sk, &CrRefundedCustomerSk),
            self.key_field(self.cr_refunded_cdemo_sk, &CrRefundedCdemoSk),
            self.key_field(self.cr_refunded_hdemo_sk, &CrRefundedHdemoSk),
            self.key_field(self.cr_refunded_addr_sk, &CrRefundedAddrSk),
            self.key_field(self.cr_returning_customer_sk, &CrReturningCustomerSk),
            self.key_field(self.cr_returning_cdemo_sk, &CrReturningCdemoSk),
            self.key_field(self.cr_returning_hdemo_sk, &CrReturningHdemoSk),
            self.key_field(self.cr_returning_addr_sk, &CrReturningAddrSk),
            self.key_field(self.cr_call_center_sk, &CrCallCenterSk),
            self.key_field(self.cr_catalog_page_sk, &CrCatalogPageSk),
            self.key_field(self.cr_ship_mode_sk, &CrShipModeSk),
            self.key_field(self.cr_warehouse_sk, &CrWarehouseSk),
            self.key_field(self.cr_reason_sk, &CrReasonSk),
            self.field(self.cr_order_number, &CrOrderNumber),
            self.field(self.cr_pricing.get_quantity(), &CrPricingQuantity),
            self.field(self.cr_pricing.get_net_paid(), &CrPricingNetPaid),
            self.field(self.cr_pricing.get_ext_tax(), &CrPricingExtTax),
            self.field(
                self.cr_pricing.get_net_paid_including_tax(),
                &CrPricingNetPaidIncTax
            ),
            self.field(self.cr_pricing.get_fee(), &CrPricingFee),
            self.field(self.cr_pricing.get_ext_ship_cost(), &CrPricingExtShipCost),
            self.field(self.cr_pricing.get_refunded_cash(), &CrPricingRefundedCash),
            self.field(
                self.cr_pricing.get_reversed_charge(),
                &CrPricingReversedCharge
            ),
            self.field(self.cr_pricing.get_store_credit(), &CrPricingStoreCredit),
            self.field(self.cr_pricing.get_net_loss(), &CrPricingNetLoss),
        )
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
        // A null bit (cr_returned_time_sk) plus a negative key (cr_reason_sk)
        // exercise both NULL paths.
        let row = CatalogReturnsRow::new(
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
            3,
            50,
            7,
            2,
            -1,
            42,
            create_test_pricing(),
        );

        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
