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

//! Join key generation utilities for TPC-DS foreign key relationships.
//!
//! This module provides functionality to generate foreign keys (join keys) between
//! TPC-DS tables, respecting the benchmark's referential integrity requirements.
use crate::config::{Scaling, Table};
use crate::distribution::calendar_distribution::{CalendarDistribution, CalendarWeights};
use crate::distribution::catalog_page_distributions::CatalogPageTypesDistribution;
use crate::distribution::hours_distribution::{HoursDistribution, HoursWeights};
use crate::pseudo_table_scaling_infos::PseudoTableScalingInfos;
use crate::random::{RandomNumberStream, RandomValueGenerator};
use crate::slowly_changing_dimension_utils;
use crate::types::Date;

const WEB_PAGES_PER_SITE: i32 = 123;
const WEB_DATE_STAGGER: i64 = 17;
/// Ship-to-return delay bounds (days) for `store_returns` and `catalog_returns`
pub const CS_MIN_SHIP_DELAY: i32 = 2;
pub const CS_MAX_SHIP_DELAY: i32 = 90;
/// Ship-to-return delay bounds (days) for `web_returns`
pub const WS_MIN_SHIP_DELAY: i32 = 1;
pub const WS_MAX_SHIP_DELAY: i32 = 120;
const CATALOGS_PER_YEAR: i32 = 18;

/// Generates a join key to a table that does not keep history (e.g.
/// `customer`, `promotion`, `ship_mode`): a key drawn uniformly from
/// `1..=row_count`.
///
/// See also [`skip_uniform_join_key`].
pub fn generate_uniform_join_key(
    to_table: Table,
    random_number_stream: &mut RandomNumberStream,
    scaling: &Scaling,
) -> i64 {
    debug_assert!(
        !to_table.keeps_history()
            && !matches!(
                to_table,
                Table::CatalogPage | Table::DateDim | Table::TimeDim
            ),
        "generate_uniform_join_key does not support {to_table:?}"
    );
    RandomValueGenerator::generate_uniform_random_key(
        1,
        i64::try_from(scaling.get_row_count(to_table)).expect("row count fits in i64"),
        random_number_stream,
    )
}

/// Advances `random_number_stream` past the values
/// [`generate_uniform_join_key`] would draw, without computing the key.
pub fn skip_uniform_join_key(random_number_stream: &mut RandomNumberStream) {
    random_number_stream.next_random();
}

/// Generates a join key to a slowly changing dimension (SCD) table (e.g.
/// `store`, `item`, `web_page`).
///
/// SCD tables keep history, so the key must match the version of the dimension
/// in effect on `julian_date`. See  [`crate::slowly_changing_dimension_utils`].
///
/// See also [`skip_scd_join_key`].
pub fn generate_scd_join_key(
    to_table: Table,
    random_number_stream: &mut RandomNumberStream,
    julian_date: i64,
    scaling: &Scaling,
) -> i64 {
    debug_assert!(
        to_table.keeps_history(),
        "generate_scd_join_key does not support {to_table:?}"
    );
    // Can't have a revision in the future
    if julian_date > Date::JULIAN_DATA_END_DATE {
        return -1;
    }

    let id_count = i64::try_from(scaling.get_id_count(to_table)).expect("ID count fits in i64");
    let unique_key =
        RandomValueGenerator::generate_uniform_random_key(1, id_count, random_number_stream);

    // Match the surrogate key based on the julian date for SCD tables
    let key = slowly_changing_dimension_utils::match_surrogate_key(
        unique_key,
        julian_date,
        to_table,
        scaling,
    );

    if key > i64::try_from(scaling.get_row_count(to_table)).expect("row count fits in i64") {
        -1
    } else {
        key
    }
}

/// Advances `random_number_stream` past the values [`generate_scd_join_key`]
/// would draw, without computing the key.
pub fn skip_scd_join_key(julian_date: i64, random_number_stream: &mut RandomNumberStream) {
    if julian_date <= Date::JULIAN_DATA_END_DATE {
        random_number_stream.next_random();
    }
}

/// Generates a join key to the `catalog_page` table.
///
/// Calculates which catalog page based on the date and catalog type (monthly,
/// bi-annual, quarterly). Each catalog type has a different frequency within
/// the year.
///
/// See also [`skip_catalog_page_join_key`].
///
/// Based on JoinKeyUtils.java:generateCatalogPageJoinKey
pub fn generate_catalog_page_join_key(
    random_number_stream: &mut RandomNumberStream,
    julian_date: i64,
    scaling: &Scaling,
) -> i64 {
    let pages_per_catalog = ((scaling.get_row_count(Table::CatalogPage) / CATALOGS_PER_YEAR as u64)
        / (Date::DATE_MAXIMUM.year() - Date::DATE_MINIMUM.year() + 2) as u64)
        as i32;

    let catalog_type =
        CatalogPageTypesDistribution::pick_random_catalog_page_type(random_number_stream);
    let page = RandomValueGenerator::generate_uniform_random_int(
        1,
        pages_per_catalog,
        random_number_stream,
    );

    let offset_from_start = (julian_date - Date::JULIAN_DATA_START_DATE - 1) as i32;
    let mut count = (offset_from_start / 365) * CATALOGS_PER_YEAR;
    let offset = offset_from_start % 365;

    // Adjust count based on catalog type frequency
    match catalog_type.as_str() {
        "bi-annual" => {
            if offset > 183 {
                count += 1;
            }
        }
        "quarterly" => {
            count += offset / 91;
        }
        "monthly" => {
            count += offset / 31;
        }
        other => panic!("Invalid catalog_page_type: {other}"),
    }

    (count * pages_per_catalog + page) as i64
}

/// Advances `random_number_stream` past the values
/// [`generate_catalog_page_join_key`] would draw, without computing the key.
pub fn skip_catalog_page_join_key(random_number_stream: &mut RandomNumberStream) {
    // catalog page type, then page within catalog
    random_number_stream.next_random();
    random_number_stream.next_random();
}

/// Draws the year every `date_dim` join key starts with.
///
/// The reference implementation draws this before deciding how to pick the
/// date, so the returns and web variants draw and discard it to keep the
/// random number stream aligned.
fn pick_random_date_year(random_number_stream: &mut RandomNumberStream) -> i32 {
    RandomValueGenerator::generate_uniform_random_int(
        Date::DATE_MINIMUM.year(),
        Date::DATE_MAXIMUM.year(),
        random_number_stream,
    )
}

/// Generates a `date_dim` join key for a sales table (`store_sales`,
/// `web_sales`): a random year, then a day weighted by sales volume.
///
/// Based on JoinKeyUtils.java:generateDateJoinKey
pub fn generate_sales_date_join_key(random_number_stream: &mut RandomNumberStream) -> i64 {
    let year = pick_random_date_year(random_number_stream);
    let weights = if Date::is_leap_year(year) {
        CalendarWeights::SalesLeapYear
    } else {
        CalendarWeights::Sales
    };

    let day_number = CalendarDistribution::pick_random_day_of_year(weights, random_number_stream);
    let result = Date::to_julian_days(&Date::new(year, 1, 1)) as i64 + day_number as i64;
    if result > Date::JULIAN_TODAYS_DATE as i64 {
        -1
    } else {
        result
    }
}

/// Generates a date join key for returns tables.
///
/// Returns have a lag between the sale date and return date.
/// The lag is calculated as: random(min*2, max*2) days after the sale.
///
/// Store and catalog returns use [`CS_MIN_SHIP_DELAY`]/[`CS_MAX_SHIP_DELAY`],
/// web returns use [`WS_MIN_SHIP_DELAY`]/[`WS_MAX_SHIP_DELAY`].
///
/// Based on JoinKeyUtils.java:generateDateReturnsJoinKey
pub fn generate_returns_date_join_key(
    random_number_stream: &mut RandomNumberStream,
    sale_date: i64,
    min_ship_delay: i32,
    max_ship_delay: i32,
) -> i64 {
    let _year = pick_random_date_year(random_number_stream);
    let lag = RandomValueGenerator::generate_uniform_random_int(
        min_ship_delay * 2,
        max_ship_delay * 2,
        random_number_stream,
    );
    sale_date + lag as i64
}

/// Generates `wp_creation_date_sk` for the `web_page` row `row_number`.
///
/// Page creation happens outside of the page window, to assure a
/// constant number of pages, so it occurs in the gap between site creation
/// and the site's actual activity.
///
/// For sites that are replaced in the time span of the data set, this will
/// depend on whether they are the first version or the second.
///
/// Based on JoinKeyUtils.java:generateWebJoinKey
pub fn generate_web_page_creation_date_join_key(
    random_number_stream: &mut RandomNumberStream,
    row_number: i64,
    scaling: &Scaling,
) -> i64 {
    let _year = pick_random_date_year(random_number_stream);
    let site = (row_number / WEB_PAGES_PER_SITE as i64 + 1) as i32;
    let web_site_duration = get_web_site_duration(scaling);
    let min_result = Date::JULIAN_DATE_MINIMUM as i64
        - ((site as i64 * WEB_DATE_STAGGER) % web_site_duration / 2);
    RandomValueGenerator::generate_uniform_random_int(
        min_result as i32,
        Date::JULIAN_DATE_MINIMUM,
        random_number_stream,
    ) as i64
}

/// Generates `web_open_date` for the `web_site` row `row_number`.
///
/// Based on JoinKeyUtils.java:generateWebJoinKey
pub fn generate_web_open_date_join_key(
    random_number_stream: &mut RandomNumberStream,
    row_number: i64,
    scaling: &Scaling,
) -> i64 {
    let _year = pick_random_date_year(random_number_stream);
    let web_site_duration = get_web_site_duration(scaling);
    Date::JULIAN_DATE_MINIMUM as i64 - ((row_number * WEB_DATE_STAGGER) % web_site_duration / 2)
}

/// Generates `web_close_date` for the `web_site` row `row_number`.
///
/// Based on JoinKeyUtils.java:generateWebJoinKey
pub fn generate_web_close_date_join_key(
    random_number_stream: &mut RandomNumberStream,
    row_number: i64,
    scaling: &Scaling,
) -> i64 {
    let _year = pick_random_date_year(random_number_stream);
    let web_site_duration = get_web_site_duration(scaling);
    let mut result = Date::JULIAN_DATE_MINIMUM as i64
        - ((row_number * WEB_DATE_STAGGER) % web_site_duration / 2);
    result += -web_site_duration; // the -1 here and below are due to undefined values in the C code

    // the site is completely replaced, and this is the first site
    if is_replaced(row_number) && !is_replacement(row_number) {
        // the close date of the first site needs to align on a revision boundary
        result -= -web_site_duration / 2;
    }
    result
}

/// Generates a join key to the `time_dim` table as seconds since midnight
/// (0 to 86399), picking the hour from `weights`:
/// - `store_sales`/`store_returns` use [`HoursWeights::Store`]
/// - catalog and web sales/returns use [`HoursWeights::CatalogAndWeb`]
///
/// Based on JoinKeyUtils.java:generateTimeJoinKey
pub fn generate_time_join_key(
    weights: HoursWeights,
    random_number_stream: &mut RandomNumberStream,
) -> i64 {
    let hour = HoursDistribution::pick_random_hour(weights, random_number_stream);
    let seconds = RandomValueGenerator::generate_uniform_random_int(0, 3599, random_number_stream);

    (hour as i64 * 3600) + seconds as i64
}

/// Calculates the duration of a web site based on concurrent sites.
/// Based on JoinKeyUtils.java:getWebSiteDuration (lines 177-180)
fn get_web_site_duration(scaling: &Scaling) -> i64 {
    let concurrent_web_sites = PseudoTableScalingInfos::get_concurrent_web_sites();
    let row_count = concurrent_web_sites
        .get_row_count_for_scale(scaling.get_scale())
        .expect("Failed to get row count for concurrent web sites");

    (Date::JULIAN_DATE_MAXIMUM as i64 - Date::JULIAN_DATE_MINIMUM as i64) * row_count
}

/// Checks if a web site is replaced (has an even join key).
/// Based on JoinKeyUtils.java:isReplaced (lines 182-185)
fn is_replaced(join_key: i64) -> bool {
    (join_key % 2) == 0
}

/// Checks if a web site is a replacement (odd division by 2).
/// Based on JoinKeyUtils.java:isReplacement (lines 187-190)
fn is_replacement(join_key: i64) -> bool {
    (join_key / 2 % 2) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomNumberStream;

    #[test]
    fn test_is_replaced() {
        assert!(is_replaced(0));
        assert!(is_replaced(2));
        assert!(is_replaced(4));
        assert!(!is_replaced(1));
        assert!(!is_replaced(3));
    }

    #[test]
    fn test_is_replacement() {
        assert!(!is_replacement(0)); // 0/2=0, 0%2=0
        assert!(!is_replacement(1)); // 1/2=0, 0%2=0
        assert!(is_replacement(2)); // 2/2=1, 1%2=1
        assert!(is_replacement(3)); // 3/2=1, 1%2=1
        assert!(!is_replacement(4)); // 4/2=2, 2%2=0
    }

    #[test]
    fn test_generate_time_join_key() {
        let mut stream = RandomNumberStream::new(1);
        let result = generate_time_join_key(HoursWeights::Store, &mut stream);

        // Time keys should be in range [0, 86400) seconds in a day
        assert!(
            (0..86400).contains(&result),
            "Time key should be valid seconds in day"
        );
    }

    #[test]
    fn test_generate_time_join_key_deterministic() {
        let mut stream1 = RandomNumberStream::new(1);
        let mut stream2 = RandomNumberStream::new(1);

        let result1 = generate_time_join_key(HoursWeights::Store, &mut stream1);
        let result2 = generate_time_join_key(HoursWeights::Store, &mut stream2);

        assert_eq!(result1, result2, "Same seed should produce same time key");
    }

    #[test]
    fn test_catalog_page_join_key() {
        let mut stream = RandomNumberStream::new(1);
        let scaling = Scaling::new(1.0);

        let key = generate_catalog_page_join_key(&mut stream, 2451545, &scaling);
        assert!(key > 0, "Key should be positive");
    }

    #[test]
    fn test_generate_returns_date_join_key() {
        let mut stream = RandomNumberStream::new(1);
        let sale_date = Date::to_julian_days(&Date::new(2003, 1, 1)) as i64;

        let return_date = generate_returns_date_join_key(
            &mut stream,
            sale_date,
            CS_MIN_SHIP_DELAY,
            CS_MAX_SHIP_DELAY,
        );

        let lag = return_date - sale_date;
        assert!(lag >= (CS_MIN_SHIP_DELAY * 2) as i64 && lag <= (CS_MAX_SHIP_DELAY * 2) as i64);
    }

    /// Two identical streams, one for a `generate_*` function and one for
    /// its `skip_*` counterpart.
    fn paired_streams() -> (RandomNumberStream, RandomNumberStream) {
        (RandomNumberStream::new(1), RandomNumberStream::new(1))
    }

    #[test]
    fn test_skip_uniform_join_key_matches_generate() {
        let scaling = Scaling::new(1.0);
        let (mut generated, mut skipped) = paired_streams();
        generate_uniform_join_key(Table::Promotion, &mut generated, &scaling);
        skip_uniform_join_key(&mut skipped);
        assert_eq!(generated.next_random(), skipped.next_random());
    }

    #[test]
    fn test_skip_catalog_page_join_key_matches_generate() {
        let scaling = Scaling::new(1.0);
        let date = Date::JULIAN_DATA_START_DATE + 100;
        let (mut generated, mut skipped) = paired_streams();
        generate_catalog_page_join_key(&mut generated, date, &scaling);
        skip_catalog_page_join_key(&mut skipped);
        assert_eq!(generated.next_random(), skipped.next_random());
    }

    #[test]
    fn test_skip_scd_join_key_matches_generate() {
        let scaling = Scaling::new(1.0);
        for date in [
            Date::JULIAN_DATA_START_DATE + 100,
            Date::JULIAN_DATA_END_DATE + 1,
        ] {
            let (mut generated, mut skipped) = paired_streams();
            generate_scd_join_key(Table::WebPage, &mut generated, date, &scaling);
            skip_scd_join_key(date, &mut skipped);
            assert_eq!(generated.next_random(), skipped.next_random(), "{date}");
        }
    }
}
