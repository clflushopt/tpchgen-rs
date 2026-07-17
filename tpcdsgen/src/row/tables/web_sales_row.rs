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

//! Web sales row definition

use crate::generator::{GeneratorColumn, WebSalesGeneratorColumn};
use crate::row::table_row::DatField;
use crate::row::TableRow;
use crate::types::Pricing;
use std::fmt;

/// Row structure for web_sales table
#[derive(Debug, Clone)]
pub struct WebSalesRow {
    null_bit_map: i64,
    ws_sold_date_sk: i64,
    ws_sold_time_sk: i64,
    ws_ship_date_sk: i64,
    ws_item_sk: i64,
    ws_bill_customer_sk: i64,
    ws_bill_cdemo_sk: i64,
    ws_bill_hdemo_sk: i64,
    ws_bill_addr_sk: i64,
    ws_ship_customer_sk: i64,
    ws_ship_cdemo_sk: i64,
    ws_ship_hdemo_sk: i64,
    ws_ship_addr_sk: i64,
    ws_web_page_sk: i64,
    ws_web_site_sk: i64,
    ws_ship_mode_sk: i64,
    ws_warehouse_sk: i64,
    ws_promo_sk: i64,
    ws_order_number: i64,
    ws_pricing: Pricing,
}

impl WebSalesRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        ws_sold_date_sk: i64,
        ws_sold_time_sk: i64,
        ws_ship_date_sk: i64,
        ws_item_sk: i64,
        ws_bill_customer_sk: i64,
        ws_bill_cdemo_sk: i64,
        ws_bill_hdemo_sk: i64,
        ws_bill_addr_sk: i64,
        ws_ship_customer_sk: i64,
        ws_ship_cdemo_sk: i64,
        ws_ship_hdemo_sk: i64,
        ws_ship_addr_sk: i64,
        ws_web_page_sk: i64,
        ws_web_site_sk: i64,
        ws_ship_mode_sk: i64,
        ws_warehouse_sk: i64,
        ws_promo_sk: i64,
        ws_order_number: i64,
        ws_pricing: Pricing,
    ) -> Self {
        WebSalesRow {
            null_bit_map,
            ws_sold_date_sk,
            ws_sold_time_sk,
            ws_ship_date_sk,
            ws_item_sk,
            ws_bill_customer_sk,
            ws_bill_cdemo_sk,
            ws_bill_hdemo_sk,
            ws_bill_addr_sk,
            ws_ship_customer_sk,
            ws_ship_cdemo_sk,
            ws_ship_hdemo_sk,
            ws_ship_addr_sk,
            ws_web_page_sk,
            ws_web_site_sk,
            ws_ship_mode_sk,
            ws_warehouse_sk,
            ws_promo_sk,
            ws_order_number,
            ws_pricing,
        }
    }

    pub fn get_ws_item_sk(&self) -> i64 {
        self.ws_item_sk
    }

    pub fn get_ws_order_number(&self) -> i64 {
        self.ws_order_number
    }

    pub fn get_ws_web_page_sk(&self) -> i64 {
        self.ws_web_page_sk
    }

    pub fn get_ws_ship_date_sk(&self) -> i64 {
        self.ws_ship_date_sk
    }

    pub fn get_ws_ship_customer_sk(&self) -> i64 {
        self.ws_ship_customer_sk
    }

    pub fn get_ws_ship_cdemo_sk(&self) -> i64 {
        self.ws_ship_cdemo_sk
    }

    pub fn get_ws_ship_hdemo_sk(&self) -> i64 {
        self.ws_ship_hdemo_sk
    }

    pub fn get_ws_ship_addr_sk(&self) -> i64 {
        self.ws_ship_addr_sk
    }

    pub fn get_ws_pricing(&self) -> &Pricing {
        &self.ws_pricing
    }

    fn is_null(&self, column: WebSalesGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - WebSalesGeneratorColumn::WsSoldDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }

    fn get_string_or_null_for_key(&self, value: i64, column: WebSalesGeneratorColumn) -> String {
        if self.is_null(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null<T: std::fmt::Display>(
        &self,
        value: T,
        column: WebSalesGeneratorColumn,
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

    pub fn get_ws_sold_date_sk(&self) -> i64 {
        self.ws_sold_date_sk
    }

    pub fn get_ws_sold_time_sk(&self) -> i64 {
        self.ws_sold_time_sk
    }

    pub fn get_ws_bill_customer_sk(&self) -> i64 {
        self.ws_bill_customer_sk
    }

    pub fn get_ws_bill_cdemo_sk(&self) -> i64 {
        self.ws_bill_cdemo_sk
    }

    pub fn get_ws_bill_hdemo_sk(&self) -> i64 {
        self.ws_bill_hdemo_sk
    }

    pub fn get_ws_bill_addr_sk(&self) -> i64 {
        self.ws_bill_addr_sk
    }

    pub fn get_ws_web_site_sk(&self) -> i64 {
        self.ws_web_site_sk
    }

    pub fn get_ws_ship_mode_sk(&self) -> i64 {
        self.ws_ship_mode_sk
    }

    pub fn get_ws_warehouse_sk(&self) -> i64 {
        self.ws_warehouse_sk
    }

    pub fn get_ws_promo_sk(&self) -> i64 {
        self.ws_promo_sk
    }
}

/// DAT field helper mirroring `get_string_or_null`/`get_string_or_null_for_key`
/// (web rows apply no key sentinel check, only the null bit).
impl WebSalesRow {
    fn field<T>(&self, value: T, column: WebSalesGeneratorColumn) -> DatField<T> {
        DatField::new(value, self.is_null(column))
    }
}

/// Formats the row as a DAT line: `|`-separated values with a trailing
/// separator and empty fields for NULL columns (no newline). Produces the
/// same bytes as joining [`TableRow::get_values`] with `|`.
impl fmt::Display for WebSalesRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use WebSalesGeneratorColumn::*;

        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|",
            self.field(self.ws_sold_date_sk, WsSoldDateSk),
            self.field(self.ws_sold_time_sk, WsSoldTimeSk),
            self.field(self.ws_ship_date_sk, WsShipDateSk),
            self.field(self.ws_item_sk, WsItemSk),
            self.field(self.ws_bill_customer_sk, WsBillCustomerSk),
            self.field(self.ws_bill_cdemo_sk, WsBillCdemoSk),
            self.field(self.ws_bill_hdemo_sk, WsBillHdemoSk),
            self.field(self.ws_bill_addr_sk, WsBillAddrSk),
            self.field(self.ws_ship_customer_sk, WsShipCustomerSk),
            self.field(self.ws_ship_cdemo_sk, WsShipCdemoSk),
            self.field(self.ws_ship_hdemo_sk, WsShipHdemoSk),
            self.field(self.ws_ship_addr_sk, WsShipAddrSk),
            self.field(self.ws_web_page_sk, WsWebPageSk),
            self.field(self.ws_web_site_sk, WsWebSiteSk),
            self.field(self.ws_ship_mode_sk, WsShipModeSk),
            self.field(self.ws_warehouse_sk, WsWarehouseSk),
            self.field(self.ws_promo_sk, WsPromoSk),
            self.field(self.ws_order_number, WsOrderNumber),
            self.field(self.ws_pricing.get_quantity(), WsPricingQuantity),
            self.field(self.ws_pricing.get_wholesale_cost(), WsPricingWholesaleCost),
            self.field(self.ws_pricing.get_list_price(), WsPricingListPrice),
            self.field(self.ws_pricing.get_sales_price(), WsPricingSalesPrice),
            self.field(self.ws_pricing.get_ext_discount_amount(), WsPricingExtDiscountAmt),
            self.field(self.ws_pricing.get_ext_sales_price(), WsPricingExtSalesPrice),
            self.field(self.ws_pricing.get_ext_wholesale_cost(), WsPricingExtWholesaleCost),
            self.field(self.ws_pricing.get_ext_list_price(), WsPricingExtListPrice),
            self.field(self.ws_pricing.get_ext_tax(), WsPricingExtTax),
            self.field(self.ws_pricing.get_coupon_amount(), WsPricingCouponAmt),
            self.field(self.ws_pricing.get_ext_ship_cost(), WsPricingExtShipCost),
            self.field(self.ws_pricing.get_net_paid(), WsPricingNetPaid),
            self.field(self.ws_pricing.get_net_paid_including_tax(), WsPricingNetPaidIncTax),
            self.field(self.ws_pricing.get_net_paid_including_shipping(), WsPricingNetPaidIncShip),
            self.field(self.ws_pricing.get_net_paid_including_shipping_and_tax(), WsPricingNetPaidIncShipTax),
            self.field(self.ws_pricing.get_net_profit(), WsPricingNetProfit),
        )
    }
}

impl TableRow for WebSalesRow {
    fn get_values(&self) -> Vec<String> {
        use WebSalesGeneratorColumn::*;
        vec![
            self.get_string_or_null_for_key(self.ws_sold_date_sk, WsSoldDateSk),
            self.get_string_or_null_for_key(self.ws_sold_time_sk, WsSoldTimeSk),
            self.get_string_or_null_for_key(self.ws_ship_date_sk, WsShipDateSk),
            self.get_string_or_null_for_key(self.ws_item_sk, WsItemSk),
            self.get_string_or_null_for_key(self.ws_bill_customer_sk, WsBillCustomerSk),
            self.get_string_or_null_for_key(self.ws_bill_cdemo_sk, WsBillCdemoSk),
            self.get_string_or_null_for_key(self.ws_bill_hdemo_sk, WsBillHdemoSk),
            self.get_string_or_null_for_key(self.ws_bill_addr_sk, WsBillAddrSk),
            self.get_string_or_null_for_key(self.ws_ship_customer_sk, WsShipCustomerSk),
            self.get_string_or_null_for_key(self.ws_ship_cdemo_sk, WsShipCdemoSk),
            self.get_string_or_null_for_key(self.ws_ship_hdemo_sk, WsShipHdemoSk),
            self.get_string_or_null_for_key(self.ws_ship_addr_sk, WsShipAddrSk),
            self.get_string_or_null_for_key(self.ws_web_page_sk, WsWebPageSk),
            self.get_string_or_null_for_key(self.ws_web_site_sk, WsWebSiteSk),
            self.get_string_or_null_for_key(self.ws_ship_mode_sk, WsShipModeSk),
            self.get_string_or_null_for_key(self.ws_warehouse_sk, WsWarehouseSk),
            self.get_string_or_null_for_key(self.ws_promo_sk, WsPromoSk),
            self.get_string_or_null_for_key(self.ws_order_number, WsOrderNumber),
            self.get_string_or_null(self.ws_pricing.get_quantity(), WsPricingQuantity),
            self.get_string_or_null(self.ws_pricing.get_wholesale_cost(), WsPricingWholesaleCost),
            self.get_string_or_null(self.ws_pricing.get_list_price(), WsPricingListPrice),
            self.get_string_or_null(self.ws_pricing.get_sales_price(), WsPricingSalesPrice),
            self.get_string_or_null(
                self.ws_pricing.get_ext_discount_amount(),
                WsPricingExtDiscountAmt,
            ),
            self.get_string_or_null(
                self.ws_pricing.get_ext_sales_price(),
                WsPricingExtSalesPrice,
            ),
            self.get_string_or_null(
                self.ws_pricing.get_ext_wholesale_cost(),
                WsPricingExtWholesaleCost,
            ),
            self.get_string_or_null(self.ws_pricing.get_ext_list_price(), WsPricingExtListPrice),
            self.get_string_or_null(self.ws_pricing.get_ext_tax(), WsPricingExtTax),
            self.get_string_or_null(self.ws_pricing.get_coupon_amount(), WsPricingCouponAmt),
            self.get_string_or_null(self.ws_pricing.get_ext_ship_cost(), WsPricingExtShipCost),
            self.get_string_or_null(self.ws_pricing.get_net_paid(), WsPricingNetPaid),
            self.get_string_or_null(
                self.ws_pricing.get_net_paid_including_tax(),
                WsPricingNetPaidIncTax,
            ),
            self.get_string_or_null(
                self.ws_pricing.get_net_paid_including_shipping(),
                WsPricingNetPaidIncShip,
            ),
            self.get_string_or_null(
                self.ws_pricing.get_net_paid_including_shipping_and_tax(),
                WsPricingNetPaidIncShipTax,
            ),
            self.get_string_or_null(self.ws_pricing.get_net_profit(), WsPricingNetProfit),
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
        // Null bit on ws_sold_time_sk; web keys have no -1 sentinel handling.
        let row = WebSalesRow::new(
            0b10,
            2451545,
            36000,
            2451550,
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
            4,
            7,
            2,
            5,
            42,
            create_test_pricing(),
        );

        let expected = format!("{}|", row.get_values().join("|"));
        assert_eq!(row.to_string(), expected);
    }
}
