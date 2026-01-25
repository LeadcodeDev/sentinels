pub mod enemy_types;
pub mod tower_defs;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SaveData {
    pub pepites: u32,
    pub best_score: u32,
    pub max_wave: u32,
    pub shop_upgrades: Vec<ShopUpgradeState>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShopUpgradeState {
    pub id: String,
    pub level: u32,
}

pub struct ShopUpgradeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub max_level: u32,
    base_cost: u32,
    cost_per_level: u32,
}

impl ShopUpgradeDef {
    pub fn cost(&self, current_level: u32) -> u32 {
        self.base_cost + self.cost_per_level * current_level
    }
}

// Upgrades permanents basés sur la progression D&D
// Les bonus sont calibrés pour une progression sur ~20 vagues
// Coûts ajustés pour un rythme d'acquisition cohérent
pub const SHOP_UPGRADES: &[ShopUpgradeDef] = &[
    ShopUpgradeDef {
        id: "bonus_gold",
        name: "Or de depart",
        description: "+40 or au debut de chaque partie", // Réduit de 50 à 40
        max_level: 10,
        base_cost: 80,      // Réduit de 100 (plus accessible)
        cost_per_level: 40, // Réduit de 50
    },
    ShopUpgradeDef {
        id: "bonus_damage",
        name: "Degats du joueur",
        description: "+8% degats de base du joueur", // Augmenté de 5% à 8%
        max_level: 10,
        base_cost: 120,     // Réduit de 150
        cost_per_level: 60, // Réduit de 75
    },
    ShopUpgradeDef {
        id: "bonus_range",
        name: "Portee du joueur",
        description: "+8 portee d'attaque du joueur", // Réduit de 10 à 8
        max_level: 5,
        base_cost: 150,     // Réduit de 200
        cost_per_level: 80, // Réduit de 100
    },
    ShopUpgradeDef {
        id: "bonus_hp",
        name: "Points de vie",
        description: "+15 PV max du joueur", // Réduit de 20 à 15
        max_level: 10,
        base_cost: 80,      // Réduit de 100
        cost_per_level: 40, // Réduit de 50
    },
    ShopUpgradeDef {
        id: "bonus_gold_earn",
        name: "Or gagne",
        description: "+8% or gagne par ennemi", // Réduit de 10% à 8%
        max_level: 5,
        base_cost: 200,      // Réduit de 250
        cost_per_level: 100, // Réduit de 125
    },
    ShopUpgradeDef {
        id: "shield",
        name: "Bouclier d'energie",
        description: "Bloque les ennemis a distance, +40 PV par niveau", // Réduit de 50 à 40
        max_level: 5,
        base_cost: 100,
        cost_per_level: 150, // Réduit de 200
    },
    ShopUpgradeDef {
        id: "tower_slots",
        name: "Slots de tourelles",
        description: "+1 emplacement de tourelle par niveau",
        max_level: 10,
        base_cost: 120,     // Réduit de 150
        cost_per_level: 80, // Réduit de 100
    },
];

impl SaveData {
    pub fn load() -> Self {
        let path = Self::save_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::save_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(path, json).ok();
        }
    }

    pub fn get_upgrade_level(&self, id: &str) -> u32 {
        self.shop_upgrades
            .iter()
            .find(|u| u.id == id)
            .map_or(0, |u| u.level)
    }

    pub fn purchase_upgrade(&mut self, id: &str, cost: u32) {
        if self.pepites < cost {
            return;
        }
        self.pepites -= cost;

        if let Some(upgrade) = self.shop_upgrades.iter_mut().find(|u| u.id == id) {
            upgrade.level += 1;
        } else {
            self.shop_upgrades.push(ShopUpgradeState {
                id: id.to_string(),
                level: 1,
            });
        }
        self.save();
    }

    fn save_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sentinels")
            .join("save.json")
    }
}
