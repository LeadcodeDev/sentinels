pub mod elemental;
pub mod enemy;
pub mod player;
pub mod skill_tree;
pub mod tower;
pub mod wave;

use crate::data::SaveData;
use elemental::TowerElement;
use enemy::Enemy;
use player::Player;
use tower::Tower;
use wave::WaveManager;

#[derive(Clone, Copy, PartialEq)]
pub enum GamePhase {
    Preparing,
    Active,
    GameOver,
}

#[derive(Clone)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn distance_to(&self, other: &Point2D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectileSource {
    Player,
    Tower(usize),
    Enemy(usize),
}

#[derive(Clone)]
pub struct Projectile {
    pub origin: Point2D,
    pub target_pos: Point2D,
    pub current_pos: Point2D,
    pub speed: f32,
    pub damage: f32,
    pub element: TowerElement,
    pub source: ProjectileSource,
    pub is_aoe: bool,
    pub aoe_radius: f32,
    pub lifetime: f32,
    pub target_enemy_id: Option<usize>,
}

#[derive(Clone)]
pub struct Economy {
    pub gold: u32,
    pub score: u32,
    pub wave_number: u32,
}

pub struct GameState {
    pub player: Player,
    pub towers: Vec<Tower>,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub wave_manager: WaveManager,
    pub economy: Economy,
    pub phase: GamePhase,
    pub viewport_size: (f32, f32),
    pub placement_mode: Option<TowerElement>,
    pub selected_tower: Option<usize>,
    pub elapsed: f32,
}

impl GameState {
    pub fn new(save_data: &SaveData) -> Self {
        let bonus_gold = save_data.get_upgrade_level("bonus_gold") as u32 * 50;

        Self {
            player: Player::new(),
            towers: Vec::new(),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            wave_manager: WaveManager::new(),
            economy: Economy {
                gold: 500 + bonus_gold,
                score: 0,
                wave_number: 0,
            },
            phase: GamePhase::Preparing,
            viewport_size: (1200.0, 800.0),
            placement_mode: None,
            selected_tower: None,
            elapsed: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        self.elapsed += dt;

        // 1. Wave manager update
        if self.phase == GamePhase::Active {
            let new_enemies = self.wave_manager.tick(dt, self.viewport_size);
            self.enemies.extend(new_enemies);

            // Check wave completion
            if self.wave_manager.is_wave_complete() && self.enemies.is_empty() {
                self.phase = GamePhase::Preparing;
                self.economy.gold += 100 + self.wave_manager.current_wave * 50;
                self.wave_manager.start_between_waves();
            }
        } else if self.phase == GamePhase::Preparing {
            if self.wave_manager.tick_between_waves(dt) {
                self.wave_manager.start_next_wave();
                self.phase = GamePhase::Active;
                self.economy.wave_number = self.wave_manager.current_wave;
            }
        }

        // 2. Enemy movement
        let center = Point2D::zero();
        for enemy in &mut self.enemies {
            enemy.tick(dt, &center);
        }

        // 3. Enemy attacks - enemies in range fire projectiles at player
        let player_pos = self.player.position.clone();
        for enemy in &mut self.enemies {
            if let Some(proj) = enemy.try_attack(&player_pos, dt) {
                self.projectiles.push(proj);
            }
        }

        // 4. Player auto-attack
        self.player.attack_cooldown -= dt;
        if self.player.attack_cooldown <= 0.0 {
            if let Some(target_idx) = find_nearest_in_range(
                &self.player.position,
                self.player.attack_range,
                &self.enemies,
            ) {
                self.player.attack_cooldown = 1.0 / self.player.attack_speed;
                let target_pos = self.enemies[target_idx].position.clone();
                let target_id = self.enemies[target_idx].id;
                self.projectiles.push(Projectile {
                    origin: self.player.position.clone(),
                    target_pos: target_pos.clone(),
                    current_pos: self.player.position.clone(),
                    speed: 400.0,
                    damage: self.player.attack_damage,
                    element: self.player.element,
                    source: ProjectileSource::Player,
                    is_aoe: false,
                    aoe_radius: 0.0,
                    lifetime: 3.0,
                    target_enemy_id: Some(target_id),
                });
            }
        }

        // 5. Tower auto-attacks
        for i in 0..self.towers.len() {
            self.towers[i].attack_cooldown -= dt;
            if self.towers[i].attack_cooldown <= 0.0 {
                let tower_pos = self.towers[i].position.clone();
                let tower_range = self.towers[i].attack_range;
                if let Some(target_idx) =
                    find_nearest_in_range(&tower_pos, tower_range, &self.enemies)
                {
                    let tower = &mut self.towers[i];
                    tower.attack_cooldown = 1.0 / tower.attack_speed;
                    let target_pos = self.enemies[target_idx].position.clone();
                    let target_id = self.enemies[target_idx].id;
                    self.projectiles.push(Projectile {
                        origin: tower_pos.clone(),
                        target_pos: target_pos.clone(),
                        current_pos: tower_pos,
                        speed: 350.0,
                        damage: tower.attack_damage,
                        element: tower.element,
                        source: ProjectileSource::Tower(i),
                        is_aoe: tower.is_aoe,
                        aoe_radius: tower.aoe_radius,
                        lifetime: 3.0,
                        target_enemy_id: Some(target_id),
                    });
                }
            }
        }

        // 6. Projectile movement + collision
        let mut player_damage: f32 = 0.0;
        let mut enemy_hits: Vec<(usize, f32, TowerElement, bool, f32, Point2D)> = Vec::new();

        for proj in &mut self.projectiles {
            // Update target position for homing projectiles
            if let Some(target_id) = proj.target_enemy_id {
                if let Some(enemy) = self.enemies.iter().find(|e| e.id == target_id) {
                    proj.target_pos = enemy.position.clone();
                }
            }

            // Move projectile toward target
            let dx = proj.target_pos.x - proj.current_pos.x;
            let dy = proj.target_pos.y - proj.current_pos.y;
            let dist_to_target = (dx * dx + dy * dy).sqrt();

            if dist_to_target > 1.0 {
                let move_dist = proj.speed * dt;
                proj.current_pos.x += dx / dist_to_target * move_dist;
                proj.current_pos.y += dy / dist_to_target * move_dist;
            }

            // Check collision based on source type
            let mut hit = false;
            match proj.source {
                ProjectileSource::Player | ProjectileSource::Tower(_) => {
                    // Check collision with any enemy near the projectile's current position
                    for (idx, enemy) in self.enemies.iter().enumerate() {
                        if enemy.position.distance_to(&proj.current_pos) < enemy.radius + 10.0 {
                            enemy_hits.push((
                                idx,
                                proj.damage,
                                proj.element,
                                proj.is_aoe,
                                proj.aoe_radius,
                                proj.current_pos.clone(),
                            ));
                            hit = true;
                            break;
                        }
                    }
                }
                ProjectileSource::Enemy(_) => {
                    // Check collision with player
                    let player_pos = &self.player.position;
                    if proj.current_pos.distance_to(player_pos) < self.player.radius + 5.0 {
                        player_damage += proj.damage;
                        hit = true;
                    }
                }
            }

            if hit || dist_to_target < 5.0 {
                proj.lifetime = 0.0;
            }

            proj.lifetime -= dt;
        }

        // 7. Apply damage
        self.player.hp -= player_damage;

        for (idx, damage, element, is_aoe, aoe_radius, pos) in enemy_hits {
            if idx < self.enemies.len() {
                self.enemies[idx].take_damage(damage, element);

                if is_aoe && aoe_radius > 0.0 {
                    for enemy in &mut self.enemies {
                        if enemy.position.distance_to(&pos) < aoe_radius {
                            enemy.take_damage(damage * 0.5, element);
                        }
                    }
                }
            }
        }

        // 8. Remove dead enemies + award gold
        self.enemies.retain(|e| {
            if e.is_dead() {
                self.economy.gold += e.gold_value;
                self.economy.score += e.gold_value;
                false
            } else {
                true
            }
        });

        // 9. Remove expired projectiles
        self.projectiles.retain(|p| p.lifetime > 0.0);

        // 10. Remove enemies that reached the player
        let player_radius = self.player.radius;
        self.enemies.retain(|e| {
            let dist = e.position.distance_to(&center);
            if dist < player_radius + e.radius {
                self.player.hp -= e.damage;
                false
            } else {
                true
            }
        });

        // 11. Game over check
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.phase = GamePhase::GameOver;
        }
    }

    pub fn try_place_tower(&mut self, element: TowerElement, x: f32, y: f32) {
        use crate::data::tower_presets::get_preset;

        let preset = get_preset(element);
        if self.economy.gold < preset.base_cost {
            return;
        }

        let pos = Point2D::new(x, y);

        // Check not too close to another tower
        for tower in &self.towers {
            if tower.position.distance_to(&pos) < 30.0 {
                return;
            }
        }

        // Check not on top of player
        if pos.distance_to(&Point2D::zero()) < self.player.radius + 20.0 {
            return;
        }

        self.economy.gold -= preset.base_cost;
        let id = self.towers.len();
        self.towers.push(Tower::from_preset(id, element, pos));
    }

    pub fn try_select_at(&mut self, x: f32, y: f32) {
        let click_pos = Point2D::new(x, y);

        // Check if clicked on a tower
        for (i, tower) in self.towers.iter().enumerate() {
            if tower.position.distance_to(&click_pos) < tower.radius + 10.0 {
                if self.selected_tower == Some(i) {
                    // Re-click on same tower -> deselect
                    self.selected_tower = None;
                } else {
                    // Select new tower (or switch selection)
                    self.selected_tower = Some(i);
                }
                return;
            }
        }

        self.selected_tower = None;
    }

    pub fn start_wave(&mut self) {
        if self.phase == GamePhase::Preparing {
            self.wave_manager.start_next_wave();
            self.phase = GamePhase::Active;
            self.economy.wave_number = self.wave_manager.current_wave;
        }
    }
}

fn find_nearest_in_range(pos: &Point2D, range: f32, enemies: &[Enemy]) -> Option<usize> {
    enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| pos.distance_to(&e.position) <= range)
        .min_by(|(_, a), (_, b)| {
            pos.distance_to(&a.position)
                .partial_cmp(&pos.distance_to(&b.position))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
}
