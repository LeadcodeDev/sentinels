pub mod enemy_types;
pub mod tower_presets;

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

pub const SHOP_UPGRADES: &[ShopUpgradeDef] = &[
    ShopUpgradeDef {
        id: "bonus_gold",
        name: "Or de depart",
        description: "+50 or au debut de chaque partie",
        max_level: 10,
        base_cost: 100,
        cost_per_level: 50,
    },
    ShopUpgradeDef {
        id: "bonus_damage",
        name: "Degats du joueur",
        description: "+5% degats de base du joueur",
        max_level: 10,
        base_cost: 150,
        cost_per_level: 75,
    },
    ShopUpgradeDef {
        id: "bonus_range",
        name: "Portee du joueur",
        description: "+10 portee d'attaque du joueur",
        max_level: 5,
        base_cost: 200,
        cost_per_level: 100,
    },
    ShopUpgradeDef {
        id: "bonus_hp",
        name: "Points de vie",
        description: "+20 PV max du joueur",
        max_level: 10,
        base_cost: 100,
        cost_per_level: 50,
    },
    ShopUpgradeDef {
        id: "bonus_gold_earn",
        name: "Or gagne",
        description: "+10% or gagne par ennemi",
        max_level: 5,
        base_cost: 250,
        cost_per_level: 125,
    },
    ShopUpgradeDef {
        id: "shield",
        name: "Bouclier d'energie",
        description: "Bloque les ennemis a distance, +50 PV par niveau",
        max_level: 5,
        base_cost: 100,
        cost_per_level: 200,
    },
    ShopUpgradeDef {
        id: "tower_slots",
        name: "Slots de tourelles",
        description: "+1 emplacement de tourelle par niveau",
        max_level: 10,
        base_cost: 150,
        cost_per_level: 100,
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
