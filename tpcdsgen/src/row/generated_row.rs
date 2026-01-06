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

//! Generated row enum for static dispatch and zero-allocation row storage.
//!
//! This enum eliminates the need for `Box<dyn TableRow>` in the hot path,
//! avoiding heap allocations and enabling static dispatch.
//!
//! See ISSUE-004 for details.

use crate::row::{
    CallCenterRow, CatalogPageRow, CatalogReturnsRow, CatalogSalesRow, CustomerAddressRow,
    CustomerDemographicsRow, CustomerRow, DateDimRow, DbgenVersionRow, HouseholdDemographicsRow,
    IncomeBandRow, InventoryRow, ItemRow, PromotionRow, ReasonRow, ShipModeRow, StoreReturnsRow,
    StoreRow, StoreSalesRow, TableRow, TimeDimRow, WarehouseRow, WebPageRow, WebReturnsRow,
    WebSalesRow, WebSiteRow,
};
use std::io::{self, Write};

/// Enum holding all possible generated row types.
///
/// This enables static dispatch and eliminates heap allocations that would
/// be required with `Box<dyn TableRow>`.
#[derive(Clone)]
pub enum GeneratedRow {
    CallCenter(CallCenterRow),
    CatalogPage(CatalogPageRow),
    CatalogReturns(CatalogReturnsRow),
    CatalogSales(CatalogSalesRow),
    Customer(CustomerRow),
    CustomerAddress(CustomerAddressRow),
    CustomerDemographics(CustomerDemographicsRow),
    DateDim(DateDimRow),
    DbgenVersion(DbgenVersionRow),
    HouseholdDemographics(HouseholdDemographicsRow),
    IncomeBand(IncomeBandRow),
    Inventory(InventoryRow),
    Item(ItemRow),
    Promotion(PromotionRow),
    Reason(ReasonRow),
    ShipMode(ShipModeRow),
    Store(StoreRow),
    StoreReturns(StoreReturnsRow),
    StoreSales(StoreSalesRow),
    TimeDim(TimeDimRow),
    Warehouse(WarehouseRow),
    WebPage(WebPageRow),
    WebReturns(WebReturnsRow),
    WebSales(WebSalesRow),
    WebSite(WebSiteRow),
}

impl TableRow for GeneratedRow {
    fn get_values(&self) -> Vec<String> {
        match self {
            GeneratedRow::CallCenter(row) => row.get_values(),
            GeneratedRow::CatalogPage(row) => row.get_values(),
            GeneratedRow::CatalogReturns(row) => row.get_values(),
            GeneratedRow::CatalogSales(row) => row.get_values(),
            GeneratedRow::Customer(row) => row.get_values(),
            GeneratedRow::CustomerAddress(row) => row.get_values(),
            GeneratedRow::CustomerDemographics(row) => row.get_values(),
            GeneratedRow::DateDim(row) => row.get_values(),
            GeneratedRow::DbgenVersion(row) => row.get_values(),
            GeneratedRow::HouseholdDemographics(row) => row.get_values(),
            GeneratedRow::IncomeBand(row) => row.get_values(),
            GeneratedRow::Inventory(row) => row.get_values(),
            GeneratedRow::Item(row) => row.get_values(),
            GeneratedRow::Promotion(row) => row.get_values(),
            GeneratedRow::Reason(row) => row.get_values(),
            GeneratedRow::ShipMode(row) => row.get_values(),
            GeneratedRow::Store(row) => row.get_values(),
            GeneratedRow::StoreReturns(row) => row.get_values(),
            GeneratedRow::StoreSales(row) => row.get_values(),
            GeneratedRow::TimeDim(row) => row.get_values(),
            GeneratedRow::Warehouse(row) => row.get_values(),
            GeneratedRow::WebPage(row) => row.get_values(),
            GeneratedRow::WebReturns(row) => row.get_values(),
            GeneratedRow::WebSales(row) => row.get_values(),
            GeneratedRow::WebSite(row) => row.get_values(),
        }
    }

    fn write_to(&self, writer: &mut dyn Write, separator: char) -> io::Result<()> {
        match self {
            GeneratedRow::CallCenter(row) => row.write_to(writer, separator),
            GeneratedRow::CatalogPage(row) => row.write_to(writer, separator),
            GeneratedRow::CatalogReturns(row) => row.write_to(writer, separator),
            GeneratedRow::CatalogSales(row) => row.write_to(writer, separator),
            GeneratedRow::Customer(row) => row.write_to(writer, separator),
            GeneratedRow::CustomerAddress(row) => row.write_to(writer, separator),
            GeneratedRow::CustomerDemographics(row) => row.write_to(writer, separator),
            GeneratedRow::DateDim(row) => row.write_to(writer, separator),
            GeneratedRow::DbgenVersion(row) => row.write_to(writer, separator),
            GeneratedRow::HouseholdDemographics(row) => row.write_to(writer, separator),
            GeneratedRow::IncomeBand(row) => row.write_to(writer, separator),
            GeneratedRow::Inventory(row) => row.write_to(writer, separator),
            GeneratedRow::Item(row) => row.write_to(writer, separator),
            GeneratedRow::Promotion(row) => row.write_to(writer, separator),
            GeneratedRow::Reason(row) => row.write_to(writer, separator),
            GeneratedRow::ShipMode(row) => row.write_to(writer, separator),
            GeneratedRow::Store(row) => row.write_to(writer, separator),
            GeneratedRow::StoreReturns(row) => row.write_to(writer, separator),
            GeneratedRow::StoreSales(row) => row.write_to(writer, separator),
            GeneratedRow::TimeDim(row) => row.write_to(writer, separator),
            GeneratedRow::Warehouse(row) => row.write_to(writer, separator),
            GeneratedRow::WebPage(row) => row.write_to(writer, separator),
            GeneratedRow::WebReturns(row) => row.write_to(writer, separator),
            GeneratedRow::WebSales(row) => row.write_to(writer, separator),
            GeneratedRow::WebSite(row) => row.write_to(writer, separator),
        }
    }
}

// Convenience From implementations for easy conversion
impl From<CallCenterRow> for GeneratedRow {
    fn from(row: CallCenterRow) -> Self {
        GeneratedRow::CallCenter(row)
    }
}

impl From<CatalogPageRow> for GeneratedRow {
    fn from(row: CatalogPageRow) -> Self {
        GeneratedRow::CatalogPage(row)
    }
}

impl From<CatalogReturnsRow> for GeneratedRow {
    fn from(row: CatalogReturnsRow) -> Self {
        GeneratedRow::CatalogReturns(row)
    }
}

impl From<CatalogSalesRow> for GeneratedRow {
    fn from(row: CatalogSalesRow) -> Self {
        GeneratedRow::CatalogSales(row)
    }
}

impl From<CustomerRow> for GeneratedRow {
    fn from(row: CustomerRow) -> Self {
        GeneratedRow::Customer(row)
    }
}

impl From<CustomerAddressRow> for GeneratedRow {
    fn from(row: CustomerAddressRow) -> Self {
        GeneratedRow::CustomerAddress(row)
    }
}

impl From<CustomerDemographicsRow> for GeneratedRow {
    fn from(row: CustomerDemographicsRow) -> Self {
        GeneratedRow::CustomerDemographics(row)
    }
}

impl From<DateDimRow> for GeneratedRow {
    fn from(row: DateDimRow) -> Self {
        GeneratedRow::DateDim(row)
    }
}

impl From<DbgenVersionRow> for GeneratedRow {
    fn from(row: DbgenVersionRow) -> Self {
        GeneratedRow::DbgenVersion(row)
    }
}

impl From<HouseholdDemographicsRow> for GeneratedRow {
    fn from(row: HouseholdDemographicsRow) -> Self {
        GeneratedRow::HouseholdDemographics(row)
    }
}

impl From<IncomeBandRow> for GeneratedRow {
    fn from(row: IncomeBandRow) -> Self {
        GeneratedRow::IncomeBand(row)
    }
}

impl From<InventoryRow> for GeneratedRow {
    fn from(row: InventoryRow) -> Self {
        GeneratedRow::Inventory(row)
    }
}

impl From<ItemRow> for GeneratedRow {
    fn from(row: ItemRow) -> Self {
        GeneratedRow::Item(row)
    }
}

impl From<PromotionRow> for GeneratedRow {
    fn from(row: PromotionRow) -> Self {
        GeneratedRow::Promotion(row)
    }
}

impl From<ReasonRow> for GeneratedRow {
    fn from(row: ReasonRow) -> Self {
        GeneratedRow::Reason(row)
    }
}

impl From<ShipModeRow> for GeneratedRow {
    fn from(row: ShipModeRow) -> Self {
        GeneratedRow::ShipMode(row)
    }
}

impl From<StoreRow> for GeneratedRow {
    fn from(row: StoreRow) -> Self {
        GeneratedRow::Store(row)
    }
}

impl From<StoreReturnsRow> for GeneratedRow {
    fn from(row: StoreReturnsRow) -> Self {
        GeneratedRow::StoreReturns(row)
    }
}

impl From<StoreSalesRow> for GeneratedRow {
    fn from(row: StoreSalesRow) -> Self {
        GeneratedRow::StoreSales(row)
    }
}

impl From<TimeDimRow> for GeneratedRow {
    fn from(row: TimeDimRow) -> Self {
        GeneratedRow::TimeDim(row)
    }
}

impl From<WarehouseRow> for GeneratedRow {
    fn from(row: WarehouseRow) -> Self {
        GeneratedRow::Warehouse(row)
    }
}

impl From<WebPageRow> for GeneratedRow {
    fn from(row: WebPageRow) -> Self {
        GeneratedRow::WebPage(row)
    }
}

impl From<WebReturnsRow> for GeneratedRow {
    fn from(row: WebReturnsRow) -> Self {
        GeneratedRow::WebReturns(row)
    }
}

impl From<WebSalesRow> for GeneratedRow {
    fn from(row: WebSalesRow) -> Self {
        GeneratedRow::WebSales(row)
    }
}

impl From<WebSiteRow> for GeneratedRow {
    fn from(row: WebSiteRow) -> Self {
        GeneratedRow::WebSite(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_row_from_call_center() {
        let row = CallCenterRow::builder().build();
        let generated: GeneratedRow = row.into();
        assert!(matches!(generated, GeneratedRow::CallCenter(_)));
    }

    #[test]
    fn test_generated_row_get_values() {
        let row = CallCenterRow::builder().build();
        let generated: GeneratedRow = row.clone().into();

        // Values should be the same whether accessed directly or through enum
        assert_eq!(row.get_values(), generated.get_values());
    }

    #[test]
    fn test_generated_row_write_to() {
        let row = CallCenterRow::builder().build();
        let generated: GeneratedRow = row.clone().into();

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        row.write_to(&mut buf1, '|').unwrap();
        generated.write_to(&mut buf2, '|').unwrap();

        assert_eq!(buf1, buf2);
    }
}
