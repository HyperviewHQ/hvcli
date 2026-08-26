use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::{DefaultOnNull, serde_as};
use std::fmt;
use uuid::Uuid;

/// A business entity as returned by `GET /api/asset/businessEntities/{id}` (`BusinessEntityDto`).
/// `businessEntityTypeValueId` is deliberately not deserialized; `businessEntityTypeValue` carries
/// the same information in a readable form.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BusinessEntityDto {
    pub id: Uuid,
    pub name: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub business_entity_type_value: String,
    pub access_policy_id: Uuid,
}

impl fmt::Display for BusinessEntityDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id: {}\nname: {}\nbusiness entity type: {}\naccess policy id: {}",
            self.id, self.name, self.business_entity_type_value, self.access_policy_id
        )
    }
}

/// The paged envelope returned when listing assets. Business entities have no collection endpoint
/// of their own, so their ids are discovered through the asset collection.
#[derive(Debug, Deserialize)]
pub struct BusinessEntityAssetListResponse {
    pub data: Vec<BusinessEntityAssetRef>,
    #[serde(rename = "_metadata")]
    pub metadata: BusinessEntityAssetListMetadata,
}

#[derive(Debug, Deserialize)]
pub struct BusinessEntityAssetListMetadata {
    pub total: i64,
}

/// The id and name of a business entity as it appears in the asset collection. The entity's type
/// is not carried here; that comes from the individual business entity endpoint.
#[derive(Debug, Deserialize)]
pub struct BusinessEntityAssetRef {
    pub id: Uuid,
    pub name: String,
}

/// A record that hangs off a business entity — a contact or an address. Both collections are
/// fetched and paged identically, so listing them is one routine; this trait supplies the two
/// things that differ between them.
pub trait BusinessEntityChild: DeserializeOwned {
    /// The parent entity's name is not part of either API response, so it is stamped on after the
    /// record is fetched.
    fn set_business_entity_name(&mut self, name: &str);

    /// The field a single entity's records are sorted by.
    fn sort_key(&self) -> &str;
}

/// A business entity contact (`BusinessEntityContactDto`), from
/// `GET /api/asset/businessEntityContacts/{businessEntityId}`.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BusinessEntityContactDto {
    #[serde(default)]
    pub business_entity_name: String,
    pub id: Uuid,
    pub parent_id: Uuid,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub name: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub phone_number_one: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub phone_number_two: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub email_address: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub note: String,
}

impl BusinessEntityChild for BusinessEntityContactDto {
    fn set_business_entity_name(&mut self, name: &str) {
        name.clone_into(&mut self.business_entity_name);
    }

    fn sort_key(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for BusinessEntityContactDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "business entity name: {}\nid: {}\nparent id: {}\nname: {}\nphone number one: {}\nphone number two: {}\nemail address: {}\nnote: {}",
            self.business_entity_name,
            self.id,
            self.parent_id,
            self.name,
            self.phone_number_one,
            self.phone_number_two,
            self.email_address,
            self.note
        )
    }
}

/// A business entity address (`BusinessEntityAddressDto`), from
/// `GET /api/asset/businessEntityAddresses/{businessEntityId}`. `streetAddressTypeValue` is the
/// readable form of `streetAddressTypeValueId`; the API returns it even though it is absent from
/// the published schema, so it is defaulted rather than required.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BusinessEntityAddressDto {
    #[serde(default)]
    pub business_entity_name: String,
    pub id: Uuid,
    pub parent_id: Uuid,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub description: String,
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub street_address_value: String,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub street_address_type_value: String,
}

impl BusinessEntityChild for BusinessEntityAddressDto {
    fn set_business_entity_name(&mut self, name: &str) {
        name.clone_into(&mut self.business_entity_name);
    }

    fn sort_key(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for BusinessEntityAddressDto {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "business entity name: {}\nid: {}\nparent id: {}\ndescription: {}\nstreet address: {}\nstreet address type: {}",
            self.business_entity_name,
            self.id,
            self.parent_id,
            self.description,
            self.street_address_value,
            self.street_address_type_value
        )
    }
}
