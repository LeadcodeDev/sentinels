use super::Point2D;
use super::elemental::TowerElement;
use crate::data::tower_defs::{ResolvedAction, UpgradeableProp, get_def};

#[derive(Clone)]
pub struct Tower {
    pub id: usize,
    pub position: Point2D,
    pub element: TowerElement,
    pub name: &'static str,
    pub range: UpgradeableProp,
    pub attack_speed: UpgradeableProp,
    pub attack_cooldown: f32,
    pub actions: Vec<TowerActionState>,
    pub base_cost: u32,
    pub radius: f32,
}

#[derive(Clone)]
pub struct TowerActionState {
    pub action_index: usize,
    pub upgrades: Vec<UpgradeableProp>,
}

/// Identifies a specific upgrade on a tower
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TowerUpgradeId {
    Range,
    AttackSpeed,
    Action {
        action_idx: usize,
        upgrade_idx: usize,
    },
}

impl Tower {
    pub fn from_def(id: usize, element: TowerElement, position: Point2D) -> Self {
        let def = get_def(element);

        let actions: Vec<TowerActionState> = def
            .actions
            .iter()
            .enumerate()
            .map(|(i, action_def)| TowerActionState {
                action_index: i,
                upgrades: action_def.upgrades.iter().map(|u| u.prop.clone()).collect(),
            })
            .collect();

        Self {
            id,
            position,
            element,
            name: def.name,
            range: def.range.clone(),
            attack_speed: def.attack_speed.clone(),
            attack_cooldown: 0.0,
            actions,
            base_cost: def.base_cost,
            radius: 14.0,
        }
    }

    pub fn attack_range(&self) -> f32 {
        self.range.value()
    }

    pub fn attack_speed_value(&self) -> f32 {
        self.attack_speed.value()
    }

    /// Returns all available upgrades with their current state
    pub fn get_upgrades(&self) -> Vec<(TowerUpgradeId, &'static str, &UpgradeableProp)> {
        let def = get_def(self.element);
        let mut result = Vec::new();

        // Range upgrade
        if self.range.max_level > 0 {
            result.push((TowerUpgradeId::Range, "Portee", &self.range));
        }

        // Attack speed upgrade
        if self.attack_speed.max_level > 0 {
            result.push((TowerUpgradeId::AttackSpeed, "Vitesse", &self.attack_speed));
        }

        // Action upgrades
        for (action_idx, action_state) in self.actions.iter().enumerate() {
            if let Some(action_def) = def.actions.get(action_idx) {
                for (upgrade_idx, upgrade_prop) in action_state.upgrades.iter().enumerate() {
                    if let Some(action_upgrade) = action_def.upgrades.get(upgrade_idx) {
                        result.push((
                            TowerUpgradeId::Action {
                                action_idx,
                                upgrade_idx,
                            },
                            action_upgrade.name,
                            upgrade_prop,
                        ));
                    }
                }
            }
        }

        result
    }

    pub fn apply_upgrade(&mut self, upgrade_id: TowerUpgradeId) -> bool {
        match upgrade_id {
            TowerUpgradeId::Range => {
                if self.range.can_upgrade() {
                    self.range.upgrade();
                    true
                } else {
                    false
                }
            }
            TowerUpgradeId::AttackSpeed => {
                if self.attack_speed.can_upgrade() {
                    self.attack_speed.upgrade();
                    true
                } else {
                    false
                }
            }
            TowerUpgradeId::Action {
                action_idx,
                upgrade_idx,
            } => {
                if let Some(action_state) = self.actions.get_mut(action_idx) {
                    if let Some(prop) = action_state.upgrades.get_mut(upgrade_idx) {
                        if prop.can_upgrade() {
                            prop.upgrade();
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    pub fn upgrade_cost(&self, upgrade_id: TowerUpgradeId) -> Option<u32> {
        match upgrade_id {
            TowerUpgradeId::Range => {
                if self.range.can_upgrade() {
                    Some(self.range.cost())
                } else {
                    None
                }
            }
            TowerUpgradeId::AttackSpeed => {
                if self.attack_speed.can_upgrade() {
                    Some(self.attack_speed.cost())
                } else {
                    None
                }
            }
            TowerUpgradeId::Action {
                action_idx,
                upgrade_idx,
            } => self
                .actions
                .get(action_idx)
                .and_then(|a| a.upgrades.get(upgrade_idx))
                .and_then(|p| {
                    if p.can_upgrade() {
                        Some(p.cost())
                    } else {
                        None
                    }
                }),
        }
    }

    /// Resolve current actions with upgrade levels applied
    pub fn resolved_actions(&self) -> Vec<ResolvedAction> {
        let mut def = get_def(self.element);

        // Apply current upgrade levels to the def's actions
        for (action_idx, action_state) in self.actions.iter().enumerate() {
            if let Some(action_def) = def.actions.get_mut(action_idx) {
                for (upgrade_idx, upgrade_prop) in action_state.upgrades.iter().enumerate() {
                    if let Some(action_upgrade) = action_def.upgrades.get_mut(upgrade_idx) {
                        action_upgrade.prop.current_level = upgrade_prop.current_level;
                    }
                }
            }
        }

        def.actions.iter().map(|a| a.resolve()).collect()
    }

    pub fn level(&self) -> u32 {
        1 + self.range.current_level
            + self.attack_speed.current_level
            + self
                .actions
                .iter()
                .flat_map(|a| a.upgrades.iter())
                .map(|u| u.current_level)
                .sum::<u32>()
    }

    pub fn sell_value(&self) -> u32 {
        self.base_cost * self.level() / 2
    }
}
