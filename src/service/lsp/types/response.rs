use super::common::{Location, LocationLink};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DefinitionResult {
    Single(Location),
    Multiple(Vec<Location>),
    MutipleLinks(Vec<LocationLink>),
}

impl DefinitionResult {
    pub fn get_locations(&self) -> Vec<Location> {
        match self {
            DefinitionResult::Single(location) => vec![location.clone()],
            DefinitionResult::Multiple(locations) => locations.clone(),
            DefinitionResult::MutipleLinks(links) => links
                .iter()
                .map(|link| Location {
                    uri: link.target_uri.clone(),
                    range: link.target_range.clone(),
                })
                .collect(),
        }
    }
}
