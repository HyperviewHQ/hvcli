use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RackPduOutletDto {
    pub id: Uuid,
    pub name: String,
    #[serde(alias = "outletNumber")]
    pub outlet_number: u64,
}

impl fmt::Display for RackPduOutletDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r"
id            : {}
name          : {}
outlet_number : {}
       ",
            self.id, self.name, self.outlet_number
        )
    }
}
