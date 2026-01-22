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
        base_hp: 18.0,
        base_speed: 130.0,
        base_damage: 3.0,
        attack_range: 45.0,
        attack_speed: 0.8,
        gold_value: 5,
        radius: 10.0,
    },
    EnemyPreset {
        shape: EnemyShape::Square,
        name: "Soldat",
        base_hp: 40.0,
        base_speed: 85.0,
        base_damage: 6.0,
        attack_range: 55.0,
        attack_speed: 0.6,
        gold_value: 10,
        radius: 13.0,
    },
    EnemyPreset {
        shape: EnemyShape::Pentagon,
        name: "Tank",
        base_hp: 80.0,
        base_speed: 45.0,
        base_damage: 10.0,
        attack_range: 65.0,
        attack_speed: 0.4,
        gold_value: 20,
        radius: 17.0,
    },
    EnemyPreset {
        shape: EnemyShape::Hexagon,
        name: "Destructeur",
        base_hp: 35.0,
        base_speed: 70.0,
        base_damage: 15.0,
        attack_range: 75.0,
        attack_speed: 0.9,
        gold_value: 15,
        radius: 14.0,
    },
    EnemyPreset {
        shape: EnemyShape::Octagon,
        name: "Boss",
        base_hp: 200.0,
        base_speed: 30.0,
        base_damage: 25.0,
        attack_range: 90.0,
        attack_speed: 0.5,
        gold_value: 60,
        radius: 24.0,
    },
];

pub fn get_preset(shape: EnemyShape) -> &'static EnemyPreset {
    ENEMY_PRESETS
        .iter()
        .find(|p| p.shape == shape)
        .unwrap_or(&ENEMY_PRESETS[0])
}
