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
use crate::output::Line;
use crate::row::TableRow;
use crate::types::Pricing;

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

    fn push_key(
        &self,
        out: &mut Line,
        sep: &[u8],
        value: i64,
        column: &CatalogSalesGeneratorColumn,
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
        column: &CatalogSalesGeneratorColumn,
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
        column: &CatalogSalesGeneratorColumn,
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
        column: &CatalogSalesGeneratorColumn,
    ) {
        if !self.is_null(column) {
            value.append_dat(out);
        }
        out.push_bytes(sep);
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

    fn append_line(&self, out: &mut Line, separator: char) {
        use CatalogSalesGeneratorColumn::*;
        let mut sep_buf = [0u8; 4];
        let sep = separator.encode_utf8(&mut sep_buf).as_bytes();

        self.push_key(out, sep, self.cs_sold_date_sk, &CsSoldDateSk);
        self.push_key(out, sep, self.cs_sold_time_sk, &CsSoldTimeSk);
        self.push_key(out, sep, self.cs_ship_date_sk, &CsShipDateSk);
        self.push_key(out, sep, self.cs_bill_customer_sk, &CsBillCustomerSk);
        self.push_key(out, sep, self.cs_bill_cdemo_sk, &CsBillCdemoSk);
        self.push_key(out, sep, self.cs_bill_hdemo_sk, &CsBillHdemoSk);
        self.push_key(out, sep, self.cs_bill_addr_sk, &CsBillAddrSk);
        self.push_key(out, sep, self.cs_ship_customer_sk, &CsShipCustomerSk);
        self.push_key(out, sep, self.cs_ship_cdemo_sk, &CsShipCdemoSk);
        self.push_key(out, sep, self.cs_ship_hdemo_sk, &CsShipHdemoSk);
        self.push_key(out, sep, self.cs_ship_addr_sk, &CsShipAddrSk);
        self.push_key(out, sep, self.cs_call_center_sk, &CsCallCenterSk);
        self.push_key(out, sep, self.cs_catalog_page_sk, &CsCatalogPageSk);
        self.push_key(out, sep, self.cs_ship_mode_sk, &CsShipModeSk);
        self.push_int_field(out, sep, self.cs_warehouse_sk, &CsWarehouseSk);
        self.push_key(out, sep, self.cs_sold_item_sk, &CsSoldItemSk);
        self.push_key(out, sep, self.cs_promo_sk, &CsPromoSk);
        self.push_int_field(out, sep, self.cs_order_number, &CsOrderNumber);
        self.push_int(out, sep, self.cs_pricing.get_quantity(), &CsPricingQuantity);
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_wholesale_cost(),
            &CsPricingWholesaleCost,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_list_price(),
            &CsPricingListPrice,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_sales_price(),
            &CsPricingSalesPrice,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_ext_discount_amount(),
            &CsPricingExtDiscountAmount,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_ext_sales_price(),
            &CsPricingExtSalesPrice,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_ext_wholesale_cost(),
            &CsPricingExtWholesaleCost,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_ext_list_price(),
            &CsPricingExtListPrice,
        );
        self.push_decimal(out, sep, &self.cs_pricing.get_ext_tax(), &CsPricingExtTax);
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_coupon_amount(),
            &CsPricingCouponAmt,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_ext_ship_cost(),
            &CsPricingExtShipCost,
        );
        self.push_decimal(out, sep, &self.cs_pricing.get_net_paid(), &CsPricingNetPaid);
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_net_paid_including_tax(),
            &CsPricingNetPaidIncTax,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_net_paid_including_shipping(),
            &CsPricingNetPaidIncShip,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_net_paid_including_shipping_and_tax(),
            &CsPricingNetPaidIncShipTax,
        );
        self.push_decimal(
            out,
            sep,
            &self.cs_pricing.get_net_profit(),
            &CsPricingNetProfit,
        );
        out.newline();
    }
}

use crate::generator::GeneratorColumn;
