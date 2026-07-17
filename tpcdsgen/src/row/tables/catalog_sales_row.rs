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

//! Catalog sales row structure

use crate::generator::CatalogSalesGeneratorColumn;
use crate::row::table_row::DatField;
use crate::row::TableRow;
use crate::types::Pricing;
use std::fmt;

/// Row structure for catalog_sales table
#[derive(Debug, Clone)]
pub struct CatalogSalesRow {
    null_bit_map: i64,
    cs_sold_date_sk: i64,
    cs_sold_time_sk: i64,
    cs_ship_date_sk: i64,
    cs_bill_customer_sk: i64,
    cs_bill_cdemo_sk: i64,
    cs_bill_hdemo_sk: i64,
    cs_bill_addr_sk: i64,
    cs_ship_customer_sk: i64,
    cs_ship_cdemo_sk: i64,
    cs_ship_hdemo_sk: i64,
    cs_ship_addr_sk: i64,
    cs_call_center_sk: i64,
    cs_catalog_page_sk: i64,
    cs_ship_mode_sk: i64,
    cs_warehouse_sk: i64,
    cs_sold_item_sk: i64,
    cs_promo_sk: i64,
    cs_order_number: i64,
    cs_pricing: Pricing,
}

impl CatalogSalesRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        cs_sold_date_sk: i64,
        cs_sold_time_sk: i64,
        cs_ship_date_sk: i64,
        cs_bill_customer_sk: i64,
        cs_bill_cdemo_sk: i64,
        cs_bill_hdemo_sk: i64,
        cs_bill_addr_sk: i64,
        cs_ship_customer_sk: i64,
        cs_ship_cdemo_sk: i64,
        cs_ship_hdemo_sk: i64,
        cs_ship_addr_sk: i64,
        cs_call_center_sk: i64,
        cs_catalog_page_sk: i64,
        cs_ship_mode_sk: i64,
        cs_warehouse_sk: i64,
        cs_sold_item_sk: i64,
        cs_promo_sk: i64,
        cs_order_number: i64,
        cs_pricing: Pricing,
    ) -> Self {
        CatalogSalesRow {
            null_bit_map,
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
            cs_sold_item_sk,
            cs_promo_sk,
            cs_order_number,
            cs_pricing,
        }
    }

    // Getters for fields needed by CatalogReturnsRowGenerator
    pub fn get_cs_ship_date_sk(&self) -> i64 {
        self.cs_ship_date_sk
    }

    pub fn get_cs_sold_item_sk(&self) -> i64 {
        self.cs_sold_item_sk
    }

    pub fn get_cs_bill_customer_sk(&self) -> i64 {
        self.cs_bill_customer_sk
    }

    pub fn get_cs_bill_cdemo_sk(&self) -> i64 {
        self.cs_bill_cdemo_sk
    }

    pub fn get_cs_bill_hdemo_sk(&self) -> i64 {
        self.cs_bill_hdemo_sk
    }

    pub fn get_cs_bill_addr_sk(&self) -> i64 {
        self.cs_bill_addr_sk
    }

    pub fn get_cs_ship_customer_sk(&self) -> i64 {
        self.cs_ship_customer_sk
    }

    pub fn get_cs_ship_cdemo_sk(&self) -> i64 {
        self.cs_ship_cdemo_sk
    }

    pub fn get_cs_ship_addr_sk(&self) -> i64 {
        self.cs_ship_addr_sk
    }

    pub fn get_cs_call_center_sk(&self) -> i64 {
        self.cs_call_center_sk
    }

    pub fn get_cs_catalog_page_sk(&self) -> i64 {
        self.cs_catalog_page_sk
    }

    pub fn get_cs_order_number(&self) -> i64 {
        self.cs_order_number
    }

    pub fn get_cs_pricing(&self) -> &Pricing {
        &self.cs_pricing
    }

    fn is_null(&self, column: &CatalogSalesGeneratorColumn) -> bool {
        let column_number = column.get_global_column_number();
        let first_column = CatalogSalesGeneratorColumn::CsSoldDateSk.get_global_column_number();
        let bit_position = column_number - first_column;
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(
        &self,
        value: i64,
        column: &CatalogSalesGeneratorColumn,
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
        column: &CatalogSalesGeneratorColumn,
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

    pub fn get_cs_sold_date_sk(&self) -> i64 {
        self.cs_sold_date_sk
    }

    pub fn get_cs_sold_time_sk(&self) -> i64 {
        self.cs_sold_time_sk
    }

    pub fn get_cs_ship_hdemo_sk(&self) -> i64 {
        self.cs_ship_hdemo_sk
    }

    pub fn get_cs_ship_mode_sk(&self) -> i64 {
        self.cs_ship_mode_sk
    }

    pub fn get_cs_warehouse_sk(&self) -> i64 {
        self.cs_warehouse_sk
    }

    pub fn get_cs_promo_sk(&self) -> i64 {
        self.cs_promo_sk
    }

    /// DAT field for a surrogate key: NULL when the null bit is set or the
    /// key is negative (mirrors `get_string_or_null_for_key`).
    fn key_field(&self, value: i64, column: &CatalogSalesGeneratorColumn) -> DatField<i64> {
        DatField::new(value, self.is_null(column) || value < 0)
    }

    /// DAT field for a regular value: NULL when the null bit is set
    /// (mirrors `get_string_or_null`).
    fn field<T>(&self, value: T, column: &CatalogSalesGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for CatalogSalesRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CatalogSalesGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.key_field(self.cs_sold_date_sk, &CsSoldDateSk),
            self.key_field(self.cs_sold_time_sk, &CsSoldTimeSk),
            self.key_field(self.cs_ship_date_sk, &CsShipDateSk),
            self.key_field(self.cs_bill_customer_sk, &CsBillCustomerSk),
            self.key_field(self.cs_bill_cdemo_sk, &CsBillCdemoSk),
            self.key_field(self.cs_bill_hdemo_sk, &CsBillHdemoSk),
            self.key_field(self.cs_bill_addr_sk, &CsBillAddrSk),
            self.key_field(self.cs_ship_customer_sk, &CsShipCustomerSk),
            self.key_field(self.cs_ship_cdemo_sk, &CsShipCdemoSk),
            self.key_field(self.cs_ship_hdemo_sk, &CsShipHdemoSk),
            self.key_field(self.cs_ship_addr_sk, &CsShipAddrSk),
            self.key_field(self.cs_call_center_sk, &CsCallCenterSk),
            self.key_field(self.cs_catalog_page_sk, &CsCatalogPageSk),
            self.key_field(self.cs_ship_mode_sk, &CsShipModeSk),
            self.field(self.cs_warehouse_sk, &CsWarehouseSk),
            self.key_field(self.cs_sold_item_sk, &CsSoldItemSk),
            self.key_field(self.cs_promo_sk, &CsPromoSk),
            self.field(self.cs_order_number, &CsOrderNumber),
            self.field(self.cs_pricing.get_quantity(), &CsPricingQuantity),
            self.field(
                self.cs_pricing.get_wholesale_cost(),
                &CsPricingWholesaleCost
            ),
            self.field(self.cs_pricing.get_list_price(), &CsPricingListPrice),
            self.field(self.cs_pricing.get_sales_price(), &CsPricingSalesPrice),
            self.field(
                self.cs_pricing.get_ext_discount_amount(),
                &CsPricingExtDiscountAmount
            ),
            self.field(
                self.cs_pricing.get_ext_sales_price(),
                &CsPricingExtSalesPrice
            ),
            self.field(
                self.cs_pricing.get_ext_wholesale_cost(),
                &CsPricingExtWholesaleCost
            ),
            self.field(
                self.cs_pricing.get_ext_list_price(),
                &CsPricingExtListPrice
            ),
            self.field(self.cs_pricing.get_ext_tax(), &CsPricingExtTax),
            self.field(self.cs_pricing.get_coupon_amount(), &CsPricingCouponAmt),
            self.field(self.cs_pricing.get_ext_ship_cost(), &CsPricingExtShipCost),
            self.field(self.cs_pricing.get_net_paid(), &CsPricingNetPaid),
            self.field(
                self.cs_pricing.get_net_paid_including_tax(),
                &CsPricingNetPaidIncTax
            ),
            self.field(
                self.cs_pricing.get_net_paid_including_shipping(),
                &CsPricingNetPaidIncShip
            ),
            self.field(
                self.cs_pricing.get_net_paid_including_shipping_and_tax(),
                &CsPricingNetPaidIncShipTax
            ),
            self.field(self.cs_pricing.get_net_profit(), &CsPricingNetProfit),
        )
    }
}

impl TableRow for CatalogSalesRow {
    fn get_values(&self) -> Vec<String> {
        use CatalogSalesGeneratorColumn::*;

        vec![
            self.get_string_or_null_for_key(self.cs_sold_date_sk, &CsSoldDateSk),
            self.get_string_or_null_for_key(self.cs_sold_time_sk, &CsSoldTimeSk),
            self.get_string_or_null_for_key(self.cs_ship_date_sk, &CsShipDateSk),
            self.get_string_or_null_for_key(self.cs_bill_customer_sk, &CsBillCustomerSk),
            self.get_string_or_null_for_key(self.cs_bill_cdemo_sk, &CsBillCdemoSk),
            self.get_string_or_null_for_key(self.cs_bill_hdemo_sk, &CsBillHdemoSk),
            self.get_string_or_null_for_key(self.cs_bill_addr_sk, &CsBillAddrSk),
            self.get_string_or_null_for_key(self.cs_ship_customer_sk, &CsShipCustomerSk),
            self.get_string_or_null_for_key(self.cs_ship_cdemo_sk, &CsShipCdemoSk),
            self.get_string_or_null_for_key(self.cs_ship_hdemo_sk, &CsShipHdemoSk),
            self.get_string_or_null_for_key(self.cs_ship_addr_sk, &CsShipAddrSk),
            self.get_string_or_null_for_key(self.cs_call_center_sk, &CsCallCenterSk),
            self.get_string_or_null_for_key(self.cs_catalog_page_sk, &CsCatalogPageSk),
            self.get_string_or_null_for_key(self.cs_ship_mode_sk, &CsShipModeSk),
            self.get_string_or_null(self.cs_warehouse_sk, &CsWarehouseSk),
            self.get_string_or_null_for_key(self.cs_sold_item_sk, &CsSoldItemSk),
            self.get_string_or_null_for_key(self.cs_promo_sk, &CsPromoSk),
            self.get_string_or_null(self.cs_order_number, &CsOrderNumber),
            self.get_string_or_null(self.cs_pricing.get_quantity(), &CsPricingQuantity),
            self.get_string_or_null(
                self.cs_pricing.get_wholesale_cost(),
                &CsPricingWholesaleCost,
            ),
            self.get_string_or_null(self.cs_pricing.get_list_price(), &CsPricingListPrice),
            self.get_string_or_null(self.cs_pricing.get_sales_price(), &CsPricingSalesPrice),
            self.get_string_or_null(
                self.cs_pricing.get_ext_discount_amount(),
                &CsPricingExtDiscountAmount,
            ),
            self.get_string_or_null(
                self.cs_pricing.get_ext_sales_price(),
                &CsPricingExtSalesPrice,
            ),
            self.get_string_or_null(
                self.cs_pricing.get_ext_wholesale_cost(),
                &CsPricingExtWholesaleCost,
            ),
            self.get_string_or_null(self.cs_pricing.get_ext_list_price(), &CsPricingExtListPrice),
            self.get_string_or_null(self.cs_pricing.get_ext_tax(), &CsPricingExtTax),
            self.get_string_or_null(self.cs_pricing.get_coupon_amount(), &CsPricingCouponAmt),
            self.get_string_or_null(self.cs_pricing.get_ext_ship_cost(), &CsPricingExtShipCost),
            self.get_string_or_null(self.cs_pricing.get_net_paid(), &CsPricingNetPaid),
            self.get_string_or_null(
                self.cs_pricing.get_net_paid_including_tax(),
                &CsPricingNetPaidIncTax,
            ),
            self.get_string_or_null(
                self.cs_pricing.get_net_paid_including_shipping(),
                &CsPricingNetPaidIncShip,
            ),
            self.get_string_or_null(
                self.cs_pricing.get_net_paid_including_shipping_and_tax(),
                &CsPricingNetPaidIncShipTax,
            ),
            self.get_string_or_null(self.cs_pricing.get_net_profit(), &CsPricingNetProfit),
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
            Decimal::ZERO,                  // refunded_cash
            Decimal::ZERO,                  // reversed_charge
            Decimal::ZERO,                  // store_credit
            Decimal::ZERO,                  // fee
            Decimal::ZERO,                  // net_loss
        )
    }

    #[test]
    fn test_display_matches_get_values() {
        // A null bit (cs_sold_time_sk) plus a negative key (cs_promo_sk)
        // exercise both NULL paths.
        let row = CatalogSalesRow::new(
            0b10,
            2451545,
            36000,
            2451550,
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
            1000,
            -1,
            42,
            create_test_pricing(),
        );

        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
