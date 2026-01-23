use crate::game::elemental::TowerElement;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TowerKind {
    Sentinelle,
    Inferno,
    Glacier,
    Tesla,
    Seisme,
    Sniper,
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
        ]
    }
}

#[derive(Clone)]
pub enum EffectTarget {
    Single,
    Multi(u32),
    Area(f32),
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
            .map(|(name, target, bonus, max_level)| ActionUpgrade {
                name,
                prop: UpgradeableProp {
                    base: 0.0,
                    bonus_per_level: bonus,
                    max_level,
                    current_level: 0,
                    cost_base: 30,
                    cost_per_level: 25,
                },
                applies_to: target,
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

pub fn all_tower_defs() -> Vec<TowerDef> {
    vec![
        TowerBuilder::new(TowerKind::Sentinelle, "Sentinelle", TowerElement::Neutral)
            .description("Tour basique equilibree")
            .cost(50)
            .range(140.0, 15.0, 5)
            .attack_speed(1.0, 0.15, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: DamageType::Fixed(8.0),
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 3.0, 5)],
            )
            .build(),
        TowerBuilder::new(TowerKind::Inferno, "Tour Inferno", TowerElement::Fire)
            .description("Degats de zone + brulure")
            .cost(80)
            .range(110.0, 15.0, 5)
            .attack_speed(0.8, 0.15, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Area(45.0),
                    damage: DamageType::Fixed(10.0),
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 3.0, 5),
                    ("Zone", ActionUpgradeTarget::AoeRadius, 10.0, 5),
                ],
            )
            .action_with_upgrades(
                TowerAction::ApplyEffect {
                    target: EffectTarget::Area(45.0),
                    effect: EffectType::Burn {
                        dps: 3.0,
                        duration: 2.0,
                    },
                },
                vec![
                    ("Brulure DPS", ActionUpgradeTarget::EffectDps, 2.0, 5),
                    ("Brulure duree", ActionUpgradeTarget::EffectDuration, 0.5, 5),
                ],
            )
            .build(),
        TowerBuilder::new(TowerKind::Glacier, "Tour Glacier", TowerElement::Water)
            .description("Ralentit les ennemis")
            .cost(65)
            .range(135.0, 15.0, 5)
            .attack_speed(0.8, 0.15, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: DamageType::Fixed(5.0),
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 3.0, 5)],
            )
            .action_with_upgrades(
                TowerAction::ApplyEffect {
                    target: EffectTarget::Single,
                    effect: EffectType::Slow {
                        ratio: 0.5,
                        duration: 2.0,
                    },
                },
                vec![
                    ("Ralentissement", ActionUpgradeTarget::EffectRatio, 0.05, 5),
                    ("Duree slow", ActionUpgradeTarget::EffectDuration, 0.3, 5),
                ],
            )
            .build(),
        TowerBuilder::new(TowerKind::Tesla, "Tour Tesla", TowerElement::Electric)
            .description("Attaque rapide electrique")
            .cost(90)
            .range(125.0, 15.0, 5)
            .attack_speed(1.5, 0.2, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Multi(3),
                    damage: DamageType::Fixed(7.0),
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 3.0, 5),
                    ("Cibles", ActionUpgradeTarget::MaxTargets, 1.0, 3),
                ],
            )
            .build(),
        TowerBuilder::new(TowerKind::Seisme, "Tour Seisme", TowerElement::Earth)
            .description("Degats massifs de zone")
            .cost(110)
            .range(90.0, 15.0, 5)
            .attack_speed(0.35, 0.05, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Area(55.0),
                    damage: DamageType::Fixed(18.0),
                },
                vec![
                    ("Degats", ActionUpgradeTarget::Damage, 4.0, 5),
                    ("Zone", ActionUpgradeTarget::AoeRadius, 10.0, 5),
                ],
            )
            .build(),
        TowerBuilder::new(TowerKind::Sniper, "Tour Sniper", TowerElement::Electric)
            .description("Attaque rapide electrique")
            .cost(90)
            .range(300.0, 15.0, 5)
            .attack_speed(5.0, 0.2, 5)
            .action_with_upgrades(
                TowerAction::ApplyDamage {
                    target: EffectTarget::Single,
                    damage: DamageType::Fixed(18.0),
                },
                vec![("Degats", ActionUpgradeTarget::Damage, 4.0, 5)],
            )
            .build(),
    ]
}

pub fn get_def(kind: TowerKind) -> TowerDef {
    all_tower_defs()
        .into_iter()
        .find(|d| d.kind == kind)
        .unwrap_or_else(|| all_tower_defs().into_iter().next().unwrap())
}
