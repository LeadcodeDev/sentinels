use super::Point2D;
use super::enemy::{Enemy, EnemyShape};
use rand::Rng;

struct EnemySpawnInfo {
    shape: EnemyShape,
}

pub struct WaveManager {
    pub current_wave: u32,
    enemies_to_spawn: Vec<EnemySpawnInfo>,
    spawn_timer: f32,
    spawn_interval: f32,
    pub wave_active: bool,
    between_waves_timer: f32,
    between_waves_duration: f32,
    next_enemy_id: usize,
}

impl WaveManager {
    pub fn new() -> Self {
        Self {
            current_wave: 0,
            enemies_to_spawn: Vec::new(),
            spawn_timer: 0.0,
            spawn_interval: 0.5,
            wave_active: false,
            between_waves_timer: 5.0,
            between_waves_duration: 5.0,
            next_enemy_id: 0,
        }
    }

    pub fn start_next_wave(&mut self) {
        self.current_wave += 1;
        self.enemies_to_spawn = self.generate_wave(self.current_wave);
        self.wave_active = true;
        self.spawn_timer = 0.0;
    }

    pub fn start_between_waves(&mut self) {
        self.wave_active = false;
        self.between_waves_timer = self.between_waves_duration;
    }

    pub fn tick_between_waves(&mut self, dt: f32) -> bool {
        self.between_waves_timer -= dt;
        self.between_waves_timer <= 0.0
    }

    pub fn is_wave_complete(&self) -> bool {
        self.wave_active && self.enemies_to_spawn.is_empty()
    }

    pub fn tick(&mut self, dt: f32, viewport: (f32, f32)) -> Vec<Enemy> {
        if !self.wave_active || self.enemies_to_spawn.is_empty() {
            return Vec::new();
        }

        self.spawn_timer -= dt;
        if self.spawn_timer > 0.0 {
            return Vec::new();
        }

        self.spawn_timer = self.spawn_interval;

        let info = self.enemies_to_spawn.remove(0);
        let spawn_pos = self.random_edge_position(viewport);
        let id = self.next_enemy_id;
        self.next_enemy_id += 1;

        vec![Enemy::new(id, info.shape, self.current_wave, spawn_pos)]
    }

    /// Génère une vague avec une composition inspirée de D&D
    ///
    /// ## Phases de difficulté :
    /// - Phase 1 (1-20): Introduction progressive des types
    /// - Phase 2 (21-50): Composition variée, nombre croissant
    /// - Phase 3 (51-100): Ennemis lourds dominent
    /// - Phase 4 (101+): Hordes massives exponentielles
    fn generate_wave(&self, wave_num: u32) -> Vec<EnemySpawnInfo> {
        let mut enemies = Vec::new();

        // Calcul du nombre d'ennemis selon la phase
        let count = self.calculate_enemy_count(wave_num);

        for _ in 0..count {
            let shape = self.pick_enemy_shape(wave_num);
            enemies.push(EnemySpawnInfo { shape });
        }

        // Ajout des boss
        self.add_bosses(&mut enemies, wave_num);

        enemies
    }

    /// Calcule le nombre d'ennemis pour une vague
    fn calculate_enemy_count(&self, wave_num: u32) -> u32 {
        match wave_num {
            // Phase 1 : Tiers D&D (1-20)
            // Croissance linéaire douce : 6 + 2 par vague
            1..=20 => 6 + wave_num * 2,

            // Phase 2 : Paragon (21-50)
            // Croissance modérée : base 46 + 3 par vague
            21..=50 => {
                let base = 6 + 20 * 2; // 46 à la vague 20
                base + (wave_num - 20) * 3
            }

            // Phase 3 : Epic (51-100)
            // Croissance plus forte : base 136 + 4 par vague
            51..=100 => {
                let base = 46 + 30 * 3; // 136 à la vague 50
                base + (wave_num - 50) * 4
            }

            // Phase 4 : Mythique (101+)
            // Croissance exponentielle : nombre double toutes les ~25 vagues
            _ => {
                let base = 136 + 50 * 4; // 336 à la vague 100
                let waves_past_100 = (wave_num - 100) as f32;
                let exp_factor = 1.03_f32.powf(waves_past_100); // +3% par vague
                (base as f32 * exp_factor) as u32
            }
        }
    }

    /// Sélectionne un type d'ennemi selon la vague
    fn pick_enemy_shape(&self, wave_num: u32) -> EnemyShape {
        let r: f32 = rand::thread_rng().r#gen();

        match wave_num {
            // Tier 1 (1-5): Éclaireurs dominent, introduction soldats
            1..=2 => {
                if r < 0.90 {
                    EnemyShape::Triangle
                } else {
                    EnemyShape::Square
                }
            }
            3..=5 => {
                if r < 0.60 {
                    EnemyShape::Triangle
                } else {
                    EnemyShape::Square
                }
            }

            // Tier 2 (6-10): Introduction tanks
            6..=7 => {
                if r < 0.40 {
                    EnemyShape::Triangle
                } else if r < 0.75 {
                    EnemyShape::Square
                } else {
                    EnemyShape::Pentagon
                }
            }
            8..=10 => {
                if r < 0.30 {
                    EnemyShape::Triangle
                } else if r < 0.60 {
                    EnemyShape::Square
                } else {
                    EnemyShape::Pentagon
                }
            }

            // Tier 3 (11-15): Introduction destructeurs
            11..=12 => {
                if r < 0.25 {
                    EnemyShape::Triangle
                } else if r < 0.50 {
                    EnemyShape::Square
                } else if r < 0.80 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }
            13..=15 => {
                if r < 0.20 {
                    EnemyShape::Triangle
                } else if r < 0.45 {
                    EnemyShape::Square
                } else if r < 0.70 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }

            // Tier 4 (16-20): Mix équilibré
            16..=20 => {
                if r < 0.15 {
                    EnemyShape::Triangle
                } else if r < 0.35 {
                    EnemyShape::Square
                } else if r < 0.60 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }

            // Phase 2 Paragon (21-50): Moins d'éclaireurs
            21..=35 => {
                if r < 0.10 {
                    EnemyShape::Triangle
                } else if r < 0.30 {
                    EnemyShape::Square
                } else if r < 0.60 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }
            36..=50 => {
                if r < 0.05 {
                    EnemyShape::Triangle
                } else if r < 0.25 {
                    EnemyShape::Square
                } else if r < 0.55 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }

            // Phase 3 Epic (51-100): Ennemis lourds dominent
            51..=75 => {
                if r < 0.05 {
                    EnemyShape::Triangle
                } else if r < 0.20 {
                    EnemyShape::Square
                } else if r < 0.50 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }
            76..=100 => {
                if r < 0.15 {
                    EnemyShape::Square
                } else if r < 0.45 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }

            // Phase 4 Mythique (101+): Tanks et destructeurs uniquement
            _ => {
                if r < 0.40 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            }
        }
    }

    /// Ajoute les boss à la vague
    fn add_bosses(&self, enemies: &mut Vec<EnemySpawnInfo>, wave_num: u32) {
        // Boss toutes les 5 vagues à partir de la vague 5
        if wave_num >= 5 && wave_num % 5 == 0 {
            let boss_count = match wave_num {
                5..=10 => 1,
                11..=20 => 2,
                21..=50 => 3 + (wave_num - 20) / 10,
                51..=100 => 5 + (wave_num - 50) / 10,
                _ => {
                    // Mythique : croissance exponentielle des boss
                    let base = 10;
                    let waves_past_100 = (wave_num - 100) as f32;
                    let exp = 1.05_f32.powf(waves_past_100 / 5.0);
                    (base as f32 * exp) as u32
                }
            };

            for _ in 0..boss_count {
                enemies.push(EnemySpawnInfo {
                    shape: EnemyShape::Octagon,
                });
            }
        }

        // Mini-boss (hexagons supplémentaires) toutes les 10 vagues après la 20
        if wave_num > 20 && wave_num % 10 == 0 {
            let mini_boss_count = (wave_num - 20) / 10;
            for _ in 0..mini_boss_count {
                enemies.push(EnemySpawnInfo {
                    shape: EnemyShape::Hexagon,
                });
            }
        }
    }

    fn random_edge_position(&self, viewport: (f32, f32)) -> Point2D {
        let mut rng = rand::thread_rng();
        let half_w = viewport.0 / 2.0;
        let half_h = viewport.1 / 2.0;
        let margin = 30.0;

        let side: u8 = rng.gen_range(0..4);
        match side {
            0 => Point2D::new(rng.gen_range(-half_w..half_w), -(half_h + margin)), // Top
            1 => Point2D::new(rng.gen_range(-half_w..half_w), half_h + margin),    // Bottom
            2 => Point2D::new(-(half_w + margin), rng.gen_range(-half_h..half_h)), // Left
            _ => Point2D::new(half_w + margin, rng.gen_range(-half_h..half_h)),    // Right
        }
    }
}
