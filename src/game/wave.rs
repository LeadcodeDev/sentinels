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
            spawn_interval: 0.8,
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

    fn generate_wave(&self, wave_num: u32) -> Vec<EnemySpawnInfo> {
        let mut enemies = Vec::new();
        let count = 5 + wave_num * 2;

        for _ in 0..count {
            let shape = if wave_num <= 2 {
                EnemyShape::Triangle
            } else if wave_num <= 5 {
                if rand::thread_rng().r#gen::<f32>() < 0.6 {
                    EnemyShape::Triangle
                } else {
                    EnemyShape::Square
                }
            } else if wave_num <= 10 {
                let r: f32 = rand::thread_rng().r#gen();
                if r < 0.3 {
                    EnemyShape::Triangle
                } else if r < 0.6 {
                    EnemyShape::Square
                } else if r < 0.85 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            } else {
                let r: f32 = rand::thread_rng().r#gen();
                if r < 0.2 {
                    EnemyShape::Triangle
                } else if r < 0.4 {
                    EnemyShape::Square
                } else if r < 0.6 {
                    EnemyShape::Pentagon
                } else {
                    EnemyShape::Hexagon
                }
            };

            enemies.push(EnemySpawnInfo { shape });
        }

        // Boss every 10 waves
        if wave_num % 10 == 0 {
            enemies.push(EnemySpawnInfo {
                shape: EnemyShape::Octagon,
            });
        }

        enemies
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
