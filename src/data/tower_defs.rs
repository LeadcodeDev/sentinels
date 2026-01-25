use std::f64::INFINITY;

use crate::game::elemental::TowerElement;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TowerKind {
    Sentinelle,
    Inferno,
    Glacier,
    Tesla,
    Seisme,
    Sniper,
    Forge,
    Alarme,
}

impl TowerKind {
    pub fn all() -> &'static [TowerKind] {
        &[
            TowerKind::Sentinelle,
            TowerKind::Inferno,
            TowerKind::Glacier,
            TowerKind::Tesla,
            TowerKind::Seisme,
            TowerKind::Sniper,
            TowerKind::Forge,
            TowerKind::Alarme,
        ]
    }
}

#[derive(Clone)]
pub enum EffectTarget {
    Single,
    Multi(u32),
    Area(f32),
    /// Chain: hits primary target, then chains to up to `count` enemies within `range` of each other
    Chain {
        count: u32,
        range: f32,
    },
}

#[derive(Clone)]
pub enum EffectType {
    Burn { dps: f32, duration: f32 },
    Slow { ratio: f32, duration: f32 },
    Stun { duration: f32 },
}

#[derive(Clone)]
pub enum TowerAction {
    ApplyDamage {
        target: EffectTarget,
        damage: DamageType,
    },
    ApplyEffect {
        target: EffectTarget,
        effect: EffectType,
    },
    /// Passive gold generation (gold per second)
    GoldGen { gold_per_second: f32 },
}

#[derive(Clone)]
pub enum DamageType {
    Fixed(f32),
    PercentHp(f32),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActionUpgradeTarget {
    Damage,
    AoeRadius,
    EffectDps,
    EffectDuration,
    EffectRatio,
    MaxTargets,
    GoldPerSecond,
}

#[derive(Clone)]
pub struct UpgradeableProp {
    pub base: f32,
    pub bonus_per_level: f32,
    pub max_level: u32,
    pub current_level: u32,
    pub cost_base: u32,
    pub cost_per_level: u32,
}

impl UpgradeableProp {
    pub fn value(&self) -> f32 {
        self.base + self.bonus_per_level * self.current_level as f32
    }

    pub fn cost(&self) -> u32 {
        self.cost_base + self.cost_per_level * self.current_level
    }

    pub fn can_upgrade(&self) -> bool {
        self.current_level < self.max_level
    }

    pub fn upgrade(&mut self) {
        if self.can_upgrade() {
            self.current_level += 1;
        }
    }
}

#[derive(Clone)]
pub struct ActionUpgrade {
    pub name: &'static str,
    pub prop: UpgradeableProp,
    pub applies_to: ActionUpgradeTarget,
}

#[derive(Clone)]
pub struct TowerActionDef {
    pub action: TowerAction,
    pub upgrades: Vec<ActionUpgrade>,
}

#[derive(Clone)]
pub struct TowerDef {
    pub kind: TowerKind,
    pub name: &'static str,
    pub description: &'static str,
    pub element: TowerElement,
    pub base_cost: u32,
    pub projectile_size: f32,
    pub range: UpgradeableProp,
    pub attack_speed: UpgradeableProp,
    pub actions: Vec<TowerActionDef>,
}

// --- Resolved actions (runtime, after applying upgrades) ---

#[derive(Clone)]
pub enum ResolvedDamage {
    Fixed(f32),
    PercentHp(f32),
}

#[derive(Clone)]
pub enum ResolvedEffect {
    Burn { dps: f32, duration: f32 },
    Slow { ratio: f32, duration: f32 },
    Stun { duration: f32 },
}

#[derive(Clone)]
pub enum ResolvedAction {
    ApplyDamage {
        target: EffectTarget,
        damage: ResolvedDamage,
    },
    ApplyEffect {
        target: EffectTarget,
        effect: ResolvedEffect,
    },
    GoldGen {
        gold_per_second: f32,
    },
}

impl TowerActionDef {
    pub fn resolve(&self) -> ResolvedAction {
        match &self.action {
            TowerAction::ApplyDamage { target, damage } => {
                let mut resolved_target = target.clone();
                let resolved_damage = match damage {
                    DamageType::Fixed(base) => {
                        let mut val = *base;
                        for u in &self.upgrades {
                            if u.applies_to == ActionUpgradeTarget::Damage {
                                val += u.prop.bonus_per_level * u.prop.current_level as f32;
                            }
                        }
                        ResolvedDamage::Fixed(val)
                    }
                    DamageType::PercentHp(base) => {
                        let mut val = *base;
                        for u in &self.upgrades {
                            if u.applies_to == ActionUpgradeTarget::Damage {
                                val += u.prop.bonus_per_level * u.prop.current_level as f32;
                            }
                        }
                        ResolvedDamage::PercentHp(val)
                    }
                };
                // Apply AoeRadius and MaxTargets upgrades
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::AoeRadius => {
                            if let EffectTarget::Area(ref mut r) = resolved_target {
                                *r += u.prop.bonus_per_level * u.prop.current_level as f32;
                            }
                        }
                        ActionUpgradeTarget::MaxTargets => {
                            if let EffectTarget::Multi(ref mut n) = resolved_target {
                                *n += (u.prop.bonus_per_level * u.prop.current_level as f32) as u32;
                            }
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ApplyDamage {
                    target: resolved_target,
                    damage: resolved_damage,
                }
            }
            TowerAction::ApplyEffect { target, effect } => {
                let mut resolved_target = target.clone();
                let resolved_effect = match effect {
                    EffectType::Burn { dps, duration } => {
                        let mut d = *dps;
                        let mut dur = *duration;
                        for u in &self.upgrades {
                            match u.applies_to {
                                ActionUpgradeTarget::EffectDps => {
                                    d += u.prop.bonus_per_level * u.prop.current_level as f32;
                                }
                                ActionUpgradeTarget::EffectDuration => {
                                    dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                                }
                                _ => {}
                            }
                        }
                        ResolvedEffect::Burn {
                            dps: d,
                            duration: dur,
                        }
                    }
                    EffectType::Slow { ratio, duration } => {
                        let mut r = *ratio;
                        let mut dur = *duration;
                        for u in &self.upgrades {
                            match u.applies_to {
                                ActionUpgradeTarget::EffectRatio => {
                                    r += u.prop.bonus_per_level * u.prop.current_level as f32;
                                }
                                ActionUpgradeTarget::EffectDuration => {
                                    dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                                }
                                _ => {}
                            }
                        }
                        ResolvedEffect::Slow {
                            ratio: r,
                            duration: dur,
                        }
                    }
                    EffectType::Stun { duration } => {
                        let mut dur = *duration;
                        for u in &self.upgrades {
                            if u.applies_to == ActionUpgradeTarget::EffectDuration {
                                dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                            }
                        }
                        ResolvedEffect::Stun { duration: dur }
                    }
                };
                // Apply AoeRadius and MaxTargets upgrades to effect target too
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::AoeRadius => {
                            if let EffectTarget::Area(ref mut r) = resolved_target {
                                *r += u.prop.bonus_per_level * u.prop.current_level as f32;
                            }
                        }
                        ActionUpgradeTarget::MaxTargets => {
                            if let EffectTarget::Multi(ref mut n) = resolved_target {
                                *n += (u.prop.bonus_per_level * u.prop.current_level as f32) as u32;
                            }
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ApplyEffect {
                    target: resolved_target,
                    effect: resolved_effect,
                }
            }
            TowerAction::GoldGen { gold_per_second } => {
                let mut gps = *gold_per_second;
                for u in &self.upgrades {
                    if u.applies_to == ActionUpgradeTarget::GoldPerSecond {
                        gps += u.prop.bonus_per_level * u.prop.current_level as f32;
                    }
                }
                ResolvedAction::GoldGen {
                    gold_per_second: gps,
                }
            }
        }
    }
}

// --- TowerBuilder ---

pub struct TowerBuilder {
    kind: TowerKind,
    name: &'static str,
    description: &'static str,
    element: TowerElement,
    base_cost: u32,
    projectile_size: f32,
    range: (f32, f32, u32),
    attack_speed: (f32, f32, u32),
    actions: Vec<TowerActionDef>,
}

impl TowerBuilder {
    pub fn new(kind: TowerKind, name: &'static str, element: TowerElement) -> Self {
        Self {
            kind,
            name,
            description: "",
            element,
            base_cost: 50,
            projectile_size: 4.0,
            range: (100.0, 15.0, 5),
            attack_speed: (1.0, 0.15, 5),
            actions: Vec::new(),
        }
    }

    pub fn description(mut self, desc: &'static str) -> Self {
        self.description = desc;
        self
    }

    pub fn cost(mut self, cost: u32) -> Self {
        self.base_cost = cost;
        self
    }

    pub fn projectile_size(mut self, size: f32) -> Self {
        self.projectile_size = size;
        self
    }

    pub fn range(mut self, base: f32, bonus: f32, max_level: u32) -> Self {
        self.range = (base, bonus, max_level);
        self
    }

    pub fn attack_speed(mut self, base: f32, bonus: f32, max_level: u32) -> Self {
        self.attack_speed = (base, bonus, max_level);
        self
    }

    pub fn action(mut self, action: TowerAction) -> Self {
        self.actions.push(TowerActionDef {
            action,
            upgrades: Vec::new(),
        });
        self
    }

    pub fn action_with_upgrades(
        mut self,
        action: TowerAction,
        upgrades: Vec<(&'static str, ActionUpgradeTarget, f32, u32)>,
    ) -> Self {
        let action_upgrades = upgrades
            .into_iter()
            .map(|(name, target, bonus, max_level)| {
                // Extract the base value from the action based on upgrade target
                let base = match (&action, target) {
                    (TowerAction::ApplyDamage { damage, .. }, ActionUpgradeTarget::Damage) => {
                        match damage {
                            DamageType::Fixed(v) | DamageType::PercentHp(v) => *v,
                        }
                    }
                    (
                        TowerAction::ApplyDamage { target: t, .. },
                        ActionUpgradeTarget::AoeRadius,
                    ) => {
                        if let EffectTarget::Area(r) = t {
                            *r
                        } else {
                            0.0
                        }
                    }
                    (
                        TowerAction::ApplyDamage { target: t, .. },
                        ActionUpgradeTarget::MaxTargets,
                    ) => {
                        if let EffectTarget::Multi(n) = t {
                            *n as f32
                        } else {
                            0.0
                        }
                    }
                    (TowerAction::ApplyEffect { effect, .. }, ActionUpgradeTarget::EffectDps) => {
                        if let EffectType::Burn { dps, .. } = effect {
                            *dps
                        } else {
                            0.0
                        }
                    }
                    (
                        TowerAction::ApplyEffect { effect, .. },
                        ActionUpgradeTarget::EffectDuration,
                    ) => match effect {
                        EffectType::Burn { duration, .. }
                        | EffectType::Slow { duration, .. }
                        | EffectType::Stun { duration } => *duration,
                    },
                    (TowerAction::ApplyEffect { effect, .. }, ActionUpgradeTarget::EffectRatio) => {
                        if let EffectType::Slow { ratio, .. } = effect {
                            *ratio
                        } else {
                            0.0
                        }
                    }
                    (
                        TowerAction::GoldGen { gold_per_second },
                        ActionUpgradeTarget::GoldPerSecond,
                    ) => *gold_per_second,
                    _ => 0.0,
                };
                ActionUpgrade {
                    name,
                    prop: UpgradeableProp {
                        base,
                        bonus_per_level: bonus,
                        max_level,
                        current_level: 0,
                        cost_base: 30,
                        cost_per_level: 25,
                    },
                    applies_to: target,
                }
            })
            .collect();
        self.actions.push(TowerActionDef {
            action,
            upgrades: action_upgrades,
        });
        self
    }

    pub fn build(self) -> TowerDef {
        TowerDef {
            kind: self.kind,
            name: self.name,
            description: self.description,
            element: self.element,
            base_cost: self.base_cost,
            projectile_size: self.projectile_size,
            range: UpgradeableProp {
                base: self.range.0,
                bonus_per_level: self.range.1,
                max_level: self.range.2,
                current_level: 0,
                cost_base: 30,
                cost_per_level: 25,
            },
            attack_speed: UpgradeableProp {
                base: self.attack_speed.0,
                bonus_per_level: self.attack_speed.1,
                max_level: self.attack_speed.2,
                current_level: 0,
                cost_base: 30,
                cost_per_level: 25,
            },
            actions: self.actions,
        }
    }
}

// --- Tower definitions ---

// Stats des tourelles basées sur les principes D&D DPR (Damage Per Round)
// Chaque tour a un rôle distinct inspiré des classes/sorts D&D
pub fn all_tower_defs() -> Vec<TowerDef> {
    vec![
        // SENTINELLE - Fighter: Baseline fiable, polyvalente
        // DPS théorique: 10 × 1.0 = 10 DPS
        TowerBuilder::new(TowerKind::Sentinelle, "Sentinelle", TowerElement::Neutral)
            .description("Tour basique equilibree")
            .cost(50)
            .range(140.0, 12.0, 5) // Portée standard, upgrade +12/lvl
            .attack_speed(1.0, 0.1, 5) // AS standard, upgrade +0.1/lvl
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: DamageType::Fixed(10.0), // Buffé de 8 à 10
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 2.0, 5)], // +2/lvl (était +3)
            )
            .build(),
        // INFERNO - Evocation Wizard: Contrôle de zone + DoT
        // DPS théorique: 8 × 0.8 + 4 burn = 10.4 DPS en AoE
        TowerBuilder::new(TowerKind::Inferno, "Tour Inferno", TowerElement::Fire)
            .description("Degats de zone + brulure")
            .cost(90) // Augmenté de 80 à 90
            .range(110.0, 12.0, 5)
            .attack_speed(0.8, 0.1, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Area(45.0),
                    damage: DamageType::Fixed(8.0), // Réduit de 10 à 8 (compensé par burn)
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 2.0, 5),
                    ("Zone", ActionUpgradeTarget::AoeRadius, 8.0, 5), // Réduit de 10 à 8
                ],
            )
            .action_with_upgrades(
                TowerAction::ApplyEffect {
                    target: EffectTarget::Area(45.0),
                    effect: EffectType::Burn {
                        dps: 4.0,      // Augmenté de 3 à 4
                        duration: 2.5, // Augmenté de 2 à 2.5
                    },
                },
                vec![
                    ("Brulure DPS", ActionUpgradeTarget::EffectDps, 1.5, 5),
                    ("Brulure duree", ActionUpgradeTarget::EffectDuration, 0.4, 5),
                ],
            )
            .build(),
        // GLACIER - Enchantment Wizard: Crowd Control spécialiste
        // DPS théorique: 7 × 0.8 = 5.6 DPS + slow (valeur utilitaire)
        TowerBuilder::new(TowerKind::Glacier, "Tour Glacier", TowerElement::Water)
            .description("Ralentit les ennemis")
            .cost(70) // Augmenté de 65 à 70
            .range(130.0, 12.0, 5)
            .attack_speed(0.8, 0.1, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: DamageType::Fixed(7.0), // Buffé de 5 à 7
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 2.0, 5)],
            )
            .action_with_upgrades(
                TowerAction::ApplyEffect {
                    target: EffectTarget::Single,
                    effect: EffectType::Slow {
                        ratio: 0.45, // Réduit de 0.5 à 0.45 (moins oppressif)
                        duration: 2.0,
                    },
                },
                vec![
                    ("Ralentissement", ActionUpgradeTarget::EffectRatio, 0.03, 5), // Réduit de 0.05
                    ("Duree slow", ActionUpgradeTarget::EffectDuration, 0.25, 5),
                ],
            )
            .build(),
        // TESLA - Chain Lightning: Multi-cibles rapide
        // DPS théorique: 6 × 3 cibles × 1.2 = 21.6 DPS répartis
        TowerBuilder::new(TowerKind::Tesla, "Tour Tesla", TowerElement::Electric)
            .description("Attaque multi-cibles electrique")
            .cost(100) // Augmenté de 90 à 100
            .range(120.0, 12.0, 5) // Réduit de 125 à 120
            .attack_speed(1.2, 0.12, 5) // Réduit de 1.5 à 1.2
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Multi(3),
                    damage: DamageType::Fixed(6.0), // Réduit de 7 à 6
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 2.0, 5),
                    ("Cibles", ActionUpgradeTarget::MaxTargets, 1.0, 3),
                ],
            )
            .build(),
        // SEISME - Earthquake: Burst AoE massif, très lent
        // DPS théorique: 22 × 0.4 = 8.8 DPS en AoE (burst)
        TowerBuilder::new(TowerKind::Seisme, "Tour Seisme", TowerElement::Earth)
            .description("Degats massifs de zone")
            .cost(120) // Augmenté de 110 à 120
            .range(95.0, 12.0, 5) // Légèrement augmenté de 90 à 95
            .attack_speed(0.4, 0.06, 5) // Légèrement augmenté de 0.35 à 0.4
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Area(55.0),
                    damage: DamageType::Fixed(22.0), // Augmenté de 18 à 22
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 3.0, 5), // Réduit de 4 à 3
                    ("Zone", ActionUpgradeTarget::AoeRadius, 8.0, 5),
                ],
            )
            .build(),
        // SNIPER - Siege Weapon: Longue portée, tir lent mais puissant
        // DPS théorique: 15 / 4.0 = 3.75 DPS (mais chaîné sur 4 cibles)
        TowerBuilder::new(TowerKind::Sniper, "Tour Sniper", TowerElement::Electric)
            .description("Tir longue portee chaine")
            .cost(130) // Augmenté de 90 à 130
            .range(280.0, 15.0, 5) // Réduit de 300 à 280
            .attack_speed(4.0, 0.15, 5) // Réduit de 5.0 à 4.0
            .projectile_size(2.0)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Chain {
                        count: 4,     // Réduit de 5 à 4
                        range: 180.0, // Réduit de 200 à 180
                    },
                    damage: DamageType::Fixed(15.0), // Réduit de 18 à 15
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 3.0, 5)],
            )
            .build(),
        // FORGE - Economy: Génération passive d'or
        TowerBuilder::new(TowerKind::Forge, "Forge", TowerElement::Earth)
            .description("Genere de l'or passivement")
            .cost(180) // Augmenté de 150 à 180 (ROI ~90s base)
            .range(0.0, 0.0, 0)
            .attack_speed(0.0, 0.0, 0)
            .action_with_upgrades(
                TowerAction::GoldGen {
                    gold_per_second: 2.0,
                },
                vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.8, 5)], // Réduit de 1.0 à 0.8
            )
            .build(),
        // ALARME - Utility: Notifications système
        TowerBuilder::new(TowerKind::Alarme, "Alarme", TowerElement::Electric)
            .description("Notifications systeme configurables")
            .cost(75)
            .build(),
    ]
}

pub fn get_def(kind: TowerKind) -> TowerDef {
    all_tower_defs()
        .into_iter()
        .find(|d| d.kind == kind)
        .unwrap_or_else(|| all_tower_defs().into_iter().next().unwrap())
}
