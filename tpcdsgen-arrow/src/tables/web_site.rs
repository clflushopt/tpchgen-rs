use crate::conversions::{AddressArrayRecycler, PrimitiveArrayRecycler, StringViewArrayRecycler};
use crate::DEFAULT_BATCH_SIZE;
use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Date32Type, Decimal128Type, Field, Int32Type, Schema, SchemaRef};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatchReader;
use std::sync::{Arc, LazyLock};
use tpcdsgen::config::{Session, Table};
use tpcdsgen::row::{WebSiteRow, WebSiteRowGenerator};

/// Generate [`WebSiteRow`]s in [`RecordBatch`] format
///
/// Each column's value and null buffers are recycled from one batch to the
/// next if possible.
///
/// Buffers can be recycled if the consumer drops each [`RecordBatch`] before
/// generating the next one.
pub struct WebSiteArrow {
    inner: WebSiteRowGenerator,
    batch_size: usize,
    // reused allocations across batches
    scratch: Vec<WebSiteRow>,
    web_site_sk: PrimitiveArrayRecycler<Int32Type>,
    web_site_id: StringViewArrayRecycler,
    web_rec_start_date: PrimitiveArrayRecycler<Date32Type>,
    web_rec_end_date: PrimitiveArrayRecycler<Date32Type>,
    web_name: StringViewArrayRecycler,
    web_open_date_sk: PrimitiveArrayRecycler<Int32Type>,
    web_close_date_sk: PrimitiveArrayRecycler<Int32Type>,
    web_class: StringViewArrayRecycler,
    web_manager: StringViewArrayRecycler,
    web_mkt_id: PrimitiveArrayRecycler<Int32Type>,
    web_mkt_class: StringViewArrayRecycler,
    web_mkt_desc: StringViewArrayRecycler,
    web_market_manager: StringViewArrayRecycler,
    web_company_id: PrimitiveArrayRecycler<Int32Type>,
    web_company_name: StringViewArrayRecycler,
    address: AddressArrayRecycler,
    web_tax_percentage: PrimitiveArrayRecycler<Decimal128Type>,
}

impl WebSiteArrow {
    /// Return the schema without initializing a data generator.
    pub fn schema_ref() -> SchemaRef {
        Arc::clone(&SCHEMA)
    }

    pub fn new(session: Session) -> Self {
        let row_count = session.get_scaling().get_row_count(Table::WebSite);
        Self {
            inner: WebSiteRowGenerator::new(session, row_count),
            batch_size: DEFAULT_BATCH_SIZE,
            scratch: Vec::with_capacity(DEFAULT_BATCH_SIZE),
            web_site_sk: PrimitiveArrayRecycler::new(),
            web_site_id: StringViewArrayRecycler::new(),
            web_rec_start_date: PrimitiveArrayRecycler::new(),
            web_rec_end_date: PrimitiveArrayRecycler::new(),
            web_name: StringViewArrayRecycler::new(),
            web_open_date_sk: PrimitiveArrayRecycler::new(),
            web_close_date_sk: PrimitiveArrayRecycler::new(),
            web_class: StringViewArrayRecycler::new(),
            web_manager: StringViewArrayRecycler::new(),
            web_mkt_id: PrimitiveArrayRecycler::new(),
            web_mkt_class: StringViewArrayRecycler::new(),
            web_mkt_desc: StringViewArrayRecycler::new(),
            web_market_manager: StringViewArrayRecycler::new(),
            web_company_id: PrimitiveArrayRecycler::new(),
            web_company_name: StringViewArrayRecycler::new(),
            address: AddressArrayRecycler::new(),
            web_tax_percentage: PrimitiveArrayRecycler::new(),
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

impl RecordBatchReader for WebSiteArrow {
    fn schema(&self) -> SchemaRef {
        Self::schema_ref()
    }
}

impl Iterator for WebSiteArrow {
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
            web_site_sk,
            web_site_id,
            web_rec_start_date,
            web_rec_end_date,
            web_name,
            web_open_date_sk,
            web_close_date_sk,
            web_class,
            web_manager,
            web_mkt_id,
            web_mkt_class,
            web_mkt_desc,
            web_market_manager,
            web_company_id,
            web_company_name,
            address,
            web_tax_percentage,
            ..
        } = self;
        let nbm = WebSiteRow::null_bit_map;
        let sk32 = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebSiteRow) -> i64| {
            rec.int32_sk(rows, pos, nbm, f)
        };
        let int = |rec: &mut PrimitiveArrayRecycler<Int32Type>, pos, f: fn(&WebSiteRow) -> i32| {
            rec.int32(rows, pos, nbm, f)
        };
        let str = |rec: &mut StringViewArrayRecycler, pos, f: fn(&WebSiteRow) -> &str| {
            rec.str(rows, pos, nbm, f)
        };
        let date =
            |rec: &mut PrimitiveArrayRecycler<Date32Type>, pos, f: fn(&WebSiteRow) -> i64| {
                rec.date32_julian(rows, pos, nbm, f)
            };

        let web_site_sk = sk32(web_site_sk, 0, WebSiteRow::get_web_site_sk);
        let web_site_id = str(web_site_id, 1, WebSiteRow::get_web_site_id);
        let web_rec_start_date = date(web_rec_start_date, 2, WebSiteRow::get_web_rec_start_date_id);
        // only the negative-Julian sentinel marks null here, not the bitmap
        let web_rec_end_date =
            web_rec_end_date.date32_julian(rows, 3, |_| 0, WebSiteRow::get_web_rec_end_date_id);
        let web_name = str(web_name, 4, WebSiteRow::web_name);
        let web_open_date_sk = sk32(web_open_date_sk, 5, WebSiteRow::web_open_date);
        let web_close_date_sk = sk32(web_close_date_sk, 6, WebSiteRow::web_close_date);
        let web_class = str(web_class, 7, WebSiteRow::web_class);
        let web_manager = str(web_manager, 8, WebSiteRow::web_manager);
        let web_mkt_id = int(web_mkt_id, 9, WebSiteRow::web_market_id);
        let web_mkt_class = str(web_mkt_class, 10, WebSiteRow::web_market_class);
        let web_mkt_desc = str(web_mkt_desc, 11, WebSiteRow::web_market_desc);
        let web_market_manager = str(web_market_manager, 12, WebSiteRow::web_market_manager);
        let web_company_id = int(web_company_id, 13, WebSiteRow::web_company_id);
        let web_company_name = str(web_company_name, 14, WebSiteRow::web_company_name);
        let address = address.build(rows, 15, nbm, WebSiteRow::web_address);
        let web_tax_percentage =
            web_tax_percentage.decimal_5_2(rows, 25, nbm, |r| *r.web_tax_percentage());

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(web_site_sk),
                Arc::new(web_site_id),
                Arc::new(web_rec_start_date),
                Arc::new(web_rec_end_date),
                Arc::new(web_name),
                Arc::new(web_open_date_sk),
                Arc::new(web_close_date_sk),
                Arc::new(web_class),
                Arc::new(web_manager),
                Arc::new(web_mkt_id),
                Arc::new(web_mkt_class),
                Arc::new(web_mkt_desc),
                Arc::new(web_market_manager),
                Arc::new(web_company_id),
                Arc::new(web_company_name),
                Arc::new(address.street_number),
                Arc::new(address.street_name),
                Arc::new(address.street_type),
                Arc::new(address.suite_number),
                Arc::new(address.city),
                Arc::new(address.county),
                Arc::new(address.state),
                Arc::new(address.zip),
                Arc::new(address.country),
                Arc::new(address.gmt_offset),
                Arc::new(web_tax_percentage),
            ],
        );
        Some(batch)
    }
}

static SCHEMA: LazyLock<SchemaRef> = LazyLock::new(make_schema);

fn make_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("web_site_sk", DataType::Int32, false),
        Field::new("web_site_id", DataType::Utf8View, false),
        Field::new("web_rec_start_date", DataType::Date32, true),
        Field::new("web_rec_end_date", DataType::Date32, true),
        Field::new("web_name", DataType::Utf8View, true),
        Field::new("web_open_date_sk", DataType::Int32, true),
        Field::new("web_close_date_sk", DataType::Int32, true),
        Field::new("web_class", DataType::Utf8View, true),
        Field::new("web_manager", DataType::Utf8View, true),
        Field::new("web_mkt_id", DataType::Int32, true),
        Field::new("web_mkt_class", DataType::Utf8View, true),
        Field::new("web_mkt_desc", DataType::Utf8View, true),
        Field::new("web_market_manager", DataType::Utf8View, true),
        Field::new("web_company_id", DataType::Int32, true),
        Field::new("web_company_name", DataType::Utf8View, true),
        Field::new("web_street_number", DataType::Utf8View, true),
        Field::new("web_street_name", DataType::Utf8View, true),
        Field::new("web_street_type", DataType::Utf8View, true),
        Field::new("web_suite_number", DataType::Utf8View, true),
        Field::new("web_city", DataType::Utf8View, true),
        Field::new("web_county", DataType::Utf8View, true),
        Field::new("web_state", DataType::Utf8View, true),
        Field::new("web_zip", DataType::Utf8View, true),
        Field::new("web_country", DataType::Utf8View, true),
        Field::new("web_gmt_offset", DataType::Decimal128(5, 2), true),
        Field::new("web_tax_percentage", DataType::Decimal128(5, 2), true),
    ]))
}
