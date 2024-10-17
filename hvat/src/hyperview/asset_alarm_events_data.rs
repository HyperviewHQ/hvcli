use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmEventDto {
    pub id: String,
    pub severity: String,
    #[serde(alias = "assetName")]
    pub asset_name: String,
    #[serde(alias = "assetLocationPath")]
    pub asset_location_path: String,
    #[serde(alias = "alarmEventSettingId")]
    pub alarm_event_setting_id: String,
    #[serde(alias = "assetId")]
    pub asset_id: String,
    #[serde(alias = "startTimestamp")]
    pub start_timestamp: String,
    #[serde(alias = "endTimestamp")]
    pub end_timestamp: String,
    #[serde(alias = "acknowledgementState")]
    pub acknowledgement_state: String,
    #[serde(alias = "acknowledgedBy")]
    pub acknowledged_by: String,
    #[serde(alias = "acknowledgedTimestamp")]
    pub acknowledged_timestamp: String,
    #[serde(alias = "closedBy")]
    pub closed_by: String,
    #[serde(alias = "alarmEventCategory")]
    pub alarm_event_category: String,
    #[serde(alias = "isActive")]
    pub is_active: String,
    #[serde(alias = "propertyValues")]
    pub property_values: String,
    #[serde(alias = "textTemplate")]
    pub text_template: String,
}

impl fmt::Display for AlarmEventDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let asset_record = format!(
            r#"
id                     : {},
severity               : {},
asset_name             : {},
asset_location_path    : {},
alarm_event_setting_id : {},
asset_id               : {},
start_timestamp        : {},
end_timestamp          : {},
acknowledgement_state  : {},
acknowledged_by        : {},
acknowledged_timestamp : {},
closed_by              : {},
alarm_event_category   : {},
is_active              : {},
property_values        : {},
text_template          : {},
"#,
            self.id,
            self.severity,
            self.asset_name,
            self.asset_location_path,
            self.alarm_event_setting_id,
            self.asset_id,
            self.start_timestamp,
            self.end_timestamp,
            self.acknowledgement_state,
            self.acknowledged_by,
            self.acknowledged_timestamp,
            self.closed_by,
            self.alarm_event_category,
            self.is_active,
            self.property_values,
            self.text_template,
        );

        write!(f, "{}", asset_record)
    }
}
