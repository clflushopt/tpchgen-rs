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

use crate::column::{Column, ColumnType, ColumnTypes, Table};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionColumn {
    PPromoSk,
    PPromoId,
    PStartDateSk,
    PEndDateSk,
    PItemSk,
    PCost,
    PResponseTarge, // Note: Matches Java typo (missing 'T')
    PPromoName,
    PChannelDmail,
    PChannelEmail,
    PChannelCatalog,
    PChannelTv,
    PChannelRadio,
    PChannelPress,
    PChannelEvent,
    PChannelDemo,
    PChannelDetails,
    PPurpose,
    PDiscountActive,
}

impl PromotionColumn {
    /// Get all column values in order
    pub fn values() -> &'static [PromotionColumn] {
        &[
            PromotionColumn::PPromoSk,
            PromotionColumn::PPromoId,
            PromotionColumn::PStartDateSk,
            PromotionColumn::PEndDateSk,
            PromotionColumn::PItemSk,
            PromotionColumn::PCost,
            PromotionColumn::PResponseTarge,
            PromotionColumn::PPromoName,
            PromotionColumn::PChannelDmail,
            PromotionColumn::PChannelEmail,
            PromotionColumn::PChannelCatalog,
            PromotionColumn::PChannelTv,
            PromotionColumn::PChannelRadio,
            PromotionColumn::PChannelPress,
            PromotionColumn::PChannelEvent,
            PromotionColumn::PChannelDemo,
            PromotionColumn::PChannelDetails,
            PromotionColumn::PPurpose,
            PromotionColumn::PDiscountActive,
        ]
    }
}

impl Column for PromotionColumn {
    fn get_table(&self) -> Table {
        Table::Promotion
    }

    fn get_name(&self) -> &'static str {
        match self {
            PromotionColumn::PPromoSk => "p_promo_sk",
            PromotionColumn::PPromoId => "p_promo_id",
            PromotionColumn::PStartDateSk => "p_start_date_sk",
            PromotionColumn::PEndDateSk => "p_end_date_sk",
            PromotionColumn::PItemSk => "p_item_sk",
            PromotionColumn::PCost => "p_cost",
            PromotionColumn::PResponseTarge => "p_response_targe", // Matches Java typo
            PromotionColumn::PPromoName => "p_promo_name",
            PromotionColumn::PChannelDmail => "p_channel_dmail",
            PromotionColumn::PChannelEmail => "p_channel_email",
            PromotionColumn::PChannelCatalog => "p_channel_catalog",
            PromotionColumn::PChannelTv => "p_channel_tv",
            PromotionColumn::PChannelRadio => "p_channel_radio",
            PromotionColumn::PChannelPress => "p_channel_press",
            PromotionColumn::PChannelEvent => "p_channel_event",
            PromotionColumn::PChannelDemo => "p_channel_demo",
            PromotionColumn::PChannelDetails => "p_channel_details",
            PromotionColumn::PPurpose => "p_purpose",
            PromotionColumn::PDiscountActive => "p_discount_active",
        }
    }

    fn get_type(&self) -> &ColumnType {
        use PromotionColumn::*;
        match self {
            PPromoSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            PPromoId => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(16))
            }
            PStartDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            PEndDateSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            PItemSk => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::identifier().clone())
            }
            PCost => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::decimal(15, 2))
            }
            PResponseTarge => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::integer().clone())
            }
            PPromoName => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(50))
            }
            PChannelDmail => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelEmail => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelCatalog => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelTv => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelRadio => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelPress => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelEvent => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelDemo => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
            PChannelDetails => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::varchar(100))
            }
            PPurpose => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(15))
            }
            PDiscountActive => {
                static TYPE: OnceLock<ColumnType> = OnceLock::new();
                TYPE.get_or_init(|| ColumnTypes::character(1))
            }
        }
    }

    fn get_position(&self) -> i32 {
        match self {
            PromotionColumn::PPromoSk => 0,
            PromotionColumn::PPromoId => 1,
            PromotionColumn::PStartDateSk => 2,
            PromotionColumn::PEndDateSk => 3,
            PromotionColumn::PItemSk => 4,
            PromotionColumn::PCost => 5,
            PromotionColumn::PResponseTarge => 6,
            PromotionColumn::PPromoName => 7,
            PromotionColumn::PChannelDmail => 8,
            PromotionColumn::PChannelEmail => 9,
            PromotionColumn::PChannelCatalog => 10,
            PromotionColumn::PChannelTv => 11,
            PromotionColumn::PChannelRadio => 12,
            PromotionColumn::PChannelPress => 13,
            PromotionColumn::PChannelEvent => 14,
            PromotionColumn::PChannelDemo => 15,
            PromotionColumn::PChannelDetails => 16,
            PromotionColumn::PPurpose => 17,
            PromotionColumn::PDiscountActive => 18,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnTypeBase;

    #[test]
    fn test_promotion_column_count() {
        assert_eq!(PromotionColumn::values().len(), 19);
    }

    #[test]
    fn test_promotion_column_names() {
        assert_eq!(PromotionColumn::PPromoSk.get_name(), "p_promo_sk");
        assert_eq!(PromotionColumn::PPromoId.get_name(), "p_promo_id");
        // Test typo is preserved
        assert_eq!(
            PromotionColumn::PResponseTarge.get_name(),
            "p_response_targe"
        );
    }

    #[test]
    fn test_promotion_column_types() {
        assert_eq!(
            PromotionColumn::PPromoSk.get_type().get_base(),
            ColumnTypeBase::Identifier
        );
        assert_eq!(
            PromotionColumn::PCost.get_type().get_base(),
            ColumnTypeBase::Decimal
        );
        assert_eq!(PromotionColumn::PCost.get_type().get_precision(), Some(15));
        assert_eq!(PromotionColumn::PCost.get_type().get_scale(), Some(2));
    }

    #[test]
    fn test_promotion_column_positions() {
        assert_eq!(PromotionColumn::PPromoSk.get_position(), 0);
        assert_eq!(PromotionColumn::PDiscountActive.get_position(), 18);
    }

    #[test]
    fn test_promotion_column_table() {
        assert_eq!(PromotionColumn::PPromoSk.get_table(), Table::Promotion);
    }
}
