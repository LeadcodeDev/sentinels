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

// Stats basées sur le système D&D Challenge Rating (CR)
// Éclaireur: CR 1/4 (Goblin) - Rapide, fragile, dégâts faibles
// Soldat: CR 1 (Orc) - Baseline équilibrée
// Tank: CR 2 (Ogre) - Gros HP, dégâts faibles, très lent
// Destructeur: CR 3 (Knight) - Glass cannon, dégâts élevés
// Boss: CR 5 (Hill Giant) - Menace majeure

pub const ENEMY_PRESETS: &[EnemyPreset] = &[
    EnemyPreset {
        shape: EnemyShape::Triangle,
        name: "Eclaireur",
        base_hp: 15.0,      // CR 1/4: fragile
        base_speed: 140.0,  // Très rapide (bonus mobilité)
        base_damage: 4.0,   // Dégâts faibles
        attack_range: 40.0, // Portée courte (doit s'approcher)
        attack_speed: 0.9,  // Attaque rapide
        gold_value: 5,
        radius: 10.0,
    },
    EnemyPreset {
        shape: EnemyShape::Square,
        name: "Soldat",
        base_hp: 35.0,      // CR 1: baseline solide
        base_speed: 80.0,   // Vitesse moyenne
        base_damage: 8.0,   // Dégâts corrects
        attack_range: 50.0, // Portée standard
        attack_speed: 0.7,  // Rythme régulier
        gold_value: 10,
        radius: 13.0,
    },
    EnemyPreset {
        shape: EnemyShape::Pentagon,
        name: "Tank",
        base_hp: 70.0,      // CR 2: très résistant
        base_speed: 40.0,   // Très lent
        base_damage: 6.0,   // Dégâts faibles (tank, pas DPS)
        attack_range: 55.0, // Portée moyenne
        attack_speed: 0.5,  // Attaque lente
        gold_value: 20,
        radius: 17.0,
    },
    EnemyPreset {
        shape: EnemyShape::Hexagon,
        name: "Destructeur",
        base_hp: 45.0,      // CR 3: HP moyen (glass cannon)
        base_speed: 65.0,   // Assez rapide
        base_damage: 14.0,  // Dégâts très élevés
        attack_range: 70.0, // Bonne portée
        attack_speed: 0.8,  // Attaque rapide
        gold_value: 18,
        radius: 14.0,
    },
    EnemyPreset {
        shape: EnemyShape::Octagon,
        name: "Boss",
        base_hp: 250.0,     // CR 5: menace majeure
        base_speed: 35.0,   // Lent mais inexorable
        base_damage: 20.0,  // Dégâts importants
        attack_range: 80.0, // Bonne portée
        attack_speed: 0.6,  // Rythme soutenu
        gold_value: 80,
        radius: 24.0,
    },
];

pub fn get_preset(shape: EnemyShape) -> &'static EnemyPreset {
    ENEMY_PRESETS
        .iter()
        .find(|p| p.shape == shape)
        .unwrap_or(&ENEMY_PRESETS[0])
}
