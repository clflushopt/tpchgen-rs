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

//! Store sales row data structure

use crate::generator::{GeneratorColumn, StoreSalesGeneratorColumn};
use crate::row::TableRow;
use crate::types::Pricing;

/// Row data structure for the store_sales table
#[derive(Debug, Clone)]
pub struct StoreSalesRow {
    null_bit_map: i64,
    ss_sold_date_sk: i64,
    ss_sold_time_sk: i64,
    ss_sold_item_sk: i64,
    ss_sold_customer_sk: i64,
    ss_sold_cdemo_sk: i64,
    ss_sold_hdemo_sk: i64,
    ss_sold_addr_sk: i64,
    ss_sold_store_sk: i64,
    ss_sold_promo_sk: i64,
    ss_ticket_number: i64,
    ss_pricing: Pricing,
}

impl StoreSalesRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        ss_sold_date_sk: i64,
        ss_sold_time_sk: i64,
        ss_sold_item_sk: i64,
        ss_sold_customer_sk: i64,
        ss_sold_cdemo_sk: i64,
        ss_sold_hdemo_sk: i64,
        ss_sold_addr_sk: i64,
        ss_sold_store_sk: i64,
        ss_sold_promo_sk: i64,
        ss_ticket_number: i64,
        ss_pricing: Pricing,
    ) -> Self {
        StoreSalesRow {
            null_bit_map,
            ss_sold_date_sk,
            ss_sold_time_sk,
            ss_sold_item_sk,
            ss_sold_customer_sk,
            ss_sold_cdemo_sk,
            ss_sold_hdemo_sk,
            ss_sold_addr_sk,
            ss_sold_store_sk,
            ss_sold_promo_sk,
            ss_ticket_number,
            ss_pricing,
        }
    }

    /// Get the ticket number (for store_returns generation)
    pub fn get_ss_ticket_number(&self) -> i64 {
        self.ss_ticket_number
    }

    /// Get the sold item sk (for store_returns generation)
    pub fn get_ss_sold_item_sk(&self) -> i64 {
        self.ss_sold_item_sk
    }

    /// Get the sold customer sk (for store_returns generation)
    pub fn get_ss_sold_customer_sk(&self) -> i64 {
        self.ss_sold_customer_sk
    }

    /// Get the sold date sk (for store_returns generation)
    pub fn get_ss_sold_date_sk(&self) -> i64 {
        self.ss_sold_date_sk
    }

    /// Get the pricing (for store_returns generation)
    pub fn get_ss_pricing(&self) -> &Pricing {
        &self.ss_pricing
    }

    fn get_string_or_null_for_key(&self, key: i64, column: StoreSalesGeneratorColumn) -> String {
        if key == -1 || self.is_null_at(column) {
            String::new()
        } else {
            key.to_string()
        }
    }

    fn get_string_or_null_int(&self, value: i32, column: StoreSalesGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_decimal(
        &self,
        value: &crate::types::Decimal,
        column: StoreSalesGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn is_null_at(&self, column: StoreSalesGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - StoreSalesGeneratorColumn::SsSoldDateSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }
}

impl TableRow for StoreSalesRow {
    fn get_values(&self) -> Vec<String> {
        use StoreSalesGeneratorColumn::*;

        // Note: Java has coupon_amount twice at positions 15 and 20 (bug in original)
        // We replicate this for byte-for-byte compatibility
        vec![
            self.get_string_or_null_for_key(self.ss_sold_date_sk, SsSoldDateSk),
            self.get_string_or_null_for_key(self.ss_sold_time_sk, SsSoldTimeSk),
            self.get_string_or_null_for_key(self.ss_sold_item_sk, SsSoldItemSk),
            self.get_string_or_null_for_key(self.ss_sold_customer_sk, SsSoldCustomerSk),
            self.get_string_or_null_for_key(self.ss_sold_cdemo_sk, SsSoldCdemoSk),
            self.get_string_or_null_for_key(self.ss_sold_hdemo_sk, SsSoldHdemoSk),
            self.get_string_or_null_for_key(self.ss_sold_addr_sk, SsSoldAddrSk),
            self.get_string_or_null_for_key(self.ss_sold_store_sk, SsSoldStoreSk),
            self.get_string_or_null_for_key(self.ss_sold_promo_sk, SsSoldPromoSk),
            self.get_string_or_null_for_key(self.ss_ticket_number, SsTicketNumber),
            self.get_string_or_null_int(self.ss_pricing.get_quantity(), SsPricingQuantity),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_wholesale_cost(),
                SsPricingWholesaleCost,
            ),
            self.get_string_or_null_decimal(&self.ss_pricing.get_list_price(), SsPricingListPrice),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_sales_price(),
                SsPricingSalesPrice,
            ),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_coupon_amount(),
                SsPricingCouponAmt,
            ),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_ext_sales_price(),
                SsPricingExtSalesPrice,
            ),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_ext_wholesale_cost(),
                SsPricingExtWholesaleCost,
            ),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_ext_list_price(),
                SsPricingExtListPrice,
            ),
            self.get_string_or_null_decimal(&self.ss_pricing.get_ext_tax(), SsPricingExtTax),
            // Note: coupon_amount appears twice in Java (bug replicated for compatibility)
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_coupon_amount(),
                SsPricingCouponAmt,
            ),
            self.get_string_or_null_decimal(&self.ss_pricing.get_net_paid(), SsPricingNetPaid),
            self.get_string_or_null_decimal(
                &self.ss_pricing.get_net_paid_including_tax(),
                SsPricingNetPaidIncTax,
            ),
            self.get_string_or_null_decimal(&self.ss_pricing.get_net_profit(), SsPricingNetProfit),
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
            Decimal::ZERO,                  // refunded_cash
            Decimal::ZERO,                  // reversed_charge
            Decimal::ZERO,                  // store_credit
            Decimal::ZERO,                  // fee
            Decimal::ZERO,                  // net_loss
        )
    }

    #[test]
    fn test_store_sales_row_creation() {
        let pricing = create_test_pricing();
        let row = StoreSalesRow::new(
            0,       // null_bit_map
            2451545, // ss_sold_date_sk
            36000,   // ss_sold_time_sk
            1,       // ss_sold_item_sk
            100,     // ss_sold_customer_sk
            200,     // ss_sold_cdemo_sk
            300,     // ss_sold_hdemo_sk
            400,     // ss_sold_addr_sk
            500,     // ss_sold_store_sk
            600,     // ss_sold_promo_sk
            1,       // ss_ticket_number
            pricing,
        );

        assert_eq!(row.get_ss_ticket_number(), 1);
        assert_eq!(row.get_ss_sold_item_sk(), 1);
        assert_eq!(row.get_ss_sold_customer_sk(), 100);
    }

    #[test]
    fn test_store_sales_row_values() {
        let pricing = create_test_pricing();
        let row = StoreSalesRow::new(
            0, 2451545, 36000, 1, 100, 200, 300, 400, 500, 600, 1, pricing,
        );

        let values = row.get_values();
        assert_eq!(values.len(), 23);
        assert_eq!(values[0], "2451545"); // ss_sold_date_sk
        assert_eq!(values[2], "1"); // ss_sold_item_sk
        assert_eq!(values[9], "1"); // ss_ticket_number
        assert_eq!(values[10], "5"); // quantity
    }

    #[test]
    fn test_store_sales_row_null_handling() {
        let pricing = create_test_pricing();
        // Set bit for ss_sold_time_sk (position 1)
        let row = StoreSalesRow::new(
            0b10, // null_bit_map - second bit set
            2451545, 36000, 1, 100, 200, 300, 400, 500, 600, 1, pricing,
        );

        let values = row.get_values();
        assert_eq!(values[0], "2451545"); // not null
        assert_eq!(values[1], ""); // null (bit 1 set)
    }
}
