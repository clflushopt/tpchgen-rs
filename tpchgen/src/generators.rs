use core::fmt;

use crate::dates;
use crate::distribution::Distribution;
use crate::distribution::Distributions;
use crate::random::RandomAlphaNumeric;
use crate::random::RandomBoundedLong;
use crate::random::RandomPhoneNumber;
use crate::random::RowRandomInt;
use crate::text::TextPool;
use std::sync::Arc;

use crate::dates::GenerateUtils;
use crate::random::{RandomBoundedInt, RandomString, RandomStringSequence, RandomText};

/// Generator for Nation table data
pub struct NationGenerator {
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl Default for NationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl NationGenerator {
    /// Creates a new NationGenerator with default distributions and text pool
    pub fn new() -> Self {
        Self::new_with_distributions_and_text_pool(Distributions::default(), TextPool::default())
    }

    /// Creates a NationGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        NationGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the nation rows
    pub fn iter(&self) -> NationGeneratorIterator {
        NationGeneratorIterator::new(self.distributions.nations(), &self.text_pool)
    }
}

impl IntoIterator for NationGenerator {
    type Item = Nation;
    type IntoIter = NationGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The NATION table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nation {
    /// Primary key (0-24)
    pub n_nationkey: i64,
    /// Nation name
    pub n_name: String,
    /// Foreign key to REGION
    pub n_regionkey: i64,
    /// Variable length comment
    pub n_comment: String,
}

impl fmt::Display for Nation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.n_nationkey, self.n_name, self.n_regionkey, self.n_comment
        )
    }
}

impl Nation {
    /// Create a new `nation` record with the specified values.
    pub fn new(n_nationkey: i64, n_name: &str, n_regionkey: i64, n_comment: &str) -> Self {
        Nation {
            n_nationkey,
            n_name: n_name.to_string(),
            n_regionkey,
            n_comment: n_comment.to_string(),
        }
    }
}

/// Iterator that generates Nation rows
pub struct NationGeneratorIterator {
    nations: Distribution,
    comment_random: RandomText,
    index: usize,
}

impl NationGeneratorIterator {
    const COMMENT_AVERAGE_LENGTH: i32 = 72;

    fn new(nations: &Distribution, text_pool: &TextPool) -> Self {
        NationGeneratorIterator {
            nations: nations.clone(),
            comment_random: RandomText::new(
                606179079,
                text_pool,
                Self::COMMENT_AVERAGE_LENGTH as f64,
            ),
            index: 0,
        }
    }
}

impl Iterator for NationGeneratorIterator {
    type Item = Nation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.nations.size() {
            return None;
        }

        let nation = Nation {
            // n_nationkey
            n_nationkey: self.index as i64,
            // n_name
            n_name: self.nations.get_value(self.index).to_string(),
            // n_regionkey
            n_regionkey: self.nations.get_weight(self.index) as i64,
            // n_comment
            n_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(nation)
    }
}

/// The REGION table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    /// Primary key (0-4)
    pub r_regionkey: i64,
    /// Region name (AFRICA, AMERICA, ASIA, EUROPE, MIDDLE EAST)
    pub r_name: String,
    /// Variable length comment
    pub r_comment: String,
}

impl Region {
    /// Creates a new `region` record with the specified values.
    pub fn new(r_regionkey: i64, r_name: &str, r_comment: &str) -> Self {
        Region {
            r_regionkey,
            r_name: r_name.to_string(),
            r_comment: r_comment.to_string(),
        }
    }
}

/// Generator for Region table data
pub struct RegionGenerator {
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl Default for RegionGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RegionGenerator {
    /// Creates a new RegionGenerator with default distributions and text pool
    pub fn new() -> Self {
        Self::new_with_distributions_and_text_pool(Distributions::default(), TextPool::default())
    }

    /// Creates a RegionGenerator with the specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        RegionGenerator {
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the region rows
    pub fn iter(&self) -> RegionGeneratorIterator {
        RegionGeneratorIterator::new(self.distributions.regions().clone(), &self.text_pool)
    }
}

impl IntoIterator for RegionGenerator {
    type Item = Region;
    type IntoIter = RegionGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Region rows
pub struct RegionGeneratorIterator {
    regions: Distribution,
    comment_random: RandomText,
    index: usize,
}

impl RegionGeneratorIterator {
    const COMMENT_AVERAGE_LENGTH: i32 = 72;

    fn new(regions: Distribution, text_pool: &TextPool) -> Self {
        RegionGeneratorIterator {
            regions,
            comment_random: RandomText::new(
                1500869201,
                text_pool,
                Self::COMMENT_AVERAGE_LENGTH as f64,
            ),
            index: 0,
        }
    }
}

impl Iterator for RegionGeneratorIterator {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.regions.size() {
            return None;
        }

        let region = Region {
            r_regionkey: self.index as i64,
            r_name: self.regions.get_value(self.index).to_string(),
            r_comment: self.comment_random.next_value(),
        };

        self.comment_random.row_finished();
        self.index += 1;

        Some(region)
    }
}

/// The PART table
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// Primary key
    pub p_partkey: i64,
    /// Part name
    pub p_name: String,
    /// Part manufacturer
    pub p_mfgr: String,
    /// Part brand
    pub p_brand: String,
    /// Part type
    pub p_type: String,
    /// Part size
    pub p_size: i32,
    /// Part container
    pub p_container: String,
    /// Part retail price
    pub p_retailprice: f64,
    /// Variable length comment
    pub p_comment: String,
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {},{}, {}, {}, {:.2}, {}",
            self.p_partkey,
            self.p_name,
            self.p_mfgr,
            self.p_brand,
            self.p_type,
            self.p_size,
            self.p_container,
            self.p_retailprice,
            self.p_comment
        )
    }
}

/// Generator for Part table data
pub struct PartGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl PartGenerator {
    /// Base scale for part generation
    const SCALE_BASE: i32 = 200_000;

    // Constants for part generation
    const NAME_WORDS: i32 = 5;
    const MANUFACTURER_MIN: i32 = 1;
    const MANUFACTURER_MAX: i32 = 5;
    const BRAND_MIN: i32 = 1;
    const BRAND_MAX: i32 = 5;
    const SIZE_MIN: i32 = 1;
    const SIZE_MAX: i32 = 50;
    const COMMENT_AVERAGE_LENGTH: i32 = 14;
    /// Creates a new PartGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::default(),
            TextPool::default(),
        )
    }

    /// Creates a PartGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        PartGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the part rows
    pub fn iter(&self) -> PartGeneratorIterator {
        PartGeneratorIterator::new(
            &self.distributions,
            self.text_pool.clone(),
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }
}

impl IntoIterator for PartGenerator {
    type Item = Part;
    type IntoIter = PartGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Part rows
pub struct PartGeneratorIterator {
    name_random: RandomStringSequence,
    manufacturer_random: RandomBoundedInt,
    brand_random: RandomBoundedInt,
    type_random: RandomString,
    size_random: RandomBoundedInt,
    container_random: RandomString,
    comment_random: RandomText,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl PartGeneratorIterator {
    fn new(
        distributions: &Distributions,
        text_pool: Arc<TextPool>,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut name_random = RandomStringSequence::new(
            709314158,
            PartGenerator::NAME_WORDS,
            distributions.part_colors(),
        );
        let mut manufacturer_random = RandomBoundedInt::new(
            1,
            PartGenerator::MANUFACTURER_MIN,
            PartGenerator::MANUFACTURER_MAX,
        );
        let mut brand_random =
            RandomBoundedInt::new(46831694, PartGenerator::BRAND_MIN, PartGenerator::BRAND_MAX);
        let mut type_random = RandomString::new(1841581359, distributions.part_types());
        let mut size_random =
            RandomBoundedInt::new(1193163244, PartGenerator::SIZE_MIN, PartGenerator::SIZE_MAX);
        let mut container_random = RandomString::new(727633698, distributions.part_containers());
        let mut comment_random = RandomText::new(
            804159733,
            &text_pool,
            PartGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance all generators to the starting position
        name_random.advance_rows(start_index);
        manufacturer_random.advance_rows(start_index);
        brand_random.advance_rows(start_index);
        type_random.advance_rows(start_index);
        size_random.advance_rows(start_index);
        container_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        PartGeneratorIterator {
            name_random,
            manufacturer_random,
            brand_random,
            type_random,
            size_random,
            container_random,
            comment_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a part with the given key
    fn make_part(&mut self, part_key: i64) -> Part {
        let name = self.name_random.next_value();

        let manufacturer = self.manufacturer_random.next_value();
        let brand = manufacturer * 10 + self.brand_random.next_value();

        Part {
            p_partkey: part_key,
            p_name: name,
            p_mfgr: format!("Manufacturer#{}", manufacturer),
            p_brand: format!("Brand#{}", brand),
            p_type: self.type_random.next_value(),
            p_size: self.size_random.next_value(),
            p_container: self.container_random.next_value(),
            p_retailprice: Self::calculate_part_price(part_key) as f64 / 100.0,
            p_comment: self.comment_random.next_value(),
        }
    }

    /// Calculates the price for a part
    pub fn calculate_part_price(part_key: i64) -> i64 {
        let mut price = 90000;

        // limit contribution to $200
        price += (part_key / 10) % 20001;
        price += (part_key % 1000) * 100;

        price
    }
}

impl Iterator for PartGeneratorIterator {
    type Item = Part;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let part = self.make_part(self.start_index + self.index + 1);

        self.name_random.row_finished();
        self.manufacturer_random.row_finished();
        self.brand_random.row_finished();
        self.type_random.row_finished();
        self.size_random.row_finished();
        self.container_random.row_finished();
        self.comment_random.row_finished();

        self.index += 1;

        Some(part)
    }
}

/// Records for the SUPPLIER table.
#[derive(Debug, Clone, PartialEq)]
pub struct Supplier {
    /// Primary key
    pub s_suppkey: i64,
    /// Supplier name
    pub s_name: String,
    /// Supplier address
    pub s_address: String,
    /// Foreign key to NATION
    pub s_nationkey: i64,
    /// Supplier phone number
    pub s_phone: String,
    /// Supplier account balance
    pub s_acctbal: f64,
    /// Variable length comment
    pub s_comment: String,
}

impl fmt::Display for Supplier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {:.2}, {}",
            self.s_suppkey,
            self.s_name,
            self.s_address,
            self.s_nationkey,
            self.s_phone,
            self.s_acctbal,
            self.s_comment
        )
    }
}

/// Generator for Supplier table data
pub struct SupplierGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl SupplierGenerator {
    /// Base scale for supplier generation
    const SCALE_BASE: i32 = 10_000;

    // Constants for supplier generation
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: i32 = 25;
    const COMMENT_AVERAGE_LENGTH: i32 = 63;

    // Better Business Bureau comment constants
    pub const BBB_BASE_TEXT: &str = "Customer ";
    pub const BBB_COMPLAINT_TEXT: &str = "Complaints";
    pub const BBB_RECOMMEND_TEXT: &str = "Recommends";
    pub const BBB_COMMENT_LENGTH: usize =
        Self::BBB_BASE_TEXT.len() + Self::BBB_COMPLAINT_TEXT.len();
    pub const BBB_COMMENTS_PER_SCALE_BASE: i32 = 10;
    pub const BBB_COMPLAINT_PERCENT: i32 = 50;

    /// Creates a new SupplierGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::default(),
            TextPool::default(),
        )
    }

    /// Creates a SupplierGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        SupplierGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the supplier rows
    pub fn iter(&self) -> SupplierGeneratorIterator {
        SupplierGeneratorIterator::new(
            &self.distributions,
            self.text_pool.clone(),
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }
}

impl IntoIterator for SupplierGenerator {
    type Item = Supplier;
    type IntoIter = SupplierGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Supplier rows
pub struct SupplierGeneratorIterator {
    address_random: RandomAlphaNumeric,
    nation_key_random: RandomBoundedInt,
    phone_random: RandomPhoneNumber,
    account_balance_random: RandomBoundedInt,
    comment_random: RandomText,
    bbb_comment_random: RandomBoundedInt,
    bbb_junk_random: RowRandomInt,
    bbb_offset_random: RowRandomInt,
    bbb_type_random: RandomBoundedInt,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl SupplierGeneratorIterator {
    fn new(
        distributions: &Distributions,
        text_pool: Arc<TextPool>,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut address_random =
            RandomAlphaNumeric::new(706178559, SupplierGenerator::ADDRESS_AVERAGE_LENGTH);
        let mut nation_key_random =
            RandomBoundedInt::new(110356601, 0, (distributions.nations().size() - 1) as i32);
        let mut phone_random = RandomPhoneNumber::new(884434366);
        let mut account_balance_random = RandomBoundedInt::new(
            962338209,
            SupplierGenerator::ACCOUNT_BALANCE_MIN,
            SupplierGenerator::ACCOUNT_BALANCE_MAX,
        );
        let mut comment_random = RandomText::new(
            1341315363,
            &text_pool,
            SupplierGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );
        let mut bbb_comment_random =
            RandomBoundedInt::new(202794285, 1, SupplierGenerator::SCALE_BASE);
        let mut bbb_junk_random = RowRandomInt::new(263032577, 1);
        let mut bbb_offset_random = RowRandomInt::new(715851524, 1);
        let mut bbb_type_random = RandomBoundedInt::new(753643799, 0, 100);

        // Advance all generators to the starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);
        bbb_comment_random.advance_rows(start_index);
        bbb_junk_random.advance_rows(start_index);
        bbb_offset_random.advance_rows(start_index);
        bbb_type_random.advance_rows(start_index);

        SupplierGeneratorIterator {
            address_random,
            nation_key_random,
            phone_random,
            account_balance_random,
            comment_random,
            bbb_comment_random,
            bbb_junk_random,
            bbb_offset_random,
            bbb_type_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a supplier with the given key
    fn make_supplier(&mut self, supplier_key: i64) -> Supplier {
        let mut comment = self.comment_random.next_value();

        // Add supplier complaints or commendation to the comment
        let bbb_comment_random_value = self.bbb_comment_random.next_value();
        if bbb_comment_random_value <= SupplierGenerator::BBB_COMMENTS_PER_SCALE_BASE {
            let _buffer = comment.clone();

            // select random place for BBB comment
            let noise = self.bbb_junk_random.next_int(
                0,
                (comment.len() - SupplierGenerator::BBB_COMMENT_LENGTH) as i32,
            ) as usize;
            let offset = self.bbb_offset_random.next_int(
                0,
                (comment.len() - (SupplierGenerator::BBB_COMMENT_LENGTH + noise)) as i32,
            ) as usize;

            // select complaint or recommendation
            let type_text =
                if self.bbb_type_random.next_value() < SupplierGenerator::BBB_COMPLAINT_PERCENT {
                    SupplierGenerator::BBB_COMPLAINT_TEXT
                } else {
                    SupplierGenerator::BBB_RECOMMEND_TEXT
                };

            // Create a mutable string that we can modify in chunks
            let mut modified_comment = String::with_capacity(comment.len());
            modified_comment.push_str(&comment[..offset]);
            modified_comment.push_str(SupplierGenerator::BBB_BASE_TEXT);
            modified_comment.push_str(
                &comment[offset + SupplierGenerator::BBB_BASE_TEXT.len()
                    ..offset + SupplierGenerator::BBB_BASE_TEXT.len() + noise],
            );
            modified_comment.push_str(type_text);
            modified_comment.push_str(
                &comment
                    [offset + SupplierGenerator::BBB_BASE_TEXT.len() + noise + type_text.len()..],
            );

            comment = modified_comment;
        }

        let nation_key = self.nation_key_random.next_value() as i64;

        Supplier {
            s_suppkey: supplier_key,
            s_name: format!("Supplier#{:09}", supplier_key),
            s_address: self.address_random.next_value(),
            s_nationkey: nation_key,
            s_phone: self.phone_random.next_value(nation_key),
            s_acctbal: self.account_balance_random.next_value() as f64 / 100.0,
            s_comment: comment,
        }
    }
}

impl Iterator for SupplierGeneratorIterator {
    type Item = Supplier;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let supplier = self.make_supplier(self.start_index + self.index + 1);

        self.address_random.row_finished();
        self.nation_key_random.row_finished();
        self.phone_random.row_finished();
        self.account_balance_random.row_finished();
        self.comment_random.row_finished();
        self.bbb_comment_random.row_finished();
        self.bbb_junk_random.row_finished();
        self.bbb_offset_random.row_finished();
        self.bbb_type_random.row_finished();

        self.index += 1;

        Some(supplier)
    }
}

/// The CUSTOMER table
#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    /// Primary key
    pub c_custkey: i64,
    /// Customer name
    pub c_name: String,
    /// Customer address
    pub c_address: String,
    /// Foreign key to NATION
    pub c_nationkey: i64,
    /// Customer phone number
    pub c_phone: String,
    /// Customer account balance
    pub c_acctbal: f64,
    /// Customer market segment
    pub c_mktsegment: String,
    /// Variable length comment
    pub c_comment: String,
}

impl fmt::Display for Customer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {:.2}, {}, {}",
            self.c_custkey,
            self.c_name,
            self.c_address,
            self.c_nationkey,
            self.c_phone,
            self.c_acctbal,
            self.c_mktsegment,
            self.c_comment
        )
    }
}

/// Generator for Customer table data
pub struct CustomerGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl CustomerGenerator {
    /// Base scale for customer generation
    const SCALE_BASE: i32 = 150_000;

    // Constants for customer generation
    const ACCOUNT_BALANCE_MIN: i32 = -99999;
    const ACCOUNT_BALANCE_MAX: i32 = 999999;
    const ADDRESS_AVERAGE_LENGTH: i32 = 25;
    const COMMENT_AVERAGE_LENGTH: i32 = 73;

    /// Creates a new CustomerGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::default(),
            TextPool::default(),
        )
    }

    /// Creates a CustomerGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        CustomerGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the customer rows
    pub fn iter(&self) -> CustomerGeneratorIterator {
        CustomerGeneratorIterator::new(
            &self.distributions,
            self.text_pool.clone(),
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }
}

impl IntoIterator for CustomerGenerator {
    type Item = Customer;
    type IntoIter = CustomerGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Customer rows
pub struct CustomerGeneratorIterator {
    address_random: RandomAlphaNumeric,
    nation_key_random: RandomBoundedInt,
    phone_random: RandomPhoneNumber,
    account_balance_random: RandomBoundedInt,
    market_segment_random: RandomString,
    comment_random: RandomText,

    start_index: i64,
    row_count: i64,
    index: i64,
}

impl CustomerGeneratorIterator {
    fn new(
        distributions: &Distributions,
        text_pool: Arc<TextPool>,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut address_random =
            RandomAlphaNumeric::new(881155353, CustomerGenerator::ADDRESS_AVERAGE_LENGTH);
        let mut nation_key_random =
            RandomBoundedInt::new(1489529863, 0, (distributions.nations().size() - 1) as i32);
        let mut phone_random = RandomPhoneNumber::new(1521138112);
        let mut account_balance_random = RandomBoundedInt::new(
            298370230,
            CustomerGenerator::ACCOUNT_BALANCE_MIN,
            CustomerGenerator::ACCOUNT_BALANCE_MAX,
        );
        let mut market_segment_random =
            RandomString::new(1140279430, distributions.market_segments());
        let mut comment_random = RandomText::new(
            1335826707,
            &text_pool,
            CustomerGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // Advance all generators to the starting position
        address_random.advance_rows(start_index);
        nation_key_random.advance_rows(start_index);
        phone_random.advance_rows(start_index);
        account_balance_random.advance_rows(start_index);
        market_segment_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        CustomerGeneratorIterator {
            address_random,
            nation_key_random,
            phone_random,
            account_balance_random,
            market_segment_random,
            comment_random,
            start_index,
            row_count,
            index: 0,
        }
    }

    /// Creates a customer with the given key
    fn make_customer(&mut self, customer_key: i64) -> Customer {
        let nation_key = self.nation_key_random.next_value() as i64;

        Customer {
            c_custkey: customer_key,
            c_name: format!("Customer#{:09}", customer_key),
            c_address: self.address_random.next_value(),
            c_nationkey: nation_key,
            c_phone: self.phone_random.next_value(nation_key),
            c_acctbal: self.account_balance_random.next_value() as f64 / 100.0,
            c_mktsegment: self.market_segment_random.next_value(),
            c_comment: self.comment_random.next_value(),
        }
    }
}

impl Iterator for CustomerGeneratorIterator {
    type Item = Customer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let customer = self.make_customer(self.start_index + self.index + 1);

        self.address_random.row_finished();
        self.nation_key_random.row_finished();
        self.phone_random.row_finished();
        self.account_balance_random.row_finished();
        self.market_segment_random.row_finished();
        self.comment_random.row_finished();

        self.index += 1;

        Some(customer)
    }
}

/// The PARTSUPP table
pub struct PartSupp {
    /// Primary key, foreign key to PART
    pub ps_partkey: i64,
    /// Primary key, foreign key to SUPPLIER
    pub ps_suppkey: i64,
    /// Available quantity
    pub ps_availqty: i32,
    /// Supplier cost
    pub ps_supplycost: f64,
    /// Variable length comment
    pub ps_comment: String,
}

impl fmt::Display for PartSupp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {:.2}, {}",
            self.ps_partkey, self.ps_suppkey, self.ps_availqty, self.ps_supplycost, self.ps_comment
        )
    }
}

/// Generator for PartSupplier table data
pub struct PartSupplierGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    text_pool: Arc<TextPool>,
}

impl PartSupplierGenerator {
    /// Base scale for part-supplier generation
    const SUPPLIERS_PER_PART: i32 = 4;

    // Constants for part-supplier generation
    const AVAILABLE_QUANTITY_MIN: i32 = 1;
    const AVAILABLE_QUANTITY_MAX: i32 = 9999;
    const SUPPLY_COST_MIN: i32 = 100;
    const SUPPLY_COST_MAX: i32 = 100000;
    const COMMENT_AVERAGE_LENGTH: i32 = 124;

    /// Creates a new PartSupplierGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_text_pool(scale_factor, part, part_count, TextPool::default())
    }

    /// Creates a PartSupplierGenerator with specified text pool
    pub fn new_with_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        text_pool: Arc<TextPool>,
    ) -> Self {
        PartSupplierGenerator {
            scale_factor,
            part,
            part_count,
            text_pool,
        }
    }

    /// Returns an iterator over the part supplier rows
    pub fn iter(&self) -> PartSupplierGeneratorIterator {
        // Use the part generator's scale base for start/row calculation
        let scale_base = PartGenerator::SCALE_BASE;

        PartSupplierGeneratorIterator::new(
            self.text_pool.clone(),
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                scale_base,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                scale_base,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }
}

impl IntoIterator for PartSupplierGenerator {
    type Item = PartSupp;
    type IntoIter = PartSupplierGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates PartSupplier rows
pub struct PartSupplierGeneratorIterator {
    scale_factor: f64,
    start_index: i64,
    row_count: i64,

    available_quantity_random: RandomBoundedInt,
    supply_cost_random: RandomBoundedInt,
    comment_random: RandomText,

    index: i64,
    part_supplier_number: i32,
}

impl PartSupplierGeneratorIterator {
    fn new(text_pool: Arc<TextPool>, scale_factor: f64, start_index: i64, row_count: i64) -> Self {
        let mut available_quantity_random = RandomBoundedInt::new_with_seeds_per_row(
            1671059989,
            PartSupplierGenerator::AVAILABLE_QUANTITY_MIN,
            PartSupplierGenerator::AVAILABLE_QUANTITY_MAX,
            PartSupplierGenerator::SUPPLIERS_PER_PART,
        );
        let mut supply_cost_random = RandomBoundedInt::new_with_seeds_per_row(
            1051288424,
            PartSupplierGenerator::SUPPLY_COST_MIN,
            PartSupplierGenerator::SUPPLY_COST_MAX,
            PartSupplierGenerator::SUPPLIERS_PER_PART,
        );
        let mut comment_random = RandomText::new_with_expected_row_count(
            1961692154,
            &text_pool,
            PartSupplierGenerator::COMMENT_AVERAGE_LENGTH as f64,
            PartSupplierGenerator::SUPPLIERS_PER_PART,
        );

        // Advance all generators to the starting position
        available_quantity_random.advance_rows(start_index);
        supply_cost_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        PartSupplierGeneratorIterator {
            scale_factor,
            start_index,
            row_count,
            available_quantity_random,
            supply_cost_random,
            comment_random,
            index: 0,
            part_supplier_number: 0,
        }
    }

    /// Creates a part-supplier entry with the given part key
    fn make_part_supplier(&mut self, part_key: i64) -> PartSupp {
        let supplier_key = Self::select_part_supplier(
            part_key,
            self.part_supplier_number as i64,
            self.scale_factor,
        );

        PartSupp {
            ps_partkey: part_key,
            ps_suppkey: supplier_key,
            ps_availqty: self.available_quantity_random.next_value(),
            ps_supplycost: self.supply_cost_random.next_value() as f64 / 100.0,
            ps_comment: self.comment_random.next_value(),
        }
    }

    /// Selects a supplier for a given part and supplier number
    pub fn select_part_supplier(part_key: i64, supplier_number: i64, scale_factor: f64) -> i64 {
        // Use supplier generator's scale base
        let supplier_count = (SupplierGenerator::SCALE_BASE as f64 * scale_factor) as i64;

        ((part_key
            + (supplier_number
                * ((supplier_count / PartSupplierGenerator::SUPPLIERS_PER_PART as i64)
                    + ((part_key - 1) / supplier_count))))
            % supplier_count)
            + 1
    }
}

impl Iterator for PartSupplierGeneratorIterator {
    type Item = PartSupp;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let part_key = self.start_index + self.index + 1;
        let part_supplier = self.make_part_supplier(part_key);
        self.part_supplier_number += 1;

        // advance next row only when all suppliers for the part have been produced
        if self.part_supplier_number >= PartSupplierGenerator::SUPPLIERS_PER_PART {
            self.available_quantity_random.row_finished();
            self.supply_cost_random.row_finished();
            self.comment_random.row_finished();

            self.index += 1;
            self.part_supplier_number = 0;
        }

        Some(part_supplier)
    }
}

/// The ORDERS table
pub struct Order {
    /// Primary key
    pub o_orderkey: i64,
    /// Foreign key to CUSTOMER
    pub o_custkey: i64,
    /// Order status (F=final, O=open, P=pending)
    pub o_orderstatus: char,
    /// Order total price
    pub o_totalprice: f64,
    /// Order date
    pub o_orderdate: String, // Could use a date type instead
    /// Order priority
    pub o_orderpriority: String,
    /// Clerk who processed the order
    pub o_clerk: String,
    /// Order shipping priority
    pub o_shippriority: i32,
    /// Variable length comment
    pub o_comment: String,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {:.2}, {}, {}, {}, {}, {}",
            self.o_orderkey,
            self.o_custkey,
            self.o_orderstatus,
            self.o_totalprice,
            self.o_orderdate,
            self.o_orderpriority,
            self.o_clerk,
            self.o_shippriority,
            self.o_comment
        )
    }
}

/// Generator for Order table data
pub struct OrderGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl OrderGenerator {
    /// Base scale for order generation
    pub const SCALE_BASE: i32 = 1_500_000;

    // Constants for order generation
    const CUSTOMER_MORTALITY: i32 = 3; // portion with no orders
    const ORDER_DATE_MIN: i32 = dates::MIN_GENERATE_DATE;
    const ORDER_DATE_MAX: i32 =
        Self::ORDER_DATE_MIN + (dates::TOTAL_DATE_RANGE - LineItemGenerator::ITEM_SHIP_DAYS - 1);
    const CLERK_SCALE_BASE: i32 = 1000;

    const LINE_COUNT_MIN: i32 = 1;
    pub const LINE_COUNT_MAX: i32 = 7;

    const COMMENT_AVERAGE_LENGTH: i32 = 49;

    const ORDER_KEY_SPARSE_BITS: i32 = 2;
    const ORDER_KEY_SPARSE_KEEP: i32 = 3;
    /// Creates a new OrderGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::default(),
            TextPool::default(),
        )
    }

    /// Creates a OrderGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        OrderGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the order rows
    pub fn iter(&self) -> OrderGeneratorIterator {
        OrderGeneratorIterator::new(
            &self.distributions,
            self.text_pool.clone(),
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                Self::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }

    /// Creates the order date random generator
    pub fn create_order_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1066728069, Self::ORDER_DATE_MIN, Self::ORDER_DATE_MAX)
    }

    /// Creates the line count random generator
    pub fn create_line_count_random() -> RandomBoundedInt {
        RandomBoundedInt::new(1434868289, Self::LINE_COUNT_MIN, Self::LINE_COUNT_MAX)
    }

    /// Creates an order key from an index
    pub fn make_order_key(order_index: i64) -> i64 {
        let low_bits = order_index & ((1 << Self::ORDER_KEY_SPARSE_KEEP) - 1);

        let mut ok = order_index;
        ok >>= Self::ORDER_KEY_SPARSE_KEEP;
        ok <<= Self::ORDER_KEY_SPARSE_BITS;
        ok <<= Self::ORDER_KEY_SPARSE_KEEP;
        ok += low_bits;

        ok
    }
}

impl IntoIterator for OrderGenerator {
    type Item = Order;
    type IntoIter = OrderGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates Order rows
pub struct OrderGeneratorIterator {
    order_date_random: RandomBoundedInt,
    line_count_random: RandomBoundedInt,
    customer_key_random: RandomBoundedLong,
    order_priority_random: RandomString,
    clerk_random: RandomBoundedInt,
    comment_random: RandomText,

    // For line item simulation to determine order status
    line_quantity_random: RandomBoundedInt,
    line_discount_random: RandomBoundedInt,
    line_tax_random: RandomBoundedInt,
    line_part_key_random: RandomBoundedLong,
    line_ship_date_random: RandomBoundedInt,

    start_index: i64,
    row_count: i64,
    max_customer_key: i64,

    index: i64,
}

impl OrderGeneratorIterator {
    fn new(
        distributions: &Distributions,
        text_pool: Arc<TextPool>,
        scale_factor: f64,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut order_date_random = OrderGenerator::create_order_date_random();
        let mut line_count_random = OrderGenerator::create_line_count_random();

        let max_customer_key = (CustomerGenerator::SCALE_BASE as f64 * scale_factor) as i64;

        let mut customer_key_random =
            RandomBoundedLong::new(851767375, scale_factor >= 30000.0, 1, max_customer_key);

        let mut order_priority_random =
            RandomString::new(591449447, distributions.order_priority());

        let max_clerk = (scale_factor * OrderGenerator::CLERK_SCALE_BASE as f64)
            .max(OrderGenerator::CLERK_SCALE_BASE as f64) as i32;
        let mut clerk_random = RandomBoundedInt::new(1171034773, 1, max_clerk);

        let mut comment_random = RandomText::new(
            276090261,
            &text_pool,
            OrderGenerator::COMMENT_AVERAGE_LENGTH as f64,
        );

        // For line item simulation
        let mut line_quantity_random = LineItemGenerator::create_quantity_random();
        let mut line_discount_random = LineItemGenerator::create_discount_random();
        let mut line_tax_random = LineItemGenerator::create_tax_random();
        let mut line_part_key_random = LineItemGenerator::create_part_key_random(scale_factor);
        let mut line_ship_date_random = LineItemGenerator::create_ship_date_random();

        // Advance all generators to the starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);
        customer_key_random.advance_rows(start_index);
        order_priority_random.advance_rows(start_index);
        clerk_random.advance_rows(start_index);
        comment_random.advance_rows(start_index);

        line_quantity_random.advance_rows(start_index);
        line_discount_random.advance_rows(start_index);
        line_tax_random.advance_rows(start_index);
        line_part_key_random.advance_rows(start_index);
        line_ship_date_random.advance_rows(start_index);

        OrderGeneratorIterator {
            order_date_random,
            line_count_random,
            customer_key_random,
            order_priority_random,
            clerk_random,
            comment_random,
            line_quantity_random,
            line_discount_random,
            line_tax_random,
            line_part_key_random,
            line_ship_date_random,
            start_index,
            row_count,
            max_customer_key,
            index: 0,
        }
    }

    /// Creates an order with the given index
    fn make_order(&mut self, index: i64) -> Order {
        let order_key = OrderGenerator::make_order_key(index);

        let order_date = self.order_date_random.next_value();

        // generate customer key, taking into account customer mortality rate
        let mut customer_key = self.customer_key_random.next_value();
        let mut delta = 1;
        while customer_key % OrderGenerator::CUSTOMER_MORTALITY as i64 == 0 {
            customer_key += delta;
            customer_key = customer_key.min(self.max_customer_key);
            delta *= -1;
        }

        let mut total_price = 0 as i64;
        let mut shipped_count = 0;

        let line_count = self.line_count_random.next_value();
        for _ in 0..line_count {
            let quantity = self.line_quantity_random.next_value();
            let discount = self.line_discount_random.next_value();
            let tax = self.line_tax_random.next_value();

            let part_key = self.line_part_key_random.next_value();

            let part_price = PartGeneratorIterator::calculate_part_price(part_key);
            let extended_price = part_price * quantity as i64;
            let discounted_price = extended_price * (100 - discount as i64);
            total_price += ((discounted_price / 100) * (100 + tax as i64)) / 100;

            let ship_date = self.line_ship_date_random.next_value() + order_date;
            if dates::DateUtils::is_in_past(ship_date) {
                shipped_count += 1;
            }
        }

        let order_status = if shipped_count == line_count {
            'F' // Fulfilled - all line items shipped
        } else if shipped_count > 0 {
            'P' // Partially fulfilled - some line items shipped
        } else {
            'O' // Open - no line items shipped
        };

        Order {
            o_orderkey: order_key,
            o_custkey: customer_key,
            o_orderstatus: order_status,
            o_totalprice: total_price as f64 / 100.,
            o_orderdate: dates::DateUtils::to_epoch_date(order_date).to_string(),
            o_orderpriority: self.order_priority_random.next_value(),
            o_clerk: format!("Clerk#{:09}", self.clerk_random.next_value()),
            o_shippriority: 0, // Fixed value per TPC-H spec
            o_comment: self.comment_random.next_value(),
        }
    }
}

impl Iterator for OrderGeneratorIterator {
    type Item = Order;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let order = self.make_order(self.start_index + self.index + 1);

        self.order_date_random.row_finished();
        self.line_count_random.row_finished();
        self.customer_key_random.row_finished();
        self.order_priority_random.row_finished();
        self.clerk_random.row_finished();
        self.comment_random.row_finished();

        self.line_quantity_random.row_finished();
        self.line_discount_random.row_finished();
        self.line_tax_random.row_finished();
        self.line_part_key_random.row_finished();
        self.line_ship_date_random.row_finished();

        self.index += 1;

        Some(order)
    }
}

/// The LINEITEM table
#[derive(Debug, Clone, PartialEq)]
pub struct LineItem {
    /// Foreign key to ORDERS
    pub l_orderkey: i64,
    /// Foreign key to PART
    pub l_partkey: i64,
    /// Foreign key to SUPPLIER
    pub l_suppkey: i64,
    /// Line item number within order
    pub l_linenumber: i32,
    /// Quantity ordered
    pub l_quantity: i64,
    /// Extended price (l_quantity * p_retailprice)
    pub l_extendedprice: f64,
    /// Discount percentage
    pub l_discount: f64,
    /// Tax percentage
    pub l_tax: f64,
    /// Return flag (R=returned, A=accepted, null=pending)
    pub l_returnflag: String,
    /// Line status (O=ordered, F=fulfilled)
    pub l_linestatus: String,
    /// Date shipped
    pub l_shipdate: String, // Could use a date type instead
    /// Date committed to ship
    pub l_commitdate: String, // Could use a date type instead
    /// Date received
    pub l_receiptdate: String, // Could use a date type instead
    /// Shipping instructions
    pub l_shipinstruct: String,
    /// Shipping mode
    pub l_shipmode: String,
    /// Variable length comment
    pub l_comment: String,
}

impl fmt::Display for LineItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {:.2}, {:.2}, {:.2}, {:.2}, {}, {}, {}, {}, {}, {}, {}, {}",
            self.l_orderkey,
            self.l_partkey,
            self.l_suppkey,
            self.l_linenumber,
            self.l_quantity,
            self.l_extendedprice,
            self.l_discount,
            self.l_tax,
            self.l_returnflag,
            self.l_linestatus,
            self.l_shipdate,
            self.l_commitdate,
            self.l_receiptdate,
            self.l_shipinstruct,
            self.l_shipmode,
            self.l_comment
        )
    }
}

/// Generator for LineItem table data
pub struct LineItemGenerator {
    scale_factor: f64,
    part: i32,
    part_count: i32,
    distributions: Distributions,
    text_pool: Arc<TextPool>,
}

impl LineItemGenerator {
    // Constants for line item generation
    const QUANTITY_MIN: i32 = 1;
    const QUANTITY_MAX: i32 = 50;
    const TAX_MIN: i32 = 0;
    const TAX_MAX: i32 = 8;
    const DISCOUNT_MIN: i32 = 0;
    const DISCOUNT_MAX: i32 = 10;
    const PART_KEY_MIN: i32 = 1;

    const SHIP_DATE_MIN: i32 = 1;
    const SHIP_DATE_MAX: i32 = 121;
    const COMMIT_DATE_MIN: i32 = 30;
    const COMMIT_DATE_MAX: i32 = 90;
    const RECEIPT_DATE_MIN: i32 = 1;
    const RECEIPT_DATE_MAX: i32 = 30;

    pub const ITEM_SHIP_DAYS: i32 = Self::SHIP_DATE_MAX + Self::RECEIPT_DATE_MAX;

    const COMMENT_AVERAGE_LENGTH: i32 = 27;

    /// Creates a new LineItemGenerator with the given scale factor
    pub fn new(scale_factor: f64, part: i32, part_count: i32) -> Self {
        Self::new_with_distributions_and_text_pool(
            scale_factor,
            part,
            part_count,
            Distributions::default(),
            TextPool::default(),
        )
    }

    /// Creates a LineItemGenerator with specified distributions and text pool
    pub fn new_with_distributions_and_text_pool(
        scale_factor: f64,
        part: i32,
        part_count: i32,
        distributions: Distributions,
        text_pool: Arc<TextPool>,
    ) -> Self {
        LineItemGenerator {
            scale_factor,
            part,
            part_count,
            distributions,
            text_pool,
        }
    }

    /// Returns an iterator over the line item rows
    pub fn iter(&self) -> LineItemGeneratorIterator {
        LineItemGeneratorIterator::new(
            &self.distributions,
            self.text_pool.clone(),
            self.scale_factor,
            GenerateUtils::calculate_start_index(
                OrderGenerator::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
            GenerateUtils::calculate_row_count(
                OrderGenerator::SCALE_BASE,
                self.scale_factor,
                self.part,
                self.part_count,
            ),
        )
    }

    /// Creates a quantity random generator
    pub fn create_quantity_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            209208115,
            Self::QUANTITY_MIN,
            Self::QUANTITY_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a discount random generator
    pub fn create_discount_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            554590007,
            Self::DISCOUNT_MIN,
            Self::DISCOUNT_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a tax random generator
    pub fn create_tax_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            721958466,
            Self::TAX_MIN,
            Self::TAX_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a part key random generator
    pub fn create_part_key_random(scale_factor: f64) -> RandomBoundedLong {
        // If scale_factor >= 30000, use long `RandomBoundedLong` otherwise
        // use `RandomBoundedInt` to avoid overflow.
        RandomBoundedLong::new_with_seeds_per_row(
            1808217256,
            scale_factor >= 30000.0,
            Self::PART_KEY_MIN as i64,
            (PartGenerator::SCALE_BASE as f64 * scale_factor) as i64,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }

    /// Creates a ship date random generator
    pub fn create_ship_date_random() -> RandomBoundedInt {
        RandomBoundedInt::new_with_seeds_per_row(
            1769349045,
            Self::SHIP_DATE_MIN,
            Self::SHIP_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        )
    }
}

impl IntoIterator for LineItemGenerator {
    type Item = LineItem;
    type IntoIter = LineItemGeneratorIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator that generates LineItem rows
pub struct LineItemGeneratorIterator {
    order_date_random: RandomBoundedInt,
    line_count_random: RandomBoundedInt,

    quantity_random: RandomBoundedInt,
    discount_random: RandomBoundedInt,
    tax_random: RandomBoundedInt,

    line_part_key_random: RandomBoundedLong,

    supplier_number_random: RandomBoundedInt,

    ship_date_random: RandomBoundedInt,
    commit_date_random: RandomBoundedInt,
    receipt_date_random: RandomBoundedInt,

    returned_flag_random: RandomString,
    ship_instructions_random: RandomString,
    ship_mode_random: RandomString,

    comment_random: RandomText,

    scale_factor: f64,
    start_index: i64,
    row_count: i64,

    index: i64,
    order_date: i32,
    line_count: i32,
    line_number: i32,
}

impl LineItemGeneratorIterator {
    fn new(
        distributions: &Distributions,
        text_pool: Arc<TextPool>,
        scale_factor: f64,
        start_index: i64,
        row_count: i64,
    ) -> Self {
        let mut order_date_random = OrderGenerator::create_order_date_random();
        let mut line_count_random = OrderGenerator::create_line_count_random();

        let mut quantity_random = LineItemGenerator::create_quantity_random();
        let mut discount_random = LineItemGenerator::create_discount_random();
        let mut tax_random = LineItemGenerator::create_tax_random();

        let mut line_part_key_random = LineItemGenerator::create_part_key_random(scale_factor);

        let mut supplier_number_random = RandomBoundedInt::new_with_seeds_per_row(
            2095021727,
            0,
            3,
            OrderGenerator::LINE_COUNT_MAX,
        );

        let mut ship_date_random = LineItemGenerator::create_ship_date_random();
        let mut commit_date_random = RandomBoundedInt::new_with_seeds_per_row(
            904914315,
            LineItemGenerator::COMMIT_DATE_MIN,
            LineItemGenerator::COMMIT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut receipt_date_random = RandomBoundedInt::new_with_seeds_per_row(
            373135028,
            LineItemGenerator::RECEIPT_DATE_MIN,
            LineItemGenerator::RECEIPT_DATE_MAX,
            OrderGenerator::LINE_COUNT_MAX,
        );

        let mut returned_flag_random = RandomString::new_with_expected_row_count(
            717419739,
            distributions.return_flags().clone(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut ship_instructions_random = RandomString::new_with_expected_row_count(
            1371272478,
            distributions.ship_instructions().clone(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut ship_mode_random = RandomString::new_with_expected_row_count(
            675466456,
            distributions.ship_modes().clone(),
            OrderGenerator::LINE_COUNT_MAX,
        );
        let mut comment_random = RandomText::new_with_expected_row_count(
            1095462486,
            &text_pool,
            LineItemGenerator::COMMENT_AVERAGE_LENGTH as f64,
            OrderGenerator::LINE_COUNT_MAX,
        );

        // Advance all generators to the starting position
        order_date_random.advance_rows(start_index);
        line_count_random.advance_rows(start_index);

        quantity_random.advance_rows(start_index);
        discount_random.advance_rows(start_index);
        tax_random.advance_rows(start_index);

        line_part_key_random.advance_rows(start_index);

        supplier_number_random.advance_rows(start_index);

        ship_date_random.advance_rows(start_index);
        commit_date_random.advance_rows(start_index);
        receipt_date_random.advance_rows(start_index);

        returned_flag_random.advance_rows(start_index);
        ship_instructions_random.advance_rows(start_index);
        ship_mode_random.advance_rows(start_index);

        comment_random.advance_rows(start_index);

        // generate information for initial order
        let order_date = order_date_random.next_value();
        let line_count = line_count_random.next_value() - 1;

        LineItemGeneratorIterator {
            order_date_random,
            line_count_random,
            quantity_random,
            discount_random,
            tax_random,
            line_part_key_random,
            supplier_number_random,
            ship_date_random,
            commit_date_random,
            receipt_date_random,
            returned_flag_random,
            ship_instructions_random,
            ship_mode_random,
            comment_random,
            scale_factor,
            start_index,
            row_count,
            index: 0,
            order_date,
            line_count,
            line_number: 0,
        }
    }

    /// Creates a line item with the given order index
    fn make_line_item(&mut self, order_index: i64) -> LineItem {
        let order_key = OrderGenerator::make_order_key(order_index);

        let quantity = self.quantity_random.next_value();
        let discount = self.discount_random.next_value();
        let tax = self.tax_random.next_value();

        let part_key = self.line_part_key_random.next_value();

        let supplier_number = self.supplier_number_random.next_value() as i64;
        let supplier_key = PartSupplierGeneratorIterator::select_part_supplier(
            part_key,
            supplier_number,
            self.scale_factor,
        );

        let part_price = PartGeneratorIterator::calculate_part_price(part_key);
        let extended_price = part_price * quantity as i64;

        let mut ship_date = self.ship_date_random.next_value();
        ship_date += self.order_date;
        let mut commit_date = self.commit_date_random.next_value();
        commit_date += self.order_date;
        let mut receipt_date = self.receipt_date_random.next_value();
        receipt_date += ship_date;

        let returned_flag = if dates::DateUtils::is_in_past(receipt_date) {
            self.returned_flag_random.next_value()
        } else {
            "N".to_string()
        };

        let status = if dates::DateUtils::is_in_past(ship_date) {
            "F".to_string() // Fulfilled
        } else {
            "O".to_string() // Open
        };

        let ship_instructions = self.ship_instructions_random.next_value();
        let ship_mode = self.ship_mode_random.next_value();
        let comment = self.comment_random.next_value();

        LineItem {
            l_orderkey: order_key,
            l_partkey: part_key,
            l_suppkey: supplier_key,
            l_linenumber: (self.line_number + 1),
            l_quantity: quantity as i64,
            l_extendedprice: extended_price as f64 / 100.0,
            l_discount: discount as f64 / 100.0,
            l_tax: tax as f64 / 100.0,
            l_returnflag: returned_flag,
            l_linestatus: status,
            l_shipdate: dates::DateUtils::to_epoch_date(ship_date).to_string(),
            l_commitdate: dates::DateUtils::to_epoch_date(commit_date).to_string(),
            l_receiptdate: dates::DateUtils::to_epoch_date(receipt_date).to_string(),
            l_shipinstruct: ship_instructions,
            l_shipmode: ship_mode,
            l_comment: comment,
        }
    }
}

impl Iterator for LineItemGeneratorIterator {
    type Item = LineItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.row_count {
            return None;
        }

        let line_item = self.make_line_item(self.start_index + self.index + 1);
        self.line_number += 1;

        // advance next row only when all lines for the order have been produced
        if self.line_number > self.line_count {
            self.order_date_random.row_finished();
            self.line_count_random.row_finished();

            self.quantity_random.row_finished();
            self.discount_random.row_finished();
            self.tax_random.row_finished();

            self.line_part_key_random.row_finished();
            self.supplier_number_random.row_finished();

            self.ship_date_random.row_finished();
            self.commit_date_random.row_finished();
            self.receipt_date_random.row_finished();

            self.returned_flag_random.row_finished();
            self.ship_instructions_random.row_finished();
            self.ship_mode_random.row_finished();

            self.comment_random.row_finished();

            self.index += 1;

            // generate information for next order
            self.line_count = self.line_count_random.next_value() - 1;
            self.order_date = self.order_date_random.next_value();
            self.line_number = 0;
        }

        Some(line_item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_generator() {
        let generator = NationGenerator::new();
        let nations: Vec<_> = generator.iter().collect();

        // TPC-H typically has 25 nations
        assert_eq!(nations.len(), 25);
    }

    #[test]
    fn test_region_generator() {
        let generator = RegionGenerator::new();
        let regions: Vec<_> = generator.iter().collect();

        // TPC-H typically has 5 regions
        assert_eq!(regions.len(), 5);
    }

    #[test]
    fn test_part_generation() {
        // Create a generator with a small scale factor
        let generator = PartGenerator::new(0.01, 1, 1);
        let parts: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 200,000 = 2,000 parts
        assert_eq!(parts.len(), 2000);
    }

    #[test]
    fn test_calculate_part_price() {
        // Test with a few part keys
        assert_eq!(PartGeneratorIterator::calculate_part_price(1), 90100);
        assert_eq!(PartGeneratorIterator::calculate_part_price(10), 91001);
        assert_eq!(PartGeneratorIterator::calculate_part_price(100), 100010);
        assert_eq!(PartGeneratorIterator::calculate_part_price(1000), 90100);
    }

    #[test]
    fn test_supplier_generation() {
        // Create a generator with a small scale factor
        let generator = SupplierGenerator::new(0.01, 1, 1);
        let suppliers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 10,000 = 100 suppliers
        assert_eq!(suppliers.len(), 100);

        // Check first supplier
        let first = &suppliers[0];
        assert_eq!(first.s_suppkey, 1);
        assert_eq!(first.s_name, "Supplier#000000001");
        assert!(!first.s_address.is_empty());
    }

    #[test]
    fn test_customer_generation() {
        // Create a generator with a small scale factor
        let generator = CustomerGenerator::new(0.01, 1, 1);
        let customers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 150,000 = 1,500 customers
        assert_eq!(customers.len(), 1500);

        // Check first customer
        let first = &customers[0];
        assert_eq!(first.c_custkey, 1);
        assert_eq!(first.c_name, "Customer#000000001");
        assert!(!first.c_address.is_empty());

        // Check market segment distribution
        let market_segments: std::collections::HashSet<_> =
            customers.iter().map(|c| &c.c_mktsegment).collect();

        // Should have multiple different market segments
        assert!(market_segments.len() > 1);

        // Check nation key distribution
        let nation_keys: std::collections::HashSet<_> =
            customers.iter().map(|c| c.c_nationkey).collect();

        // Should have multiple different nation keys
        assert!(nation_keys.len() > 1);
    }

    #[test]
    fn test_part_supplier_generation() {
        // Create a generator with a small scale factor
        let generator = PartSupplierGenerator::new(0.01, 1, 1);
        let part_suppliers: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 200,000 * 4 = 8,000 part-supplier relationships
        assert_eq!(part_suppliers.len(), 8000);

        // Each part should have SUPPLIERS_PER_PART suppliers
        let part_keys: std::collections::HashSet<_> =
            part_suppliers.iter().map(|ps| ps.ps_partkey).collect();

        assert_eq!(part_keys.len(), 2000); // 8,000 / 4 = 2,000 parts

        // Check first part supplier
        let first = &part_suppliers[0];
        assert_eq!(first.ps_partkey, 1);
        assert_ne!(first.ps_suppkey, 0); // Should have a valid supplier key
        assert!(first.ps_availqty > 0);
        assert!(first.ps_supplycost > 0.0);
        assert!(!first.ps_comment.is_empty());

        // Verify supplier distribution
        let suppliers_for_first_part: Vec<_> = part_suppliers
            .iter()
            .filter(|ps| ps.ps_partkey == 1)
            .map(|ps| ps.ps_suppkey)
            .collect();

        assert_eq!(
            suppliers_for_first_part.len(),
            PartSupplierGenerator::SUPPLIERS_PER_PART as usize
        );

        // Supplier keys should be unique for each part
        let unique_suppliers: std::collections::HashSet<_> =
            suppliers_for_first_part.iter().collect();
        assert_eq!(
            unique_suppliers.len(),
            PartSupplierGenerator::SUPPLIERS_PER_PART as usize
        );
    }

    #[test]
    fn test_select_part_supplier() {
        // Test the supplier selection logic for consistency
        let scale_factor = 1.0;

        // Same part with different supplier numbers should yield different suppliers
        let supplier1 = PartSupplierGeneratorIterator::select_part_supplier(1, 0, scale_factor);
        let supplier2 = PartSupplierGeneratorIterator::select_part_supplier(1, 1, scale_factor);
        let supplier3 = PartSupplierGeneratorIterator::select_part_supplier(1, 2, scale_factor);
        let supplier4 = PartSupplierGeneratorIterator::select_part_supplier(1, 3, scale_factor);

        // All suppliers should be different
        let suppliers = vec![supplier1, supplier2, supplier3, supplier4];
        let unique_suppliers: std::collections::HashSet<_> = suppliers.iter().collect();
        assert_eq!(
            unique_suppliers.len(),
            PartSupplierGenerator::SUPPLIERS_PER_PART as usize
        );

        // All supplier keys should be within valid range (1 to supplier_count)
        let supplier_count = (SupplierGenerator::SCALE_BASE as f64 * scale_factor) as i64;
        for supplier in suppliers {
            assert!(supplier >= 1 && supplier <= supplier_count);
        }
    }

    #[test]
    fn test_order_generation() {
        // Create a generator with a small scale factor
        let generator = OrderGenerator::new(0.01, 1, 1);
        let orders: Vec<_> = generator.iter().collect();

        // Should have 0.01 * 1,500,000 = 15,000 orders
        assert_eq!(orders.len(), 15000);

        // Check first order
        let first = &orders[0];
        assert_eq!(first.o_orderkey, OrderGenerator::make_order_key(1));
        assert!(first.o_custkey > 0);
        assert!(
            first.o_orderstatus == 'F' || first.o_orderstatus == 'P' || first.o_orderstatus == 'O'
        );
        assert!(first.o_totalprice > 0.0);

        // Check order status distribution
        let status_counts =
            orders
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, order| {
                    *acc.entry(&order.o_orderstatus).or_insert(0) += 1;
                    acc
                });

        // Should have multiple order statuses
        assert!(status_counts.len() >= 2);

        // Check customer key distribution - no customer with mortality factor
        assert!(orders
            .iter()
            .all(|o| o.o_custkey % OrderGenerator::CUSTOMER_MORTALITY as i64 != 0));

        // Check order key sparsity
        for (i, order) in orders.iter().enumerate() {
            assert_eq!(
                order.o_orderkey,
                OrderGenerator::make_order_key(i as i64 + 1)
            );
        }
    }

    #[test]
    fn test_make_order_key() {
        // Test order key generation logic
        assert_eq!(OrderGenerator::make_order_key(1), 1); // Low values are preserved
        assert_eq!(OrderGenerator::make_order_key(8), 32); // 8 becomes 1000000
        assert_eq!(OrderGenerator::make_order_key(9), 32 + 1); // 9 becomes 1000001
        assert_eq!(OrderGenerator::make_order_key(10), 32 + 2); // 10 becomes 1000010
    }

    #[test]
    fn test_line_item_generation() {
        // Create a generator with a small scale factor
        let generator = LineItemGenerator::new(0.01, 1, 1);
        let line_items: Vec<_> = generator.iter().collect();

        // Check first line item
        let first = &line_items[0];
        assert_eq!(first.l_orderkey, OrderGenerator::make_order_key(1));
        assert_eq!(first.l_linenumber, 1);
        assert!(first.l_partkey > 0);
        assert!(first.l_suppkey > 0);

        assert!(first.l_quantity >= LineItemGenerator::QUANTITY_MIN as i64);
        assert!(first.l_quantity <= LineItemGenerator::QUANTITY_MAX as i64);

        assert!(first.l_discount >= LineItemGenerator::DISCOUNT_MIN as f64 / 100.0);
        assert!(first.l_discount <= LineItemGenerator::DISCOUNT_MAX as f64 / 100.0);

        assert!(first.l_tax >= LineItemGenerator::TAX_MIN as f64 / 100.0);
        assert!(first.l_tax <= LineItemGenerator::TAX_MAX as f64 / 100.0);

        // Verify line numbers are sequential per order
        let mut order_lines = std::collections::HashMap::new();
        for line in &line_items {
            order_lines
                .entry(line.l_orderkey)
                .or_insert_with(Vec::new)
                .push(line.l_linenumber);
        }

        // Check each order's line numbers
        for (_, lines) in order_lines {
            let mut sorted_lines = lines.clone();
            sorted_lines.sort();

            // Line numbers should start at 1 and be sequential
            for (i, line_num) in sorted_lines.iter().enumerate() {
                assert_eq!(*line_num, (i + 1) as i32);
            }
        }

        // Verify return flags and line status distributions
        let return_flags: std::collections::HashSet<_> =
            line_items.iter().map(|l| &l.l_returnflag).collect();

        assert!(return_flags.len() > 1);

        let line_statuses: std::collections::HashSet<_> =
            line_items.iter().map(|l| &l.l_linestatus).collect();

        assert!(!line_statuses.is_empty());
    }
}
