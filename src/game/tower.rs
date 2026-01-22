use super::Point2D;
use super::elemental::TowerElement;
use crate::data::tower_presets::get_preset;

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
}

impl Tower {
    pub fn from_preset(id: usize, element: TowerElement, position: Point2D) -> Self {
        let preset = get_preset(element);
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
        }
    }

    pub fn sell_value(&self) -> u32 {
        self.cost * self.level / 2
    }
}
