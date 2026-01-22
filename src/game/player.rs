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
    pub fn new() -> Self {
        Self {
            position: Point2D::zero(),
            hp: 100.0,
            max_hp: 100.0,
            attack_damage: 10.0,
            attack_range: 150.0,
            attack_speed: 1.0,
            attack_cooldown: 0.0,
            radius: 20.0,
            element: TowerElement::Neutral,
        }
    }
}
