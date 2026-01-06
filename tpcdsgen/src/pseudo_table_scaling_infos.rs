use crate::scaling_info::{ScalingInfo, ScalingModel};

/// Pseudo table scaling information (PseudoTableScalingInfos)
pub struct PseudoTableScalingInfos;

impl PseudoTableScalingInfos {
    pub fn get_concurrent_web_sites() -> ScalingInfo {
        ScalingInfo::new(
            0,
            ScalingModel::Logarithmic,
            &[0, 2, 3, 4, 5, 5, 5, 5, 5, 5],
            0,
        )
        .expect("Failed to create CONCURRENT_WEB_SITES scaling info")
    }

    pub fn get_active_cities() -> ScalingInfo {
        ScalingInfo::new(
            0,
            ScalingModel::Logarithmic,
            &[0, 2, 6, 18, 30, 54, 90, 165, 270, 495],
            0,
        )
        .expect("Failed to create ACTIVE_CITIES scaling info")
    }

    pub fn get_active_counties() -> ScalingInfo {
        ScalingInfo::new(
            0,
            ScalingModel::Logarithmic,
            &[0, 1, 3, 9, 15, 27, 45, 81, 135, 245],
            0,
        )
        .expect("Failed to create ACTIVE_COUNTIES scaling info")
    }

    pub fn get_active_cities_row_count_for_scale(scale: f64) -> i64 {
        Self::get_active_cities()
            .get_row_count_for_scale(scale)
            .expect("Failed to get active cities row count")
    }

    pub fn get_active_counties_row_count_for_scale(scale: f64) -> i64 {
        Self::get_active_counties()
            .get_row_count_for_scale(scale)
            .expect("Failed to get active counties row count")
    }
}
