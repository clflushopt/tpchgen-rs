use crate::conversions::{PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Decimal128Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{ItemRow, ItemRowGenerator};

/// Generate [`ItemRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct ItemArrow {
    inner: ItemRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<ItemRow>,
    i_item_sk: PrimitiveArrayRecycler<Int32Type>,
    i_item_id: StringViewArrayRecycler,
    i_rec_start_date: PrimitiveArrayRecycler<Date32Type>,
    i_rec_end_date: PrimitiveArrayRecycler<Date32Type>,
    i_item_desc: StringViewArrayRecycler,
    i_current_price: PrimitiveArrayRecycler<Decimal128Type>,
    i_wholesale_cost: PrimitiveArrayRecycler<Decimal128Type>,
    i_brand_id: PrimitiveArrayRecycler<Int32Type>,
    i_brand: StringViewArrayRecycler,
    i_class_id: PrimitiveArrayRecycler<Int32Type>,
    i_class: StringViewArrayRecycler,
    i_category_id: PrimitiveArrayRecycler<Int32Type>,
    i_category: StringViewArrayRecycler,
    i_manufact_id: PrimitiveArrayRecycler<Int32Type>,
    i_manufact: StringViewArrayRecycler,
    i_size: StringViewArrayRecycler,
    i_formulation: StringViewArrayRecycler,
    i_color: StringViewArrayRecycler,
    i_units: StringViewArrayRecycler,
    i_container: StringViewArrayRecycler,
    i_manager_id: PrimitiveArrayRecycler<Int32Type>,
    i_product_name: StringViewArrayRecycler,
}

impl ItemArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::Item);
        Self {
            inner: ItemRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            i_item_sk: PrimitiveArrayRecycler::new(),
            i_item_id: StringViewArrayRecycler::new(),
            i_rec_start_date: PrimitiveArrayRecycler::new(),
            i_rec_end_date: PrimitiveArrayRecycler::new(),
            i_item_desc: StringViewArrayRecycler::new(),
            i_current_price: PrimitiveArrayRecycler::new(),
            i_wholesale_cost: PrimitiveArrayRecycler::new(),
            i_brand_id: PrimitiveArrayRecycler::new(),
            i_brand: StringViewArrayRecycler::new(),
            i_class_id: PrimitiveArrayRecycler::new(),
            i_class: StringViewArrayRecycler::new(),
            i_category_id: PrimitiveArrayRecycler::new(),
            i_category: StringViewArrayRecycler::new(),
            i_manufact_id: PrimitiveArrayRecycler::new(),
            i_manufact: StringViewArrayRecycler::new(),
            i_size: StringViewArrayRecycler::new(),
            i_formulation: StringViewArrayRecycler::new(),
            i_color: StringViewArrayRecycler::new(),
            i_units: StringViewArrayRecycler::new(),
            i_container: StringViewArrayRecycler::new(),
            i_manager_id: PrimitiveArrayRecycler::new(),
            i_product_name: StringViewArrayRecycler::new(),
        }
    }
    pub fn skip_rows_until_starting_row_number(&mut self, starting_row_number: u64) {
        self.inner
            .skip_rows_until_starting_row_number(starting_row_number);
    }

    /// Generate only source rows `starting_row_number..=ending_row_number`
    /// (1-based, inclusive). The ending row number is clamped to the table's
    /// row count.
    pub fn with_source_row_range(
        mut self,
        starting_row_number: u64,
        ending_row_number: u64,
    ) -> Self {
        self.inner
            .set_source_row_range(starting_row_number, ending_row_number);
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl RecordBatchReader for ItemArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for ItemArrow {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.scratch.clear();
        self.scratch
            .extend(self.inner.by_ref().take(self.batch_size));
        let rows = &self.scratch;
        if rows.is_empty() {
            return None;
        }

        let schema = self.schema();
        let Self {
            i_item_sk,
            i_item_id,
            i_rec_start_date,
            i_rec_end_date,
            i_item_desc,
            i_current_price,
            i_wholesale_cost,
            i_brand_id,
            i_brand,
            i_class_id,
            i_class,
            i_category_id,
            i_category,
            i_manufact_id,
            i_manufact,
            i_size,
            i_formulation,
            i_color,
            i_units,
            i_container,
            i_manager_id,
            i_product_name,
            ..
        } = self;
        let nbm = ItemRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&ItemRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&ItemRow) -> i64| {
            rec.int32(rows, pos, nbm, |r| {
                i32::try_from(f(r)).expect("TPC-DS INTEGER value exceeds i32 range")
            })
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&ItemRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };

        let i_item_sk = sk32(i_item_sk, 0, ItemRow::get_i_item_sk);
        let i_item_id = str(i_item_id, 1, ItemRow::get_i_item_id);
        // IRecStartDateId is at bit 2 which CAN be set (not in not_null_bit_map);
        // check the null bit first, then apply the julian-day < 0 sentinel.
        let i_rec_start_date =
            i_rec_start_date.date32_julian(rows, 2, nbm, ItemRow::get_i_rec_start_date_id);
        // i_rec_end_date ignores the bitmap; only the julian-day < 0 sentinel applies.
        let i_rec_end_date =
            i_rec_end_date.date32_julian(rows, 3, |_| 0, ItemRow::get_i_rec_end_date_id);
        let i_item_desc = str(i_item_desc, 4, ItemRow::get_i_item_desc);
        let i_current_price =
            i_current_price.decimal_7_2(rows, 5, nbm, ItemRow::get_i_current_price);
        let i_wholesale_cost =
            i_wholesale_cost.decimal_7_2(rows, 6, nbm, ItemRow::get_i_wholesale_cost);
        let i_brand_id = int(i_brand_id, 7, ItemRow::get_i_brand_id);
        let i_brand = str(i_brand, 8, ItemRow::get_i_brand);
        let i_class_id = int(i_class_id, 9, ItemRow::get_i_class_id);
        let i_class = str(i_class, 10, ItemRow::get_i_class);
        let i_category_id = int(i_category_id, 11, ItemRow::get_i_category_id);
        let i_category = str(i_category, 12, ItemRow::get_i_category);
        let i_manufact_id = int(i_manufact_id, 13, ItemRow::get_i_manufact_id);
        let i_manufact = str(i_manufact, 14, ItemRow::get_i_manufact);
        let i_size = str(i_size, 15, ItemRow::get_i_size);
        let i_formulation = str(i_formulation, 16, ItemRow::get_i_formulation);
        let i_color = str(i_color, 17, ItemRow::get_i_color);
        let i_units = str(i_units, 18, ItemRow::get_i_units);
        let i_container = str(i_container, 19, ItemRow::get_i_container);
        let i_manager_id = int(i_manager_id, 20, ItemRow::get_i_manager_id);
        let i_product_name = str(i_product_name, 21, ItemRow::get_i_product_name);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(i_item_sk),
                Arc::new(i_item_id),
                Arc::new(i_rec_start_date),
                Arc::new(i_rec_end_date),
                Arc::new(i_item_desc),
                Arc::new(i_current_price),
                Arc::new(i_wholesale_cost),
                Arc::new(i_brand_id),
                Arc::new(i_brand),
                Arc::new(i_class_id),
                Arc::new(i_class),
                Arc::new(i_category_id),
                Arc::new(i_category),
                Arc::new(i_manufact_id),
                Arc::new(i_manufact),
                Arc::new(i_size),
                Arc::new(i_formulation),
                Arc::new(i_color),
                Arc::new(i_units),
                Arc::new(i_container),
                Arc::new(i_manager_id),
                Arc::new(i_product_name),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("i_item_sk", DataType::Int32, false),
        Field::new("i_item_id", DataType::Utf8View, false),
        Field::new("i_rec_start_date", DataType::Date32, true),
        Field::new("i_rec_end_date", DataType::Date32, true),
        Field::new("i_item_desc", DataType::Utf8View, true),
        Field::new("i_current_price", DataType::Decimal128(7, 2), true),
        Field::new("i_wholesale_cost", DataType::Decimal128(7, 2), true),
        Field::new("i_brand_id", DataType::Int32, true),
        Field::new("i_brand", DataType::Utf8View, true),
        Field::new("i_class_id", DataType::Int32, true),
        Field::new("i_class", DataType::Utf8View, true),
        Field::new("i_category_id", DataType::Int32, true),
        Field::new("i_category", DataType::Utf8View, true),
        Field::new("i_manufact_id", DataType::Int32, true),
        Field::new("i_manufact", DataType::Utf8View, true),
        Field::new("i_size", DataType::Utf8View, true),
        Field::new("i_formulation", DataType::Utf8View, true),
        Field::new("i_color", DataType::Utf8View, true),
        Field::new("i_units", DataType::Utf8View, true),
        Field::new("i_container", DataType::Utf8View, true),
        Field::new("i_manager_id", DataType::Int32, true),
        Field::new("i_product_name", DataType::Utf8View, true),
    ]))
}
