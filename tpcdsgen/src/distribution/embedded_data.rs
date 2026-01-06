//! Embedded distribution data files.
//!
//! This module embeds all .dst files at compile time, eliminating runtime file I/O.
//! Files are stored as raw bytes (ISO-8859-1 encoded).

/// Get embedded distribution file bytes by filename, returns `None` if the filename
/// is not known.
pub fn get_embedded_distribution(filename: &str) -> Option<&'static [u8]> {
    match filename {
        "adjectives.dst" => Some(include_bytes!("../../data/adjectives.dst")),
        "adverbs.dst" => Some(include_bytes!("../../data/adverbs.dst")),
        "articles.dst" => Some(include_bytes!("../../data/articles.dst")),
        "auxiliaries.dst" => Some(include_bytes!("../../data/auxiliaries.dst")),
        "book_class.dst" => Some(include_bytes!("../../data/book_class.dst")),
        "brand_syllables.dst" => Some(include_bytes!("../../data/brand_syllables.dst")),
        "buy_potential.dst" => Some(include_bytes!("../../data/buy_potential.dst")),
        "calendar.dst" => Some(include_bytes!("../../data/calendar.dst")),
        "call_center_classes.dst" => Some(include_bytes!("../../data/call_center_classes.dst")),
        "call_center_hours.dst" => Some(include_bytes!("../../data/call_center_hours.dst")),
        "call_centers.dst" => Some(include_bytes!("../../data/call_centers.dst")),
        "catalog_page_types.dst" => Some(include_bytes!("../../data/catalog_page_types.dst")),
        "categories.dst" => Some(include_bytes!("../../data/categories.dst")),
        "children_class.dst" => Some(include_bytes!("../../data/children_class.dst")),
        "cities.dst" => Some(include_bytes!("../../data/cities.dst")),
        "colors.dst" => Some(include_bytes!("../../data/colors.dst")),
        "countries.dst" => Some(include_bytes!("../../data/countries.dst")),
        "credit_ratings.dst" => Some(include_bytes!("../../data/credit_ratings.dst")),
        "dep_count.dst" => Some(include_bytes!("../../data/dep_count.dst")),
        "education.dst" => Some(include_bytes!("../../data/education.dst")),
        "electronic_class.dst" => Some(include_bytes!("../../data/electronic_class.dst")),
        "fips.dst" => Some(include_bytes!("../../data/fips.dst")),
        "first_names.dst" => Some(include_bytes!("../../data/first_names.dst")),
        "genders.dst" => Some(include_bytes!("../../data/genders.dst")),
        "home_class.dst" => Some(include_bytes!("../../data/home_class.dst")),
        "hours.dst" => Some(include_bytes!("../../data/hours.dst")),
        "income_band.dst" => Some(include_bytes!("../../data/income_band.dst")),
        "item_current_price.dst" => Some(include_bytes!("../../data/item_current_price.dst")),
        "item_manager_id.dst" => Some(include_bytes!("../../data/item_manager_id.dst")),
        "item_manufact_id.dst" => Some(include_bytes!("../../data/item_manufact_id.dst")),
        "jewelry_class.dst" => Some(include_bytes!("../../data/jewelry_class.dst")),
        "last_names.dst" => Some(include_bytes!("../../data/last_names.dst")),
        "location_types.dst" => Some(include_bytes!("../../data/location_types.dst")),
        "marital_statuses.dst" => Some(include_bytes!("../../data/marital_statuses.dst")),
        "men_class.dst" => Some(include_bytes!("../../data/men_class.dst")),
        "music_class.dst" => Some(include_bytes!("../../data/music_class.dst")),
        "nouns.dst" => Some(include_bytes!("../../data/nouns.dst")),
        "prepositions.dst" => Some(include_bytes!("../../data/prepositions.dst")),
        "purchase_band.dst" => Some(include_bytes!("../../data/purchase_band.dst")),
        "return_reasons.dst" => Some(include_bytes!("../../data/return_reasons.dst")),
        "salutations.dst" => Some(include_bytes!("../../data/salutations.dst")),
        "sentences.dst" => Some(include_bytes!("../../data/sentences.dst")),
        "ship_mode_carrier.dst" => Some(include_bytes!("../../data/ship_mode_carrier.dst")),
        "ship_mode_code.dst" => Some(include_bytes!("../../data/ship_mode_code.dst")),
        "ship_mode_type.dst" => Some(include_bytes!("../../data/ship_mode_type.dst")),
        "shoe_class.dst" => Some(include_bytes!("../../data/shoe_class.dst")),
        "sizes.dst" => Some(include_bytes!("../../data/sizes.dst")),
        "sport_class.dst" => Some(include_bytes!("../../data/sport_class.dst")),
        "street_names.dst" => Some(include_bytes!("../../data/street_names.dst")),
        "street_types.dst" => Some(include_bytes!("../../data/street_types.dst")),
        "syllables.dst" => Some(include_bytes!("../../data/syllables.dst")),
        "terminators.dst" => Some(include_bytes!("../../data/terminators.dst")),
        "top_domains.dst" => Some(include_bytes!("../../data/top_domains.dst")),
        "units.dst" => Some(include_bytes!("../../data/units.dst")),
        "vehicle_count.dst" => Some(include_bytes!("../../data/vehicle_count.dst")),
        "verbs.dst" => Some(include_bytes!("../../data/verbs.dst")),
        "web_page_use.dst" => Some(include_bytes!("../../data/web_page_use.dst")),
        "women_class.dst" => Some(include_bytes!("../../data/women_class.dst")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_files_exist() {
        assert!(get_embedded_distribution("call_centers.dst").is_some());
        assert!(get_embedded_distribution("genders.dst").is_some());
        assert!(get_embedded_distribution("nonexistent.dst").is_none());
    }

    #[test]
    fn test_embedded_content_not_empty() {
        let bytes = get_embedded_distribution("call_centers.dst").unwrap();
        assert!(!bytes.is_empty());
    }
}
