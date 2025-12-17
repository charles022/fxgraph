use serde::{Deserialize, Serialize};

/// Request payload sent from the client asking for a specific player.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PlayerReq {
    pub id: u32,
}

/// Player profile that flows over the wire encoded as bincode.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerProfile {
    pub id: u32,
    pub name: String,
    pub inventory: Vec<String>,
}

impl PlayerProfile {
    /// Handy helper used by the server to seed the response.
    pub fn demo() -> Self {
        Self {
            id: 7,
            name: "Ash".into(),
            inventory: vec!["potion".into()],
        }
    }

    /// Mutate the profile to simulate client-side changes.
    pub fn add_item(&mut self, item: impl Into<String>) {
        self.inventory.push(item.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_with_bincode() {
        let profile = PlayerProfile::demo();
        let bytes = bincode::serialize(&profile).unwrap();
        let decoded: PlayerProfile = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded, profile);
    }

    #[test]
    fn mutation_adds_inventory_entry() {
        let mut profile = PlayerProfile::demo();
        profile.add_item("ultra-ball");
        assert_eq!(profile.inventory.last(), Some(&"ultra-ball".to_string()));
    }
}
