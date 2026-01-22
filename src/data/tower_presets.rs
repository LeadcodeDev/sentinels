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
        base_damage: 10.0,
        base_range: 150.0,
        base_attack_speed: 1.0,
        base_cost: 100,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Tour basique equilibree",
    },
    TowerPreset {
        element: TowerElement::Fire,
        name: "Tour Inferno",
        base_damage: 15.0,
        base_range: 120.0,
        base_attack_speed: 0.8,
        base_cost: 150,
        is_aoe: true,
        aoe_radius: 50.0,
        description: "Degats de zone de feu",
    },
    TowerPreset {
        element: TowerElement::Water,
        name: "Tour Glacier",
        base_damage: 6.0,
        base_range: 140.0,
        base_attack_speed: 0.7,
        base_cost: 120,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Ralentit les ennemis",
    },
    TowerPreset {
        element: TowerElement::Electric,
        name: "Tour Tesla",
        base_damage: 12.0,
        base_range: 130.0,
        base_attack_speed: 1.5,
        base_cost: 180,
        is_aoe: false,
        aoe_radius: 0.0,
        description: "Attaque rapide electrique",
    },
    TowerPreset {
        element: TowerElement::Earth,
        name: "Tour Seisme",
        base_damage: 25.0,
        base_range: 100.0,
        base_attack_speed: 0.3,
        base_cost: 200,
        is_aoe: true,
        aoe_radius: 60.0,
        description: "Degats massifs de zone",
    },
];

pub fn get_preset(element: TowerElement) -> &'static TowerPreset {
    TOWER_PRESETS
        .iter()
        .find(|p| p.element == element)
        .unwrap_or(&TOWER_PRESETS[0])
}
