use crate::game::elemental::TowerElement;

pub struct TowerPreset {
    pub element: TowerElement,
    pub name: &'static str,
    pub base_damage: f32,
    pub base_range: f32,
    pub base_attack_speed: f32,
    pub base_cost: u32,
    pub is_aoe: bool,
    pub aoe_radius: f32,
    pub description: &'static str,
}

pub const TOWER_PRESETS: &[TowerPreset] = &[
    TowerPreset {
        element: TowerElement::Neutral,
        name: "Sentinelle",
        base_damage: 8.0,
        base_range: 140.0,
        base_attack_speed: 1.0,
        base_cost: 50,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Tour basique equilibree",
    },
    TowerPreset {
        element: TowerElement::Fire,
        name: "Tour Inferno",
        base_damage: 10.0,
        base_range: 110.0,
        base_attack_speed: 0.8,
        base_cost: 80,
        is_aoe: true,
        aoe_radius: 45.0,
        description: "Degats de zone de feu",
    },
    TowerPreset {
        element: TowerElement::Water,
        name: "Tour Glacier",
        base_damage: 5.0,
        base_range: 135.0,
        base_attack_speed: 0.8,
        base_cost: 65,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Ralentit les ennemis",
    },
    TowerPreset {
        element: TowerElement::Electric,
        name: "Tour Tesla",
        base_damage: 7.0,
        base_range: 125.0,
        base_attack_speed: 1.5,
        base_cost: 90,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Attaque rapide electrique",
    },
    TowerPreset {
        element: TowerElement::Earth,
        name: "Tour Seisme",
        base_damage: 18.0,
        base_range: 90.0,
        base_attack_speed: 0.35,
        base_cost: 110,
        is_aoe: true,
        aoe_radius: 55.0,
        description: "Degats massifs de zone",
    },
];

pub fn get_preset(element: TowerElement) -> &'static TowerPreset {
    TOWER_PRESETS
        .iter()
        .find(|p| p.element == element)
        .unwrap_or(&TOWER_PRESETS[0])
}
