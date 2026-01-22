use super::Point2D;
use super::elemental::TowerElement;
use crate::data::tower_presets::get_preset;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TowerUpgradeType {
    Damage,
    Range,
    AttackSpeed,
    AoeRadius,
}

impl TowerUpgradeType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Damage => "Degats",
            Self::Range => "Portee",
            Self::AttackSpeed => "Vitesse",
            Self::AoeRadius => "Zone",
        }
    }
}

#[derive(Clone)]
pub struct TowerUpgrade {
    pub upgrade_type: TowerUpgradeType,
    pub level: u32,
    pub max_level: u32,
}

impl TowerUpgrade {
    pub fn cost(&self) -> u32 {
        30 + self.level * 25
    }

    pub fn bonus_per_level(&self) -> f32 {
        match self.upgrade_type {
            TowerUpgradeType::Damage => 3.0,
            TowerUpgradeType::Range => 15.0,
            TowerUpgradeType::AttackSpeed => 0.15,
            TowerUpgradeType::AoeRadius => 10.0,
        }
    }
}

#[derive(Clone)]
pub struct Tower {
    pub id: usize,
    pub position: Point2D,
    pub element: TowerElement,
    pub level: u32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub attack_cooldown: f32,
    pub is_aoe: bool,
    pub aoe_radius: f32,
    pub radius: f32,
    pub cost: u32,
    pub upgrades: Vec<TowerUpgrade>,
}

impl Tower {
    pub fn from_preset(id: usize, element: TowerElement, position: Point2D) -> Self {
        let preset = get_preset(element);

        let mut upgrades = vec![
            TowerUpgrade {
                upgrade_type: TowerUpgradeType::Damage,
                level: 0,
                max_level: 5,
            },
            TowerUpgrade {
                upgrade_type: TowerUpgradeType::Range,
                level: 0,
                max_level: 5,
            },
            TowerUpgrade {
                upgrade_type: TowerUpgradeType::AttackSpeed,
                level: 0,
                max_level: 5,
            },
        ];

        if preset.is_aoe {
            upgrades.push(TowerUpgrade {
                upgrade_type: TowerUpgradeType::AoeRadius,
                level: 0,
                max_level: 5,
            });
        }

        Self {
            id,
            position,
            element,
            level: 1,
            attack_damage: preset.base_damage,
            attack_range: preset.base_range,
            attack_speed: preset.base_attack_speed,
            attack_cooldown: 0.0,
            is_aoe: preset.is_aoe,
            aoe_radius: preset.aoe_radius,
            radius: 14.0,
            cost: preset.base_cost,
            upgrades,
        }
    }

    pub fn apply_upgrade(&mut self, upgrade_type: TowerUpgradeType) -> Option<u32> {
        let upgrade = self
            .upgrades
            .iter_mut()
            .find(|u| u.upgrade_type == upgrade_type)?;
        if upgrade.level >= upgrade.max_level {
            return None;
        }
        let cost = upgrade.cost();
        let bonus = upgrade.bonus_per_level();
        upgrade.level += 1;

        match upgrade_type {
            TowerUpgradeType::Damage => self.attack_damage += bonus,
            TowerUpgradeType::Range => self.attack_range += bonus,
            TowerUpgradeType::AttackSpeed => self.attack_speed += bonus,
            TowerUpgradeType::AoeRadius => self.aoe_radius += bonus,
        }

        self.level = 1 + self.upgrades.iter().map(|u| u.level).sum::<u32>();
        Some(cost)
    }

    pub fn sell_value(&self) -> u32 {
        self.cost * self.level / 2
    }
}
