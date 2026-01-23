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
pub struct Shield {
    pub active: bool,
    pub hp: f32,
    pub max_hp: f32,
    pub radius: f32,
    pub regen_timer: f32,
    pub regen_delay: f32,
}

impl Shield {
    pub fn new(level: u32) -> Self {
        if level == 0 {
            return Self {
                active: false,
                hp: 0.0,
                max_hp: 0.0,
                radius: 0.0,
                regen_timer: 0.0,
                regen_delay: 15.0,
            };
        }
        let max_hp = 50.0 * level as f32;
        Self {
            active: true,
            hp: max_hp,
            max_hp,
            radius: 80.0,
            regen_timer: 0.0,
            regen_delay: 15.0,
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.max_hp > 0.0
    }
}

#[derive(Clone)]
pub struct AoeSplash {
    pub position: Point2D,
    pub radius: f32,
    pub color: (f32, f32, f32), // h, s, l
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Clone)]
pub struct Economy {
    pub gold: u32,
    pub score: u32,
    pub wave_number: u32,
    pub pepites: u32,
}

pub struct GameState {
    pub player: Player,
    pub shield: Shield,
    pub towers: Vec<Tower>,
    pub max_towers: u32,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub aoe_splashes: Vec<AoeSplash>,
    pub wave_manager: WaveManager,
    pub economy: Economy,
    pub phase: GamePhase,
    pub viewport_size: (f32, f32),
    pub placement_mode: Option<TowerElement>,
    pub selected_tower: Option<usize>,
    pub elapsed: f32,
    pub save_data: SaveData,
}

impl GameState {
    pub fn new(save_data: &SaveData) -> Self {
        let bonus_gold = save_data.get_upgrade_level("bonus_gold") as u32 * 50;
        let shield_level = save_data.get_upgrade_level("shield");
        let tower_slots_level = save_data.get_upgrade_level("tower_slots");

        Self {
            player: Player::new(),
            shield: Shield::new(shield_level),
            towers: Vec::new(),
            max_towers: 5 + tower_slots_level,
            enemies: Vec::new(),
            projectiles: Vec::new(),
            aoe_splashes: Vec::new(),
            wave_manager: WaveManager::new(),
            economy: Economy {
                gold: 500 + bonus_gold,
                score: 0,
                wave_number: 0,
                pepites: 0,
            },
            phase: GamePhase::Preparing,
            viewport_size: (1200.0, 800.0),
            placement_mode: None,
            selected_tower: None,
            elapsed: 0.0,
            save_data: save_data.clone(),
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

        // 2. Shield regen tick
        if self.shield.is_unlocked() && !self.shield.active {
            self.shield.regen_timer -= dt;
            if self.shield.regen_timer <= 0.0 {
                self.shield.active = true;
                self.shield.hp = self.shield.max_hp;
            }
        }

        // 3. Enemy movement (blocked by shield)
        let center = Point2D::zero();
        let shield_stop = if self.shield.active {
            Some(self.shield.radius)
        } else {
            None
        };
        for enemy in &mut self.enemies {
            enemy.tick(dt, &center, shield_stop);
        }

        // 3. Enemy attacks - enemies target shield if active, otherwise player
        let player_pos = self.player.position.clone();
        let attack_target_radius = if self.shield.active {
            self.shield.radius
        } else {
            0.0
        };
        for enemy in &mut self.enemies {
            if let Some(proj) = enemy.try_attack(&player_pos, attack_target_radius, dt) {
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
                    let player_pos = &self.player.position;
                    // Check collision with shield first
                    if self.shield.active {
                        let dist_to_center = proj.current_pos.distance_to(player_pos);
                        if dist_to_center < self.shield.radius + 5.0 {
                            self.shield.hp -= proj.damage;
                            if self.shield.hp <= 0.0 {
                                self.shield.hp = 0.0;
                                self.shield.active = false;
                                self.shield.regen_timer = self.shield.regen_delay;
                            }
                            hit = true;
                        }
                    } else {
                        // No shield: check collision with player
                        if proj.current_pos.distance_to(player_pos) < self.player.radius + 5.0 {
                            player_damage += proj.damage;
                            hit = true;
                        }
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
                    let color = element.color();
                    self.aoe_splashes.push(AoeSplash {
                        position: pos.clone(),
                        radius: aoe_radius,
                        color: (color.h, color.s, color.l),
                        lifetime: 0.4,
                        max_lifetime: 0.4,
                    });
                    for enemy in &mut self.enemies {
                        if enemy.position.distance_to(&pos) < aoe_radius {
                            enemy.take_damage(damage * 0.5, element);
                        }
                    }
                }
            }
        }

        // 8. Remove dead enemies + award gold + random pepite drops
        self.enemies.retain(|e| {
            if e.is_dead() {
                self.economy.gold += e.gold_value;
                self.economy.score += e.gold_value;
                // Pepite drops: bosses always drop 3-5 + bonus tower slot, others 10% chance for 1
                if e.is_boss {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    self.economy.pepites += rng.r#gen_range(3..=5);
                    self.max_towers += 1;
                } else {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    if rng.r#gen_range(0..100) < 10 {
                        self.economy.pepites += 1;
                    }
                }
                false
            } else {
                true
            }
        });

        // 9. Remove expired projectiles and tick AoE splashes
        self.projectiles.retain(|p| p.lifetime > 0.0);
        for splash in &mut self.aoe_splashes {
            splash.lifetime -= dt;
        }
        self.aoe_splashes.retain(|s| s.lifetime > 0.0);

        // 10. Remove enemies that reached the player (no shield)
        let player_radius = self.player.radius;
        if !self.shield.active {
            self.enemies.retain(|e| {
                let dist = e.position.distance_to(&center);
                if dist < player_radius + e.radius {
                    self.player.hp -= e.damage;
                    false
                } else {
                    true
                }
            });
        }

        // 11. Game over check
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.phase = GamePhase::GameOver;
            // Transfer pepites to persistent save
            self.save_data.pepites += self.economy.pepites;
            if self.economy.score > self.save_data.best_score {
                self.save_data.best_score = self.economy.score;
            }
            if self.economy.wave_number > self.save_data.max_wave {
                self.save_data.max_wave = self.economy.wave_number;
            }
            self.save_data.save();
        }
    }

    pub fn try_place_tower(&mut self, element: TowerElement, x: f32, y: f32) {
        use crate::data::tower_presets::get_preset;

        if self.towers.len() >= self.max_towers as usize {
            return;
        }

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

    pub fn upgrade_tower(
        &mut self,
        tower_idx: usize,
        upgrade_type: tower::TowerUpgradeType,
    ) -> bool {
        if tower_idx >= self.towers.len() {
            return false;
        }
        let cost = {
            let tower = &self.towers[tower_idx];
            let upgrade = tower
                .upgrades
                .iter()
                .find(|u| u.upgrade_type == upgrade_type);
            match upgrade {
                Some(u) if u.level < u.max_level => u.cost(),
                _ => return false,
            }
        };
        if self.economy.gold < cost {
            return false;
        }
        self.economy.gold -= cost;
        self.towers[tower_idx].apply_upgrade(upgrade_type);
        true
    }

    pub fn sell_tower(&mut self, tower_idx: usize) {
        if tower_idx >= self.towers.len() {
            return;
        }
        let value = self.towers[tower_idx].sell_value();
        self.economy.gold += value;
        self.towers.remove(tower_idx);
        self.selected_tower = None;
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
