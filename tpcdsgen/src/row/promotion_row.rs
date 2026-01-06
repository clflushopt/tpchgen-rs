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

use crate::generator::{GeneratorColumn, PromotionGeneratorColumn};
use crate::row::TableRow;
use crate::types::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct PromotionRow {
    null_bit_map: i64,
    p_promo_sk: i64,
    p_promo_id: String,
    p_start_date_id: i64,
    p_end_date_id: i64,
    p_item_sk: i64,
    p_cost: Decimal,
    p_response_target: i32,
    p_promo_name: String,
    p_channel_dmail: bool,
    p_channel_email: bool,
    p_channel_catalog: bool,
    p_channel_tv: bool,
    p_channel_radio: bool,
    p_channel_press: bool,
    p_channel_event: bool,
    p_channel_demo: bool,
    p_channel_details: String,
    p_purpose: String,
    p_discount_active: bool,
}

impl PromotionRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        null_bit_map: i64,
        p_promo_sk: i64,
        p_promo_id: String,
        p_start_date_id: i64,
        p_end_date_id: i64,
        p_item_sk: i64,
        p_cost: Decimal,
        p_response_target: i32,
        p_promo_name: String,
        p_channel_dmail: bool,
        p_channel_email: bool,
        p_channel_catalog: bool,
        p_channel_tv: bool,
        p_channel_radio: bool,
        p_channel_press: bool,
        p_channel_event: bool,
        p_channel_demo: bool,
        p_channel_details: String,
        p_purpose: String,
        p_discount_active: bool,
    ) -> Self {
        PromotionRow {
            null_bit_map,
            p_promo_sk,
            p_promo_id,
            p_start_date_id,
            p_end_date_id,
            p_item_sk,
            p_cost,
            p_response_target,
            p_promo_name,
            p_channel_dmail,
            p_channel_email,
            p_channel_catalog,
            p_channel_tv,
            p_channel_radio,
            p_channel_press,
            p_channel_event,
            p_channel_demo,
            p_channel_details,
            p_purpose,
            p_discount_active,
        }
    }

    fn get_string_or_null_for_key(&self, key: i64, column: PromotionGeneratorColumn) -> String {
        if key == -1 || self.is_null_at(column) {
            String::new()
        } else {
            key.to_string()
        }
    }

    fn get_string_or_null_string(&self, value: &str, column: PromotionGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_decimal(
        &self,
        value: &Decimal,
        column: PromotionGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_int(&self, value: i32, column: PromotionGeneratorColumn) -> String {
        if self.is_null_at(column) {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn get_string_or_null_for_boolean(
        &self,
        value: bool,
        column: PromotionGeneratorColumn,
    ) -> String {
        if self.is_null_at(column) {
            String::new()
        } else if value {
            "Y".to_string()
        } else {
            "N".to_string()
        }
    }

    fn is_null_at(&self, column: PromotionGeneratorColumn) -> bool {
        let bit_position = column.get_global_column_number()
            - PromotionGeneratorColumn::PPromoSk.get_global_column_number();
        (self.null_bit_map & (1 << bit_position)) != 0
    }
}

impl TableRow for PromotionRow {
    fn get_values(&self) -> Vec<String> {
        vec![
            self.get_string_or_null_for_key(self.p_promo_sk, PromotionGeneratorColumn::PPromoSk),
            self.get_string_or_null_string(&self.p_promo_id, PromotionGeneratorColumn::PPromoId),
            self.get_string_or_null_for_key(
                self.p_start_date_id,
                PromotionGeneratorColumn::PStartDateId,
            ),
            self.get_string_or_null_for_key(
                self.p_end_date_id,
                PromotionGeneratorColumn::PEndDateId,
            ),
            self.get_string_or_null_for_key(self.p_item_sk, PromotionGeneratorColumn::PItemSk),
            self.get_string_or_null_decimal(&self.p_cost, PromotionGeneratorColumn::PCost),
            self.get_string_or_null_int(
                self.p_response_target,
                PromotionGeneratorColumn::PResponseTarget,
            ),
            self.get_string_or_null_string(
                &self.p_promo_name,
                PromotionGeneratorColumn::PPromoName,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_dmail,
                PromotionGeneratorColumn::PChannelDmail,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_email,
                PromotionGeneratorColumn::PChannelEmail,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_catalog,
                PromotionGeneratorColumn::PChannelCatalog,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_tv,
                PromotionGeneratorColumn::PChannelTv,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_radio,
                PromotionGeneratorColumn::PChannelRadio,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_press,
                PromotionGeneratorColumn::PChannelPress,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_event,
                PromotionGeneratorColumn::PChannelEvent,
            ),
            self.get_string_or_null_for_boolean(
                self.p_channel_demo,
                PromotionGeneratorColumn::PChannelDemo,
            ),
            self.get_string_or_null_string(
                &self.p_channel_details,
                PromotionGeneratorColumn::PChannelDetails,
            ),
            self.get_string_or_null_string(&self.p_purpose, PromotionGeneratorColumn::PPurpose),
            self.get_string_or_null_for_boolean(
                self.p_discount_active,
                PromotionGeneratorColumn::PDiscountActive,
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotion_row_creation() {
        let row = PromotionRow::new(
            0,
            1,
            "test_id".to_string(),
            2450815,
            2450875,
            100,
            Decimal::new(1000, 2).unwrap(),
            1,
            "TestPromo".to_string(),
            true,
            false,
            true,
            false,
            true,
            false,
            true,
            false,
            "Details".to_string(),
            "Unknown".to_string(),
            true,
        );

        assert_eq!(row.p_promo_sk, 1);
        assert_eq!(row.p_promo_id, "test_id");
    }

    #[test]
    fn test_promotion_row_values() {
        let row = PromotionRow::new(
            0,
            1,
            "AAAAAAAABAAAAAAA".to_string(),
            2450815,
            2450875,
            100,
            Decimal::new(100000, 2).unwrap(),
            1,
            "TestPromo".to_string(),
            true,
            false,
            true,
            false,
            true,
            false,
            true,
            false,
            "Channel details".to_string(),
            "Unknown".to_string(),
            true,
        );

        let values = row.get_values();
        assert_eq!(values.len(), 19);
        assert_eq!(values[0], "1");
        assert_eq!(values[1], "AAAAAAAABAAAAAAA");
        assert_eq!(values[8], "Y"); // p_channel_dmail
        assert_eq!(values[9], "N"); // p_channel_email
    }
}
