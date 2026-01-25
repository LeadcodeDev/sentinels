use super::elemental::TowerElement;
use super::{Point2D, Projectile, ProjectileSource};
use crate::data::tower_defs::{EffectTarget, ResolvedAction, ResolvedDamage};

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
pub struct BurnState {
    pub dps: f32,
    pub remaining: f32,
}

// ============================================================================
// ELEMENTAL STATES (Brûlé, Trempé, Sismique)
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElementalStateKind {
    /// Brûlé: DoT (dégâts sur la durée)
    Burned,
    /// Trempé: Ralentissement
    Soaked,
    /// Sismique: Stun court à chaque dégât subi
    Seismic,
    /// Froid: Si la cible est Trempée, elle perd Trempé et Froid et devient Gelée
    Cold,
    /// Gelé: Stun complet pendant X secondes
    Frozen,
}

#[derive(Clone)]
pub struct ElementalState {
    pub kind: ElementalStateKind,
    pub remaining: f32,
    pub strength: f32, // dps pour Burned, ratio slow pour Soaked, stun duration pour Seismic
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
    pub elemental_states: Vec<ElementalState>,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub stun_duration: f32,
    pub is_boss: bool,
    pub burn: Option<BurnState>,
}

impl Enemy {
    /// Calcule les multiplicateurs de stats basés sur les Tiers D&D
    /// Tier 1 (vagues 1-5):   Base stats - Introduction
    /// Tier 2 (vagues 6-10):  HP ×1.25, DMG ×1.15 - Montée en puissance
    /// Tier 3 (vagues 11-15): HP ×1.50, DMG ×1.30 - Challenge sérieux
    /// Tier 4 (vagues 16-20): HP ×2.00, DMG ×1.50 - Difficile
    /// Tier 5 (vagues 21+):   HP ×2.50, DMG ×1.75 - Epic/Endgame
    fn get_tier_scaling(wave_number: u32) -> (f32, f32, f32) {
        // Retourne (hp_multiplier, damage_multiplier, gold_multiplier)
        match wave_number {
            1..=5 => (1.0, 1.0, 1.0),
            6..=10 => {
                // Interpolation linéaire dans le tier pour une transition douce
                let progress = (wave_number - 5) as f32 / 5.0;
                (
                    1.0 + 0.25 * progress, // 1.0 -> 1.25
                    1.0 + 0.15 * progress, // 1.0 -> 1.15
                    1.0 + 0.10 * progress, // 1.0 -> 1.10 (bonus or)
                )
            }
            11..=15 => {
                let progress = (wave_number - 10) as f32 / 5.0;
                (
                    1.25 + 0.25 * progress, // 1.25 -> 1.50
                    1.15 + 0.15 * progress, // 1.15 -> 1.30
                    1.10 + 0.10 * progress, // 1.10 -> 1.20
                )
            }
            16..=20 => {
                let progress = (wave_number - 15) as f32 / 5.0;
                (
                    1.50 + 0.50 * progress, // 1.50 -> 2.00
                    1.30 + 0.20 * progress, // 1.30 -> 1.50
                    1.20 + 0.15 * progress, // 1.20 -> 1.35
                )
            }
            _ => {
                // Tier 5+: scaling continu mais plus lent
                let extra_waves = (wave_number - 20) as f32;
                (
                    2.00 + 0.10 * extra_waves, // +10% HP par vague au-delà de 20
                    1.50 + 0.05 * extra_waves, // +5% DMG par vague
                    1.35 + 0.05 * extra_waves, // +5% or par vague
                )
            }
        }
    }

    pub fn new(id: usize, shape: EnemyShape, wave_number: u32, spawn_pos: Point2D) -> Self {
        use crate::data::enemy_types::get_preset;

        let preset = get_preset(shape);
        let (hp_scale, damage_scale, gold_scale) = Self::get_tier_scaling(wave_number);

        Self {
            id,
            position: spawn_pos,
            shape,
            hp: preset.base_hp * hp_scale,
            max_hp: preset.base_hp * hp_scale,
            speed: preset.base_speed,
            damage: preset.base_damage * damage_scale,
            attack_range: preset.attack_range,
            attack_speed: preset.attack_speed,
            attack_cooldown: 0.0,
            gold_value: (preset.gold_value as f32 * gold_scale) as u32,
            radius: preset.radius,
            applied_elements: Vec::new(),
            elemental_states: Vec::new(),
            slow_factor: 1.0,
            slow_duration: 0.0,
            stun_duration: 0.0,
            is_boss: shape == EnemyShape::Octagon,
            burn: None,
        }
    }

    pub fn tick(&mut self, dt: f32, center: &Point2D, shield_radius: Option<f32>) {
        // Tick stun
        if self.stun_duration > 0.0 {
            self.stun_duration -= dt;
        }

        let is_stunned = self.stun_duration > 0.0;

        // Movement (blocked by stun)
        if !is_stunned {
            let stop_distance = match shield_radius {
                Some(r) => r,
                None => self.attack_range,
            };

            let dist_to_center = self.position.distance_to(center);
            if dist_to_center > stop_distance {
                let dx = center.x - self.position.x;
                let dy = center.y - self.position.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > 1.0 {
                    let effective_speed = self.speed * self.slow_factor * dt;
                    self.position.x += dx / dist * effective_speed;
                    self.position.y += dy / dist * effective_speed;
                }
            }
        }

        // Tick slow duration
        if self.slow_duration > 0.0 {
            self.slow_duration -= dt;
            if self.slow_duration <= 0.0 {
                self.slow_factor = 1.0;
            }
        }

        // Tick burn damage
        if let Some(ref mut burn) = self.burn {
            self.hp -= burn.dps * dt;
            burn.remaining -= dt;
            if burn.remaining <= 0.0 {
                self.burn = None;
            }
        }

        // Tick applied elements
        self.applied_elements.retain_mut(|ae| {
            ae.duration -= dt;
            ae.duration > 0.0
        });
    }

    pub fn try_attack(
        &mut self,
        target_pos: &Point2D,
        target_radius: f32,
        dt: f32,
    ) -> Option<Projectile> {
        if self.stun_duration > 0.0 {
            return None;
        }

        let dist = self.position.distance_to(target_pos);
        if dist > target_radius + self.attack_range {
            return None;
        }

        self.attack_cooldown -= dt;
        if self.attack_cooldown <= 0.0 {
            self.attack_cooldown = 1.0 / self.attack_speed;
            Some(Projectile {
                origin: self.position.clone(),
                target_pos: target_pos.clone(),
                current_pos: self.position.clone(),
                speed: 200.0,
                element: TowerElement::Neutral,
                source: ProjectileSource::Enemy(self.id),
                actions: vec![ResolvedAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: ResolvedDamage::Fixed(self.damage),
                }],
                lifetime: 3.0,
                target_enemy_id: None,
                fade_timer: None,
                size: 3.0,
            })
        } else {
            None
        }
    }

    pub fn take_damage(&mut self, damage: f32, element: TowerElement) {
        // Surpression: +20% dégâts si Brûlé + Trempé
        let damage = damage * self.surpression_multiplier();

        self.hp -= damage;

        // Sismique: stun court à chaque dégât
        if let Some(seismic) = self
            .elemental_states
            .iter()
            .find(|s| s.kind == ElementalStateKind::Seismic)
        {
            self.apply_stun(seismic.strength);
        }

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

    pub fn apply_slow(&mut self, ratio: f32, duration: f32) {
        // Lower ratio = slower; keep the strongest slow
        if ratio < self.slow_factor || duration > self.slow_duration {
            self.slow_factor = self.slow_factor.min(ratio);
            self.slow_duration = self.slow_duration.max(duration);
        }
    }

    pub fn apply_stun(&mut self, duration: f32) {
        self.stun_duration = self.stun_duration.max(duration);
    }

    pub fn apply_burn(&mut self, dps: f32, duration: f32) {
        // Refresh burn with the strongest values
        if let Some(ref mut burn) = self.burn {
            burn.dps = burn.dps.max(dps);
            burn.remaining = burn.remaining.max(duration);
        } else {
            self.burn = Some(BurnState {
                dps,
                remaining: duration,
            });
        }
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    // ========================================================================
    // ELEMENTAL STATES (Brûlé, Trempé, Sismique)
    // ========================================================================

    /// Applique l'état Brûlé (DoT)
    pub fn apply_burned(&mut self, dps: f32, duration: f32) {
        if let Some(state) = self
            .elemental_states
            .iter_mut()
            .find(|s| s.kind == ElementalStateKind::Burned)
        {
            // Refresh with strongest values
            state.strength = state.strength.max(dps);
            state.remaining = state.remaining.max(duration);
        } else {
            self.elemental_states.push(ElementalState {
                kind: ElementalStateKind::Burned,
                remaining: duration,
                strength: dps,
            });
        }
    }

    /// Applique l'état Trempé (Slow)
    pub fn apply_soaked(&mut self, slow_ratio: f32, duration: f32) {
        if let Some(state) = self
            .elemental_states
            .iter_mut()
            .find(|s| s.kind == ElementalStateKind::Soaked)
        {
            // Refresh with strongest values (lower ratio = stronger slow)
            state.strength = state.strength.min(slow_ratio);
            state.remaining = state.remaining.max(duration);
        } else {
            self.elemental_states.push(ElementalState {
                kind: ElementalStateKind::Soaked,
                remaining: duration,
                strength: slow_ratio,
            });
        }
    }

    /// Applique l'état Sismique (stun on hit)
    pub fn apply_seismic(&mut self, stun_duration: f32, state_duration: f32) {
        if let Some(state) = self
            .elemental_states
            .iter_mut()
            .find(|s| s.kind == ElementalStateKind::Seismic)
        {
            // Refresh with strongest values
            state.strength = state.strength.max(stun_duration);
            state.remaining = state.remaining.max(state_duration);
        } else {
            self.elemental_states.push(ElementalState {
                kind: ElementalStateKind::Seismic,
                remaining: state_duration,
                strength: stun_duration,
            });
        }
    }

    /// Applique l'état Froid - si Trempé, déclenche Gelé
    /// Retourne true si la cible a été gelée
    pub fn apply_cold(&mut self, freeze_duration: f32, cold_duration: f32) -> bool {
        // Si la cible est Trempée, elle devient Gelée
        if self.has_state(ElementalStateKind::Soaked) {
            // Supprime Trempé et Froid
            self.elemental_states.retain(|s| {
                s.kind != ElementalStateKind::Soaked && s.kind != ElementalStateKind::Cold
            });
            // Applique Gelé
            self.apply_frozen(freeze_duration);
            return true;
        }

        // Sinon, applique simplement Froid
        if let Some(state) = self
            .elemental_states
            .iter_mut()
            .find(|s| s.kind == ElementalStateKind::Cold)
        {
            // Refresh with strongest values
            state.strength = state.strength.max(freeze_duration);
            state.remaining = state.remaining.max(cold_duration);
        } else {
            self.elemental_states.push(ElementalState {
                kind: ElementalStateKind::Cold,
                remaining: cold_duration,
                strength: freeze_duration, // Durée du freeze si combiné avec Trempé
            });
        }
        false
    }

    /// Applique l'état Gelé (stun complet)
    pub fn apply_frozen(&mut self, duration: f32) {
        if let Some(state) = self
            .elemental_states
            .iter_mut()
            .find(|s| s.kind == ElementalStateKind::Frozen)
        {
            // Refresh duration
            state.remaining = state.remaining.max(duration);
        } else {
            self.elemental_states.push(ElementalState {
                kind: ElementalStateKind::Frozen,
                remaining: duration,
                strength: duration,
            });
        }
        // Applique également le stun
        self.apply_stun(duration);
    }

    /// Vérifie si l'ennemi a un état spécifique
    pub fn has_state(&self, kind: ElementalStateKind) -> bool {
        self.elemental_states.iter().any(|s| s.kind == kind)
    }

    /// Compte le nombre d'états élémentaires actifs
    pub fn count_states(&self) -> usize {
        self.elemental_states.len()
    }

    /// Supprime tous les états élémentaires
    pub fn clear_states(&mut self) {
        self.elemental_states.clear();
    }

    /// Tick des états élémentaires (appelé chaque frame)
    pub fn tick_elemental_states(&mut self, dt: f32) {
        let mut damage_from_burn = 0.0;

        for state in &mut self.elemental_states {
            state.remaining -= dt;

            match state.kind {
                ElementalStateKind::Burned => {
                    // DoT: applique dégâts basés sur dps
                    damage_from_burn += state.strength * dt;
                }
                ElementalStateKind::Soaked => {
                    // Le slow est appliqué dans effective_speed()
                }
                ElementalStateKind::Seismic => {
                    // Le stun est appliqué dans take_damage()
                }
                ElementalStateKind::Cold => {
                    // Froid attend d'être combiné avec Trempé
                }
                ElementalStateKind::Frozen => {
                    // Gelé: maintient le stun actif
                    self.stun_duration = self.stun_duration.max(state.remaining);
                }
            }
        }

        // Applique les dégâts de brûlure (sans déclencher Sismique)
        if damage_from_burn > 0.0 {
            self.hp -= damage_from_burn;
        }

        // Supprime les états expirés
        self.elemental_states.retain(|s| s.remaining > 0.0);
    }

    /// Retourne la vitesse effective (avec slow de Trempé)
    pub fn effective_speed(&self) -> f32 {
        let mut speed = self.speed;

        // Slow from Soaked state
        if let Some(soaked) = self
            .elemental_states
            .iter()
            .find(|s| s.kind == ElementalStateKind::Soaked)
        {
            speed *= soaked.strength;
        }

        // Legacy slow system
        if self.slow_duration > 0.0 {
            speed *= self.slow_factor;
        }

        speed
    }

    /// Vérifie la Surpression (Brûlé + Trempé) - retourne le multiplicateur de dégâts
    pub fn surpression_multiplier(&self) -> f32 {
        if self.has_state(ElementalStateKind::Burned) && self.has_state(ElementalStateKind::Soaked)
        {
            1.20 // +20% dégâts
        } else {
            1.0
        }
    }
}
