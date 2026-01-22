use super::elemental::TowerElement;
use super::{Point2D, Projectile, ProjectileSource};

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyShape {
    Triangle,
    Square,
    Pentagon,
    Hexagon,
    Octagon,
}

impl EnemyShape {
    pub fn sides(&self) -> u32 {
        match self {
            EnemyShape::Triangle => 3,
            EnemyShape::Square => 4,
            EnemyShape::Pentagon => 5,
            EnemyShape::Hexagon => 6,
            EnemyShape::Octagon => 8,
        }
    }
}

#[derive(Clone)]
pub struct AppliedElement {
    pub element: TowerElement,
    pub duration: f32,
}

#[derive(Clone)]
pub struct Enemy {
    pub id: usize,
    pub position: Point2D,
    pub shape: EnemyShape,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub attack_cooldown: f32,
    pub gold_value: u32,
    pub radius: f32,
    pub applied_elements: Vec<AppliedElement>,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub is_boss: bool,
}

impl Enemy {
    pub fn new(id: usize, shape: EnemyShape, wave_number: u32, spawn_pos: Point2D) -> Self {
        use crate::data::enemy_types::get_preset;

        let preset = get_preset(shape);
        let hp_scale = 1.0 + 0.15 * wave_number as f32;

        Self {
            id,
            position: spawn_pos,
            shape,
            hp: preset.base_hp * hp_scale,
            max_hp: preset.base_hp * hp_scale,
            speed: preset.base_speed,
            damage: preset.base_damage,
            attack_range: preset.attack_range,
            attack_speed: preset.attack_speed,
            attack_cooldown: 0.0,
            gold_value: preset.gold_value,
            radius: preset.radius,
            applied_elements: Vec::new(),
            slow_factor: 1.0,
            slow_duration: 0.0,
            is_boss: shape == EnemyShape::Octagon,
        }
    }

    pub fn tick(&mut self, dt: f32, center: &Point2D) {
        // Move toward center
        let dx = center.x - self.position.x;
        let dy = center.y - self.position.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 1.0 {
            let effective_speed = self.speed * self.slow_factor * dt;
            self.position.x += dx / dist * effective_speed;
            self.position.y += dy / dist * effective_speed;
        }

        // Tick slow duration
        if self.slow_duration > 0.0 {
            self.slow_duration -= dt;
            if self.slow_duration <= 0.0 {
                self.slow_factor = 1.0;
            }
        }

        // Tick applied elements
        self.applied_elements.retain_mut(|ae| {
            ae.duration -= dt;
            ae.duration > 0.0
        });
    }

    pub fn try_attack(&mut self, player_pos: &Point2D, dt: f32) -> Option<Projectile> {
        let dist = self.position.distance_to(player_pos);
        if dist > self.attack_range {
            return None;
        }

        self.attack_cooldown -= dt;
        if self.attack_cooldown <= 0.0 {
            self.attack_cooldown = 1.0 / self.attack_speed;
            Some(Projectile {
                origin: self.position.clone(),
                target_pos: player_pos.clone(),
                current_pos: self.position.clone(),
                speed: 200.0,
                damage: self.damage,
                element: TowerElement::Neutral,
                source: ProjectileSource::Enemy(self.id),
                is_aoe: false,
                aoe_radius: 0.0,
                lifetime: 3.0,
            })
        } else {
            None
        }
    }

    pub fn take_damage(&mut self, damage: f32, element: TowerElement) {
        self.hp -= damage;

        if element != TowerElement::Neutral {
            // Check for elemental reactions with existing elements
            let _reaction = self.applied_elements.iter().find_map(|ae| {
                super::elemental::ElementalReaction::from_elements(ae.element, element)
            });

            // Apply element
            self.applied_elements.push(AppliedElement {
                element,
                duration: 3.0,
            });

            // Simple reaction effects
            if let Some(reaction) = self.applied_elements.iter().find_map(|ae| {
                if ae.element != element {
                    super::elemental::ElementalReaction::from_elements(ae.element, element)
                } else {
                    None
                }
            }) {
                match reaction {
                    super::elemental::ElementalReaction::Steam => {
                        self.slow_factor = 0.5;
                        self.slow_duration = 2.0;
                    }
                    super::elemental::ElementalReaction::Overload => {
                        self.hp -= damage * 0.5;
                    }
                    super::elemental::ElementalReaction::Magma => {
                        self.hp -= damage * 0.3;
                    }
                    super::elemental::ElementalReaction::Conductor => {
                        self.hp -= damage * 0.4;
                    }
                    super::elemental::ElementalReaction::Erosion => {
                        self.slow_factor = 0.3;
                        self.slow_duration = 3.0;
                    }
                    super::elemental::ElementalReaction::Magnetic => {
                        self.slow_factor = 0.0;
                        self.slow_duration = 1.5;
                    }
                }
                // Consume elements on reaction
                self.applied_elements.clear();
            }
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }
}
