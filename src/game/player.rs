use super::Point2D;
use super::elemental::TowerElement;

#[derive(Clone)]
pub struct Player {
    pub position: Point2D,
    pub hp: f32,
    pub max_hp: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub attack_cooldown: f32,
    pub radius: f32,
    pub element: TowerElement,
}

impl Player {
    /// Stats de base inspirées d'un Fighter D&D niveau 1
    /// HP: 120 (10 + CON mod × niveau, équivalent à un d10 + bonus)
    /// Dégâts: 12 (1d10 longsword + STR mod)
    /// Portée: 140 (cohérent avec la Sentinelle)
    /// Vitesse d'attaque: 1.2 (Action + Bonus Action)
    pub fn new() -> Self {
        Self {
            position: Point2D::zero(),
            hp: 120.0, // Augmenté de 100 à 120
            max_hp: 120.0,
            attack_damage: 12.0, // Augmenté de 10 à 12
            attack_range: 140.0, // Réduit de 150 à 140 (cohérent avec Sentinelle)
            attack_speed: 1.2,   // Augmenté de 1.0 à 1.2
            attack_cooldown: 0.0,
            radius: 20.0,
            element: TowerElement::Neutral,
        }
    }
}
