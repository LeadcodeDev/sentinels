use crate::game::enemy::EnemyShape;

pub struct EnemyPreset {
    pub shape: EnemyShape,
    pub name: &'static str,
    pub base_hp: f32,
    pub base_speed: f32,
    pub base_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub gold_value: u32,
    pub radius: f32,
}

pub const ENEMY_PRESETS: &[EnemyPreset] = &[
    EnemyPreset {
        shape: EnemyShape::Triangle,
        name: "Eclaireur",
        base_hp: 30.0,
        base_speed: 120.0,
        base_damage: 5.0,
        attack_range: 50.0,
        attack_speed: 0.8,
        gold_value: 10,
        radius: 12.0,
    },
    EnemyPreset {
        shape: EnemyShape::Square,
        name: "Soldat",
        base_hp: 60.0,
        base_speed: 80.0,
        base_damage: 10.0,
        attack_range: 60.0,
        attack_speed: 0.6,
        gold_value: 20,
        radius: 14.0,
    },
    EnemyPreset {
        shape: EnemyShape::Pentagon,
        name: "Tank",
        base_hp: 120.0,
        base_speed: 40.0,
        base_damage: 15.0,
        attack_range: 70.0,
        attack_speed: 0.4,
        gold_value: 40,
        radius: 18.0,
    },
    EnemyPreset {
        shape: EnemyShape::Hexagon,
        name: "Destructeur",
        base_hp: 50.0,
        base_speed: 60.0,
        base_damage: 25.0,
        attack_range: 80.0,
        attack_speed: 1.0,
        gold_value: 30,
        radius: 15.0,
    },
    EnemyPreset {
        shape: EnemyShape::Octagon,
        name: "Boss",
        base_hp: 300.0,
        base_speed: 25.0,
        base_damage: 40.0,
        attack_range: 100.0,
        attack_speed: 0.5,
        gold_value: 100,
        radius: 25.0,
    },
];

pub fn get_preset(shape: EnemyShape) -> &'static EnemyPreset {
    ENEMY_PRESETS
        .iter()
        .find(|p| p.shape == shape)
        .unwrap_or(&ENEMY_PRESETS[0])
}
