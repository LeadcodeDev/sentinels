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
    Burned = 0,
    /// Trempé: Ralentissement
    Soaked = 1,
    /// Sismique: Stun court à chaque dégât subi
    Seismic = 2,
    /// Froid: Si la cible est Trempée, elle perd Trempé et Froid et devient Gelée
    Cold = 3,
    /// Gelé: Stun complet pendant X secondes
    Frozen = 4,
}

impl ElementalStateKind {
    /// Retourne l'index pour le tableau de cooldowns
    pub fn index(&self) -> usize {
        *self as usize
    }
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
    /// Immunité post-Gel : temps restant d'immunité à Trempé/Froid après un Gel
    pub freeze_immunity: f32,
    /// Cooldowns par type d'état élémentaire (pour éviter le spam d'auras)
    pub state_cooldowns: [f32; 5], // Un cooldown par ElementalStateKind
}

impl Enemy {
    /// Système de scaling inspiré de D&D avec 4 phases :
    ///
    /// ## Phase 1 : Tiers D&D (vagues 1-20)
    /// Progression par paliers de 5 vagues, comme les tiers de jeu D&D
    /// - Tier 1 (1-5):   Niveau 1-4   - Apprentis aventuriers
    /// - Tier 2 (6-10):  Niveau 5-10  - Héros locaux
    /// - Tier 3 (11-15): Niveau 11-16 - Héros du royaume
    /// - Tier 4 (16-20): Niveau 17-20 - Maîtres du monde
    ///
    /// ## Phase 2 : Paragon (vagues 21-50)
    /// Croissance linéaire modérée, consolidation
    ///
    /// ## Phase 3 : Epic (vagues 51-100)
    /// Croissance linéaire plus forte, défis majeurs
    ///
    /// ## Phase 4 : Mythique (vagues 101+)
    /// Croissance EXPONENTIELLE - mode survie ultime
    fn get_tier_scaling(wave_number: u32) -> (f32, f32, f32) {
        // Retourne (hp_multiplier, damage_multiplier, gold_multiplier)
        match wave_number {
            // ================================================================
            // PHASE 1 : TIERS D&D (vagues 1-20)
            // ================================================================

            // Tier 1 (CR 1/4 - 1) : Introduction, apprentissage
            1..=5 => {
                let progress = (wave_number - 1) as f32 / 4.0;
                (
                    1.0 + 0.15 * progress, // 1.0 -> 1.15
                    1.0 + 0.10 * progress, // 1.0 -> 1.10
                    1.0,
                )
            }

            // Tier 2 (CR 2-4) : Montée en puissance
            6..=10 => {
                let progress = (wave_number - 5) as f32 / 5.0;
                (
                    1.15 + 0.35 * progress, // 1.15 -> 1.50
                    1.10 + 0.20 * progress, // 1.10 -> 1.30
                    1.0 + 0.15 * progress,  // 1.0 -> 1.15
                )
            }

            // Tier 3 (CR 5-10) : Challenge sérieux
            11..=15 => {
                let progress = (wave_number - 10) as f32 / 5.0;
                (
                    1.50 + 0.50 * progress, // 1.50 -> 2.00
                    1.30 + 0.25 * progress, // 1.30 -> 1.55
                    1.15 + 0.15 * progress, // 1.15 -> 1.30
                )
            }

            // Tier 4 (CR 11-16) : Héros accomplis
            16..=20 => {
                let progress = (wave_number - 15) as f32 / 5.0;
                (
                    2.00 + 0.75 * progress, // 2.00 -> 2.75
                    1.55 + 0.30 * progress, // 1.55 -> 1.85
                    1.30 + 0.20 * progress, // 1.30 -> 1.50
                )
            }

            // ================================================================
            // PHASE 2 : PARAGON (vagues 21-50)
            // Croissance linéaire modérée (+3% HP, +1.5% DMG par vague)
            // ================================================================
            21..=50 => {
                let waves_past_20 = (wave_number - 20) as f32;
                (
                    2.75 + 0.03 * waves_past_20 * (waves_past_20 + 1.0) / 2.0 * 0.1, // Légère accélération
                    1.85 + 0.015 * waves_past_20 * (waves_past_20 + 1.0) / 2.0 * 0.05,
                    1.50 + 0.02 * waves_past_20,
                )
            }

            // ================================================================
            // PHASE 3 : EPIC (vagues 51-100)
            // Croissance linéaire plus forte (+5% HP, +2.5% DMG par vague)
            // ================================================================
            51..=100 => {
                // Valeurs à la vague 50
                let base_hp = 2.75 + 0.03 * 30.0 * 31.0 / 2.0 * 0.1; // ~4.14
                let base_dmg = 1.85 + 0.015 * 30.0 * 31.0 / 2.0 * 0.05; // ~2.20
                let base_gold = 1.50 + 0.02 * 30.0; // 2.10

                let waves_past_50 = (wave_number - 50) as f32;
                (
                    base_hp + 0.05 * waves_past_50 + 0.001 * waves_past_50 * waves_past_50,
                    base_dmg + 0.025 * waves_past_50 + 0.0005 * waves_past_50 * waves_past_50,
                    base_gold + 0.03 * waves_past_50,
                )
            }

            // ================================================================
            // PHASE 4 : MYTHIQUE (vagues 101+)
            // Croissance EXPONENTIELLE - survie impossible à long terme
            // HP et DMG doublent environ toutes les 20 vagues
            // ================================================================
            _ => {
                // Valeurs à la vague 100
                let base_hp = 4.14 + 0.05 * 50.0 + 0.001 * 50.0 * 50.0; // ~9.14
                let base_dmg = 2.20 + 0.025 * 50.0 + 0.0005 * 50.0 * 50.0; // ~4.70
                let base_gold = 2.10 + 0.03 * 50.0; // 3.60

                let waves_past_100 = (wave_number - 100) as f32;

                // Croissance exponentielle : multiplié par 1.035^n (double tous les ~20 vagues)
                let exp_factor = 1.035_f32.powf(waves_past_100);

                (
                    base_hp * exp_factor,
                    base_dmg * exp_factor,
                    base_gold * (1.0 + 0.05 * waves_past_100).min(10.0), // Cap à x10 or
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
            freeze_immunity: 0.0,
            state_cooldowns: [0.0; 5],
        }
    }

    pub fn tick(&mut self, dt: f32, center: &Point2D, shield_radius: Option<f32>) {
        // Tick stun
        if self.stun_duration > 0.0 {
            self.stun_duration -= dt;
        }

        // Tick freeze immunity
        if self.freeze_immunity > 0.0 {
            self.freeze_immunity -= dt;
        }

        // Tick state cooldowns
        for cd in &mut self.state_cooldowns {
            if *cd > 0.0 {
                *cd -= dt;
            }
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
    // ELEMENTAL STATES (Brûlé, Trempé, Sismique, Froid, Gelé)
    // ========================================================================

    /// Durée du cooldown entre deux applications du même état par une aura
    const STATE_COOLDOWN: f32 = 1.0;
    /// Durée de l'immunité post-Gel à Trempé et Froid
    const FREEZE_IMMUNITY_DURATION: f32 = 2.0;

    /// Vérifie si l'ennemi est immunisé aux changements d'état (Gelé bloque tout)
    pub fn is_state_immune(&self) -> bool {
        self.has_state(ElementalStateKind::Frozen)
    }

    /// Vérifie si un état spécifique peut être appliqué (cooldown check)
    pub fn can_apply_state(&self, kind: ElementalStateKind) -> bool {
        self.state_cooldowns[kind.index()] <= 0.0
    }

    /// Déclenche le cooldown pour un état
    fn trigger_state_cooldown(&mut self, kind: ElementalStateKind) {
        self.state_cooldowns[kind.index()] = Self::STATE_COOLDOWN;
    }

    /// Applique l'état Brûlé (DoT)
    /// Option 4: Si l'ennemi est Trempé, Brûlé consume Trempé (évaporation)
    pub fn apply_burned(&mut self, dps: f32, duration: f32) {
        // Gelé bloque les nouveaux états
        if self.is_state_immune() {
            return;
        }

        // Cooldown check
        if !self.can_apply_state(ElementalStateKind::Burned) {
            return;
        }

        // Option 4: Évaporation - Brûlé annule Trempé
        if self.has_state(ElementalStateKind::Soaked) {
            self.elemental_states
                .retain(|s| s.kind != ElementalStateKind::Soaked);
            // Note: on applique quand même Brûlé après l'évaporation
        }

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

        self.trigger_state_cooldown(ElementalStateKind::Burned);
    }

    /// Applique l'état Trempé (Slow)
    /// Bloqué par l'immunité post-Gel
    pub fn apply_soaked(&mut self, slow_ratio: f32, duration: f32) {
        // Gelé bloque les nouveaux états
        if self.is_state_immune() {
            return;
        }

        // Option 1: Immunité post-Gel bloque Trempé
        if self.freeze_immunity > 0.0 {
            return;
        }

        // Cooldown check
        if !self.can_apply_state(ElementalStateKind::Soaked) {
            return;
        }

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

        self.trigger_state_cooldown(ElementalStateKind::Soaked);
    }

    /// Applique l'état Sismique (stun on hit)
    pub fn apply_seismic(&mut self, stun_duration: f32, state_duration: f32) {
        // Gelé bloque les nouveaux états
        if self.is_state_immune() {
            return;
        }

        // Cooldown check
        if !self.can_apply_state(ElementalStateKind::Seismic) {
            return;
        }

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

        self.trigger_state_cooldown(ElementalStateKind::Seismic);
    }

    /// Applique l'état Froid - si Trempé, déclenche Gelé
    /// Retourne true si la cible a été gelée
    /// Option 1: Bloqué par l'immunité post-Gel
    /// Option 5: Quand Gel se déclenche, Froid est consommé
    pub fn apply_cold(&mut self, freeze_duration: f32, cold_duration: f32) -> bool {
        // Gelé bloque les nouveaux états (y compris re-freeze)
        if self.is_state_immune() {
            return false;
        }

        // Option 1: Immunité post-Gel bloque Froid
        if self.freeze_immunity > 0.0 {
            return false;
        }

        // Cooldown check
        if !self.can_apply_state(ElementalStateKind::Cold) {
            return false;
        }

        // Si la cible est Trempée, elle devient Gelée
        if self.has_state(ElementalStateKind::Soaked) {
            // Option 5: Supprime Trempé et Froid (les deux sont consommés)
            self.elemental_states.retain(|s| {
                s.kind != ElementalStateKind::Soaked && s.kind != ElementalStateKind::Cold
            });
            // Applique Gelé (qui déclenchera l'immunité post-Gel)
            self.apply_frozen(freeze_duration);
            self.trigger_state_cooldown(ElementalStateKind::Cold);
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

        self.trigger_state_cooldown(ElementalStateKind::Cold);
        false
    }

    /// Applique l'état Gelé (stun complet)
    /// Note: Frozen ne peut pas être ré-appliqué tant que l'ennemi est déjà gelé
    /// Option 1: Déclenche l'immunité post-Gel quand l'état Frozen expire
    pub fn apply_frozen(&mut self, duration: f32) {
        // Si déjà gelé, ne pas ré-appliquer (empêche le perma-freeze)
        if self.has_state(ElementalStateKind::Frozen) {
            return;
        }

        self.elemental_states.push(ElementalState {
            kind: ElementalStateKind::Frozen,
            remaining: duration,
            strength: duration,
        });
        // Applique également le stun
        self.apply_stun(duration);
        // Option 1: Déclenche l'immunité post-Gel (sera active quand Frozen expire)
        // On l'ajoute maintenant pour qu'elle soit prête quand le gel se termine
        self.freeze_immunity = duration + Self::FREEZE_IMMUNITY_DURATION;
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
