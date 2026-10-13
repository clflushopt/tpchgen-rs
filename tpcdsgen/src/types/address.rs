#[derive(Debug, Clone, PartialEq, Default)]
pub struct Address {
    suite_number: &'static str,
    street_number: i32,
    street_name1: &'static str,
    street_name2: &'static str,
    street_type: &'static str,
    city: &'static str,
    county: Option<&'static str>,
    state: &'static str,
    country: &'static str,
    zip: i32,
    gmt_offset: i32,
}

/// `street_name1 + " " + street_name2`, formatted on demand to avoid a per-row
/// allocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreetName {
    name1: &'static str,
    name2: &'static str,
}

impl std::fmt::Display for StreetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name1, self.name2)
    }
}

#[allow(clippy::too_many_arguments)]
impl Address {
    pub fn new(
        suite_number: &'static str,
        street_number: i32,
        street_name1: &'static str,
        street_name2: &'static str,
        street_type: &'static str,
        city: &'static str,
        county: Option<&'static str>,
        state: &'static str,
        country: &'static str,
        zip: i32,
        gmt_offset: i32,
    ) -> Self {
        assert!(
            (1..=1000).contains(&street_number),
            "streetNumber is not between 1 and 1000"
        );
        assert!((0..=99999).contains(&zip), "zip is not between 0 and 99999");

        Address {
            suite_number,
            street_number,
            street_name1,
            street_name2,
            street_type,
            city,
            county,
            state,
            country,
            zip,
            gmt_offset,
        }
    }

    pub fn get_street_number(&self) -> i32 {
        self.street_number
    }

    pub fn get_street_name(&self) -> StreetName {
        StreetName {
            name1: self.street_name1,
            name2: self.street_name2,
        }
    }

    pub fn get_suite_number(&self) -> &'static str {
        self.suite_number
    }

    pub fn get_street_type(&self) -> &'static str {
        self.street_type
    }

    pub fn get_city(&self) -> &'static str {
        self.city
    }

    pub fn get_county(&self) -> Option<&'static str> {
        self.county
    }

    pub fn get_state(&self) -> &'static str {
        self.state
    }

    pub fn get_zip(&self) -> i32 {
        self.zip
    }

    pub fn get_country(&self) -> &'static str {
        self.country
    }

    pub fn get_gmt_offset(&self) -> i32 {
        self.gmt_offset
    }

    pub fn get_street_name1(&self) -> &'static str {
        self.street_name1
    }

    pub fn get_street_name2(&self) -> &'static str {
        self.street_name2
    }

    /// Create a new AddressBuilder
    pub fn builder() -> AddressBuilder {
        AddressBuilder::new()
    }

    // Static method to compute city hash (implementation exactly)
    pub fn compute_city_hash(name: &str) -> i32 {
        let mut hash_value = 0i32;
        let mut result = 0i32;

        for ch in name.chars() {
            hash_value = hash_value.wrapping_mul(26);
            hash_value = hash_value.wrapping_add((ch as i32) - ('A' as i32));
            if hash_value > 1000000 {
                hash_value %= 10000;
                result = result.wrapping_add(hash_value);
                hash_value = 0;
            }
        }

        hash_value %= 1000;
        result = result.wrapping_add(hash_value);
        result % 10000 // looking for a 4 digit result
    }

    // Static method to create address for a specific table column (implementation exactly)
    pub fn make_address_for_column(
        table: crate::table::Table,
        stream: &mut crate::random::stream::RandomNumberStream,
        scaling: &crate::config::Scaling,
    ) -> Self {
        use crate::distribution::{
            get_city_at_index, pick_random_city, pick_random_street_name, pick_random_street_type,
            CitiesWeights, FipsCountyDistribution, FipsWeights, StreetNamesWeights,
        };
        use crate::pseudo_table_scaling_infos::PseudoTableScalingInfos;
        use crate::random::RandomValueGenerator;

        let street_number = RandomValueGenerator::generate_uniform_random_int(1, 1000, stream);
        let street_name1 = pick_random_street_name(StreetNamesWeights::Default, stream);
        let street_name2 = pick_random_street_name(StreetNamesWeights::HalfEmpty, stream);
        let street_type = pick_random_street_type(stream);

        let random_int = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
        let suite_number = suite_number_for(random_int);

        let config_table = match table {
            crate::table::Table::CallCenter => crate::config::table::Table::CallCenter,
            crate::table::Table::WebSite => crate::config::table::Table::WebSite,
            crate::table::Table::Warehouse => crate::config::table::Table::Warehouse,
            crate::table::Table::CustomerAddress => crate::config::table::Table::CustomerAddress,
            crate::table::Table::Store => crate::config::table::Table::Store,
            _ => panic!(
                "Table {:?} not yet supported in Address::make_address_for_column",
                table
            ),
        };
        let row_count = scaling.get_row_count(config_table) as i32;
        let city = if table.is_small() {
            let max_cities =
                PseudoTableScalingInfos::get_active_cities_row_count_for_scale(scaling.get_scale())
                    as i32;
            let random_int = RandomValueGenerator::generate_uniform_random_int(
                0,
                if max_cities > row_count {
                    row_count - 1
                } else {
                    max_cities - 1
                },
                stream,
            );
            get_city_at_index(random_int as usize)
        } else {
            pick_random_city(CitiesWeights::UnifiedStepFunction, stream)
        };

        // county is picked from a distribution, based on population and keys the rest
        let region_number = if table.is_small() {
            let max_counties = PseudoTableScalingInfos::get_active_counties_row_count_for_scale(
                scaling.get_scale(),
            ) as i32;
            RandomValueGenerator::generate_uniform_random_int(
                0,
                if max_counties > row_count {
                    row_count - 1
                } else {
                    max_counties - 1
                },
                stream,
            ) as usize
        } else {
            FipsCountyDistribution::pick_random_index(FipsWeights::Uniform, stream)
        };

        let county = FipsCountyDistribution::get_county_at_index(region_number);
        // let county = if table.is_small() {
        //     FipsCountyDistribution::get_county_at_index(region_number)
        //         .unwrap_or("Williamson County")
        // } else {
        //     // stubbed.
        //     FipsCountyDistribution::get_county_at_index(region_number)
        //         .unwrap_or("Williamson County")
        // };

        // match state with the selected region/county
        let state = FipsCountyDistribution::get_state_abbreviation_at_index(region_number);

        // match the zip prefix with the selected region/county
        let mut zip = Self::compute_city_hash(city);

        // 00000 - 00600 are unused. Avoid them
        let zip_prefix = FipsCountyDistribution::get_zip_prefix_at_index(region_number);
        if zip_prefix == 0 && zip < 9400 {
            zip += 600;
        }
        zip += zip_prefix * 10000;

        let gmt_offset = FipsCountyDistribution::get_gmt_offset_at_index(region_number);
        let country = "United States";

        Address::new(
            suite_number,
            street_number,
            street_name1,
            street_name2,
            street_type,
            city,
            Some(county),
            state,
            country,
            zip,
            gmt_offset,
        )
    }
}

/// Precomputed `Suite {n}` strings: odd `i` yields `Suite {(i/2)*10}`, even `i`
/// yields `Suite {letter}` with `letter = (i/2) % 25 + 'A'` (i in 1..=100).
fn suite_number_for(i: i32) -> &'static str {
    if i % 2 == 1 {
        SUITE_NUMBERS[(i / 2) as usize]
    } else {
        SUITE_LETTERS[((i / 2) % 25) as usize]
    }
}

const SUITE_NUMBERS: [&str; 50] = [
    "Suite 0",
    "Suite 10",
    "Suite 20",
    "Suite 30",
    "Suite 40",
    "Suite 50",
    "Suite 60",
    "Suite 70",
    "Suite 80",
    "Suite 90",
    "Suite 100",
    "Suite 110",
    "Suite 120",
    "Suite 130",
    "Suite 140",
    "Suite 150",
    "Suite 160",
    "Suite 170",
    "Suite 180",
    "Suite 190",
    "Suite 200",
    "Suite 210",
    "Suite 220",
    "Suite 230",
    "Suite 240",
    "Suite 250",
    "Suite 260",
    "Suite 270",
    "Suite 280",
    "Suite 290",
    "Suite 300",
    "Suite 310",
    "Suite 320",
    "Suite 330",
    "Suite 340",
    "Suite 350",
    "Suite 360",
    "Suite 370",
    "Suite 380",
    "Suite 390",
    "Suite 400",
    "Suite 410",
    "Suite 420",
    "Suite 430",
    "Suite 440",
    "Suite 450",
    "Suite 460",
    "Suite 470",
    "Suite 480",
    "Suite 490",
];

const SUITE_LETTERS: [&str; 25] = [
    "Suite A", "Suite B", "Suite C", "Suite D", "Suite E", "Suite F", "Suite G", "Suite H",
    "Suite I", "Suite J", "Suite K", "Suite L", "Suite M", "Suite N", "Suite O", "Suite P",
    "Suite Q", "Suite R", "Suite S", "Suite T", "Suite U", "Suite V", "Suite W", "Suite X",
    "Suite Y",
];

/// Builder for Address
#[derive(Debug, Default)]
pub struct AddressBuilder {
    suite_number: Option<&'static str>,
    street_number: Option<i32>,
    street_name1: Option<&'static str>,
    street_name2: Option<&'static str>,
    street_type: Option<&'static str>,
    city: Option<&'static str>,
    county: Option<&'static str>,
    state: Option<&'static str>,
    country: Option<&'static str>,
    zip: Option<i32>,
    gmt_offset: Option<i32>,
}

impl AddressBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn suite_number(mut self, value: &'static str) -> Self {
        self.suite_number = Some(value);
        self
    }

    pub fn street_number(mut self, value: i32) -> Self {
        self.street_number = Some(value);
        self
    }

    pub fn street_name(mut self, value: &'static str) -> Self {
        self.street_name1 = Some(value);
        self
    }

    pub fn street_name1(mut self, value: &'static str) -> Self {
        self.street_name1 = Some(value);
        self
    }

    pub fn street_name2(mut self, value: &'static str) -> Self {
        self.street_name2 = Some(value);
        self
    }

    pub fn street_type(mut self, value: &'static str) -> Self {
        self.street_type = Some(value);
        self
    }

    pub fn city(mut self, value: &'static str) -> Self {
        self.city = Some(value);
        self
    }

    pub fn county(mut self, value: &'static str) -> Self {
        self.county = Some(value);
        self
    }

    pub fn state(mut self, value: &'static str) -> Self {
        self.state = Some(value);
        self
    }

    pub fn country(mut self, value: &'static str) -> Self {
        self.country = Some(value);
        self
    }

    pub fn zip(mut self, value: i32) -> Self {
        self.zip = Some(value);
        self
    }

    pub fn gmt_offset(mut self, value: i32) -> Self {
        self.gmt_offset = Some(value);
        self
    }

    pub fn build(self) -> Address {
        Address {
            suite_number: self.suite_number.unwrap_or_default(),
            street_number: self.street_number.unwrap_or(1),
            street_name1: self.street_name1.unwrap_or_default(),
            street_name2: self.street_name2.unwrap_or_default(),
            street_type: self.street_type.unwrap_or_default(),
            city: self.city.unwrap_or_default(),
            county: self.county,
            state: self.state.unwrap_or_default(),
            country: self.country.unwrap_or_default(),
            zip: self.zip.unwrap_or(0),
            gmt_offset: self.gmt_offset.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let address = Address::new(
            "Suite 100",
            123,
            "Main",
            "Street",
            "St",
            "Anytown",
            Some("AnyCounty"),
            "CA",
            "United States",
            12345,
            -8,
        );

        assert_eq!(address.get_street_number(), 123);
        assert_eq!(address.get_street_name().to_string(), "Main Street");
        assert_eq!(address.get_city(), "Anytown");
        assert_eq!(address.get_zip(), 12345);
    }

    #[test]
    fn test_suite_number_table() {
        for i in 1..=100 {
            let expected = if i % 2 == 1 {
                format!("Suite {}", (i / 2) * 10)
            } else {
                format!("Suite {}", ((i / 2) % 25 + ('A' as i32)) as u8 as char)
            };
            assert_eq!(suite_number_for(i), expected);
        }
    }

    #[test]
    fn test_city_hash() {
        let hash = Address::compute_city_hash("TESTCITY");
        assert!((0..10000).contains(&hash));
    }

    #[test]
    fn test_address_builder() {
        let address = Address::builder()
            .street_number(456)
            .city("TestCity")
            .zip(54321)
            .build();

        assert_eq!(address.get_street_number(), 456);
        assert_eq!(address.get_city(), "TestCity");
        assert_eq!(address.get_zip(), 54321);
    }
}
