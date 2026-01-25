use crate::game::elemental::TowerElement;

/// Priorité de ciblage des tourelles
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum TargetPriority {
    /// Cible l'ennemi le plus proche (par défaut)
    #[default]
    Closest,
    /// Cible l'ennemi le plus éloigné
    Farthest,
    /// Cible l'ennemi avec le plus de HP
    HighestHp,
    /// Cible l'ennemi avec le moins de HP
    LowestHp,
}

impl TargetPriority {
    /// Retourne toutes les priorités disponibles
    pub fn all() -> &'static [TargetPriority] {
        &[
            TargetPriority::Closest,
            TargetPriority::Farthest,
            TargetPriority::HighestHp,
            TargetPriority::LowestHp,
        ]
    }

    /// Retourne le nom affiché de la priorité
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetPriority::Closest => "Plus proche",
            TargetPriority::Farthest => "Plus loin",
            TargetPriority::HighestHp => "HP max",
            TargetPriority::LowestHp => "HP min",
        }
    }

    /// Passe à la priorité suivante (cycle)
    pub fn next(&self) -> Self {
        match self {
            TargetPriority::Closest => TargetPriority::Farthest,
            TargetPriority::Farthest => TargetPriority::HighestHp,
            TargetPriority::HighestHp => TargetPriority::LowestHp,
            TargetPriority::LowestHp => TargetPriority::Closest,
        }
    }
}

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
    Void,
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
            TowerKind::Void,
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

// ============================================================================
// UPGRADEABLE PROPERTIES
// ============================================================================

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
    pub fn new(base: f32, bonus_per_level: f32, max_level: u32) -> Self {
        Self {
            base,
            bonus_per_level,
            max_level,
            current_level: 0,
            cost_base: 30,
            cost_per_level: 25,
        }
    }

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

// ============================================================================
// PROPERTY - Configurable as Fixed or Upgradable
// ============================================================================

/// Propriété numérique pouvant être fixe ou améliorable
#[derive(Clone)]
pub enum Property {
    /// Valeur fixe, non upgradable
    Fixed(f32),
    /// Valeur upgradable avec progression
    Upgradable(UpgradeableProp),
}

impl Property {
    pub fn fixed(value: f32) -> Self {
        Property::Fixed(value)
    }

    pub fn upgradable(base: f32, bonus_per_level: f32, max_level: u32) -> Self {
        Property::Upgradable(UpgradeableProp::new(base, bonus_per_level, max_level))
    }

    pub fn value(&self) -> f32 {
        match self {
            Property::Fixed(v) => *v,
            Property::Upgradable(prop) => prop.value(),
        }
    }

    pub fn is_upgradable(&self) -> bool {
        matches!(self, Property::Upgradable(_))
    }

    pub fn as_upgradable(&self) -> Option<&UpgradeableProp> {
        match self {
            Property::Upgradable(prop) => Some(prop),
            Property::Fixed(_) => None,
        }
    }

    pub fn as_upgradable_mut(&mut self) -> Option<&mut UpgradeableProp> {
        match self {
            Property::Upgradable(prop) => Some(prop),
            Property::Fixed(_) => None,
        }
    }
}

// ============================================================================
// SKILL SYSTEM
// ============================================================================

/// Type de compétence
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillType {
    /// Active : une seule à la fois, possède range/attack_speed, tire des projectiles
    Active,
    /// Passive : toujours active quand achetée, effets continus (ex: génération d'or)
    Passive,
}

/// Définition d'une compétence de tourelle (template immuable)
#[derive(Clone)]
pub struct TowerSkillDef {
    /// Identifiant unique dans la tourelle (0, 1, ou 2)
    pub id: u8,
    /// Nom affiché
    pub name: &'static str,
    /// Description pour tooltip
    pub description: &'static str,
    /// Icône (emoji ou lettre)
    pub icon: &'static str,
    /// Type de compétence (active ou passive)
    pub skill_type: SkillType,
    /// Coût d'achat en or
    pub purchase_cost: u32,
    /// Portée (seulement pour Active)
    pub range: Option<Property>,
    /// Vitesse d'attaque (seulement pour Active)
    pub attack_speed: Option<Property>,
    /// Taille du projectile
    pub projectile_size: f32,
    /// Permet de changer la cible (affiche le bouton de priorité)
    pub can_change_target: bool,
    /// Actions effectuées par cette compétence
    pub actions: Vec<TowerActionDef>,
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

/// Définition d'une tourelle avec système de compétences
#[derive(Clone)]
pub struct TowerDef {
    pub kind: TowerKind,
    pub name: &'static str,
    pub description: &'static str,
    pub element: TowerElement,
    /// Coût de base pour placer la tourelle
    pub base_cost: u32,
    /// Priorité de ciblage par défaut
    pub default_target_priority: TargetPriority,
    /// Exactement 3 compétences par tourelle
    pub skills: [TowerSkillDef; 3],
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

// ============================================================================
// SKILL BUILDER
// ============================================================================

pub struct SkillBuilder {
    id: u8,
    name: &'static str,
    description: &'static str,
    icon: &'static str,
    skill_type: SkillType,
    purchase_cost: u32,
    range: Option<Property>,
    attack_speed: Option<Property>,
    projectile_size: f32,
    can_change_target: bool,
    actions: Vec<TowerActionDef>,
}

impl SkillBuilder {
    /// Crée un builder pour une compétence active
    pub fn active(id: u8, name: &'static str) -> Self {
        Self {
            id,
            name,
            description: "",
            icon: "A",
            skill_type: SkillType::Active,
            purchase_cost: 50,
            range: None,
            attack_speed: None,
            projectile_size: 4.0,
            can_change_target: false,
            actions: Vec::new(),
        }
    }

    /// Crée un builder pour une compétence passive
    pub fn passive(id: u8, name: &'static str) -> Self {
        Self {
            id,
            name,
            description: "",
            icon: "P",
            skill_type: SkillType::Passive,
            purchase_cost: 50,
            range: None,
            attack_speed: None,
            projectile_size: 0.0,
            can_change_target: false,
            actions: Vec::new(),
        }
    }

    pub fn description(mut self, desc: &'static str) -> Self {
        self.description = desc;
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = icon;
        self
    }

    pub fn cost(mut self, cost: u32) -> Self {
        self.purchase_cost = cost;
        self
    }

    /// Définit une portée fixe (non upgradable)
    pub fn range_fixed(mut self, value: f32) -> Self {
        self.range = Some(Property::Fixed(value));
        self
    }

    /// Définit une portée upgradable
    pub fn range(mut self, base: f32, bonus: f32, max_level: u32) -> Self {
        self.range = Some(Property::upgradable(base, bonus, max_level));
        self
    }

    /// Définit une vitesse d'attaque fixe (non upgradable)
    pub fn attack_speed_fixed(mut self, value: f32) -> Self {
        self.attack_speed = Some(Property::Fixed(value));
        self
    }

    /// Définit une vitesse d'attaque upgradable
    pub fn attack_speed(mut self, base: f32, bonus: f32, max_level: u32) -> Self {
        self.attack_speed = Some(Property::upgradable(base, bonus, max_level));
        self
    }

    pub fn projectile_size(mut self, size: f32) -> Self {
        self.projectile_size = size;
        self
    }

    /// Permet de changer la priorité de ciblage via l'UI
    pub fn can_change_target(mut self) -> Self {
        self.can_change_target = true;
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
                let base = extract_base_value(&action, target);
                ActionUpgrade {
                    name,
                    prop: UpgradeableProp::new(base, bonus, max_level),
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

    pub fn build(self) -> TowerSkillDef {
        TowerSkillDef {
            id: self.id,
            name: self.name,
            description: self.description,
            icon: self.icon,
            skill_type: self.skill_type,
            purchase_cost: self.purchase_cost,
            range: self.range,
            attack_speed: self.attack_speed,
            projectile_size: self.projectile_size,
            can_change_target: self.can_change_target,
            actions: self.actions,
        }
    }
}

/// Extrait la valeur de base d'une action selon le type d'upgrade
fn extract_base_value(action: &TowerAction, target: ActionUpgradeTarget) -> f32 {
    match (action, target) {
        (TowerAction::ApplyDamage { damage, .. }, ActionUpgradeTarget::Damage) => match damage {
            DamageType::Fixed(v) | DamageType::PercentHp(v) => *v,
        },
        (TowerAction::ApplyDamage { target: t, .. }, ActionUpgradeTarget::AoeRadius) => {
            if let EffectTarget::Area(r) = t {
                *r
            } else {
                0.0
            }
        }
        (TowerAction::ApplyDamage { target: t, .. }, ActionUpgradeTarget::MaxTargets) => {
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
        (TowerAction::ApplyEffect { effect, .. }, ActionUpgradeTarget::EffectDuration) => {
            match effect {
                EffectType::Burn { duration, .. }
                | EffectType::Slow { duration, .. }
                | EffectType::Stun { duration } => *duration,
            }
        }
        (TowerAction::ApplyEffect { effect, .. }, ActionUpgradeTarget::EffectRatio) => {
            if let EffectType::Slow { ratio, .. } = effect {
                *ratio
            } else {
                0.0
            }
        }
        (TowerAction::GoldGen { gold_per_second }, ActionUpgradeTarget::GoldPerSecond) => {
            *gold_per_second
        }
        _ => 0.0,
    }
}

// ============================================================================
// TOWER BUILDER
// ============================================================================

pub struct TowerBuilder {
    kind: TowerKind,
    name: &'static str,
    description: &'static str,
    element: TowerElement,
    base_cost: u32,
    default_target_priority: TargetPriority,
    skills: Vec<TowerSkillDef>,
}

impl TowerBuilder {
    pub fn new(kind: TowerKind, name: &'static str, element: TowerElement) -> Self {
        Self {
            kind,
            name,
            description: "",
            element,
            base_cost: 50,
            default_target_priority: TargetPriority::default(),
            skills: Vec::new(),
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

    /// Définit la priorité de ciblage par défaut
    pub fn target_priority(mut self, priority: TargetPriority) -> Self {
        self.default_target_priority = priority;
        self
    }

    /// Ajoute une compétence à la tourelle (max 3)
    pub fn skill(mut self, skill: TowerSkillDef) -> Self {
        assert!(
            self.skills.len() < 3,
            "Une tourelle ne peut avoir que 3 competences"
        );
        self.skills.push(skill);
        self
    }

    pub fn build(self) -> TowerDef {
        assert_eq!(
            self.skills.len(),
            3,
            "Une tourelle doit avoir exactement 3 competences"
        );
        TowerDef {
            kind: self.kind,
            name: self.name,
            description: self.description,
            element: self.element,
            base_cost: self.base_cost,
            default_target_priority: self.default_target_priority,
            skills: [
                self.skills[0].clone(),
                self.skills[1].clone(),
                self.skills[2].clone(),
            ],
        }
    }
}

// ============================================================================
// TOWER DEFINITIONS
// ============================================================================

// Chaque tourelle a 3 compétences (actives ou passives)
// Les compétences doivent être achetées avant d'être utilisables
// Une seule compétence Active peut être active à la fois

pub fn all_tower_defs() -> Vec<TowerDef> {
    vec![
        // ===================================================================
        // SENTINELLE - Tour polyvalente pour débutants
        // ===================================================================
        TowerBuilder::new(TowerKind::Sentinelle, "Sentinelle", TowerElement::Neutral)
            .description("Tour basique equilibree avec 3 modes de tir")
            .cost(50)
            // Skill 1: Tir Standard - Baseline équilibré
            .skill(
                SkillBuilder::active(0, "Tir Standard")
                    .description("Attaque equilibree a cible unique")
                    .icon("S")
                    .cost(30)
                    .range(140.0, 12.0, 5)
                    .attack_speed(1.0, 0.1, 5)
                    .can_change_target()
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(10.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 2.0, 5)],
                    )
                    .build(),
            )
            // Skill 2: Tir Rapide - Cadence élevée, dégâts réduits
            .skill(
                SkillBuilder::active(1, "Tir Rapide")
                    .description("Cadence elevee mais degats reduits")
                    .icon("R")
                    .cost(60)
                    .range_fixed(100.0) // Portée fixe, non upgradable
                    .attack_speed(2.0, 0.15, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(5.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 1.0, 5)],
                    )
                    .build(),
            )
            // Skill 3: Vigilance - Génération passive d'or
            .skill(
                SkillBuilder::passive(2, "Vigilance")
                    .description("Genere de l'or passivement")
                    .icon("V")
                    .cost(100)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.0,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.5, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // INFERNO - Tour de feu, dégâts de zone + brûlure
        // ===================================================================
        TowerBuilder::new(TowerKind::Inferno, "Inferno", TowerElement::Fire)
            .description("Tour de feu avec degats de zone")
            .cost(80)
            .skill(
                SkillBuilder::active(0, "Boule de Feu")
                    .description("Explosion de zone avec brulure")
                    .icon("F")
                    .cost(40)
                    .range(110.0, 10.0, 5)
                    .attack_speed(0.8, 0.08, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Area(45.0),
                            damage: DamageType::Fixed(8.0),
                        },
                        vec![
                            ("Degats", ActionUpgradeTarget::Damage, 2.0, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 8.0, 5),
                        ],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Area(45.0),
                            effect: EffectType::Burn {
                                dps: 4.0,
                                duration: 2.5,
                            },
                        },
                        vec![("Brulure", ActionUpgradeTarget::EffectDps, 1.5, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Mur de Flammes")
                    .description("Zone de brulure prolongee")
                    .icon("M")
                    .cost(75)
                    .range_fixed(80.0)
                    .attack_speed_fixed(0.5)
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Area(60.0),
                            effect: EffectType::Burn {
                                dps: 6.0,
                                duration: 4.0,
                            },
                        },
                        vec![
                            ("Brulure", ActionUpgradeTarget::EffectDps, 2.0, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 10.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Chaleur Intense")
                    .description("Genere de l'or grace a la forge")
                    .icon("C")
                    .cost(120)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.5,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.75, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // GLACIER - Tour de glace, ralentissement
        // ===================================================================
        TowerBuilder::new(TowerKind::Glacier, "Glacier", TowerElement::Water)
            .description("Tour de glace specialisee dans le controle")
            .cost(70)
            .skill(
                SkillBuilder::active(0, "Trait de Givre")
                    .description("Tir unique avec ralentissement")
                    .icon("G")
                    .cost(35)
                    .range(130.0, 10.0, 5)
                    .attack_speed(0.8, 0.08, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(7.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 1.5, 5)],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Single,
                            effect: EffectType::Slow {
                                ratio: 0.45,
                                duration: 2.0,
                            },
                        },
                        vec![("Ralentissement", ActionUpgradeTarget::EffectRatio, 0.03, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Blizzard")
                    .description("Zone de ralentissement massive")
                    .icon("B")
                    .cost(80)
                    .range_fixed(100.0)
                    .attack_speed(0.6, 0.05, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Area(70.0),
                            effect: EffectType::Slow {
                                ratio: 0.3,
                                duration: 3.0,
                            },
                        },
                        vec![
                            ("Ralentissement", ActionUpgradeTarget::EffectRatio, 0.02, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 10.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Permafrost")
                    .description("Genere de l'or par cristallisation")
                    .icon("P")
                    .cost(90)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.2,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.6, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // TESLA - Tour électrique, multi-cibles
        // ===================================================================
        TowerBuilder::new(TowerKind::Tesla, "Tesla", TowerElement::Electric)
            .description("Tour electrique a cibles multiples")
            .cost(100)
            .skill(
                SkillBuilder::active(0, "Arc Electrique")
                    .description("Frappe plusieurs ennemis simultanement")
                    .icon("A")
                    .cost(50)
                    .range(120.0, 10.0, 5)
                    .attack_speed(1.2, 0.1, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Multi(3),
                            damage: DamageType::Fixed(6.0),
                        },
                        vec![
                            ("Degats", ActionUpgradeTarget::Damage, 1.5, 5),
                            ("Cibles", ActionUpgradeTarget::MaxTargets, 1.0, 3),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Chaine d'Eclairs")
                    .description("L'eclair rebondit entre les ennemis")
                    .icon("C")
                    .cost(90)
                    .range(150.0, 12.0, 5)
                    .attack_speed_fixed(0.8)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Chain {
                                count: 4,
                                range: 100.0,
                            },
                            damage: DamageType::Fixed(10.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 2.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Generateur")
                    .description("Produit de l'energie convertie en or")
                    .icon("G")
                    .cost(110)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.8,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.8, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // SEISME - Tour de terre, dégâts massifs de zone
        // ===================================================================
        TowerBuilder::new(TowerKind::Seisme, "Seisme", TowerElement::Earth)
            .description("Tour de terre avec degats de zone massifs")
            .cost(120)
            .skill(
                SkillBuilder::active(0, "Secousse")
                    .description("Impact de zone devastateur")
                    .icon("S")
                    .cost(60)
                    .range(95.0, 8.0, 5)
                    .attack_speed(0.4, 0.04, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Area(55.0),
                            damage: DamageType::Fixed(22.0),
                        },
                        vec![
                            ("Degats", ActionUpgradeTarget::Damage, 3.0, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 8.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Faille")
                    .description("Cree une fissure qui stun les ennemis")
                    .icon("F")
                    .cost(100)
                    .range_fixed(80.0)
                    .attack_speed_fixed(0.25)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Area(70.0),
                            damage: DamageType::Fixed(15.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 2.5, 5)],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Area(70.0),
                            effect: EffectType::Stun { duration: 1.0 },
                        },
                        vec![("Stun", ActionUpgradeTarget::EffectDuration, 0.2, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Mine d'Or")
                    .description("Extrait de l'or du sol")
                    .icon("M")
                    .cost(150)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 2.5,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 1.0, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // SNIPER - Tour longue portée
        // ===================================================================
        TowerBuilder::new(TowerKind::Sniper, "Sniper", TowerElement::Electric)
            .description("Tour de precision a longue portee")
            .cost(130)
            .target_priority(TargetPriority::HighestHp)
            .skill(
                SkillBuilder::active(0, "Tir de Precision")
                    .description("Tir unique tres longue portee")
                    .icon("P")
                    .cost(65)
                    .range(280.0, 15.0, 5)
                    .attack_speed(0.25, 0.02, 5)
                    .projectile_size(2.0)
                    .can_change_target()
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(25.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 4.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Tir Chaine")
                    .description("Le projectile rebondit entre cibles")
                    .icon("C")
                    .cost(100)
                    .range(250.0, 12.0, 5)
                    .attack_speed_fixed(0.2)
                    .projectile_size(2.0)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Chain {
                                count: 4,
                                range: 180.0,
                            },
                            damage: DamageType::Fixed(15.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 3.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Observation")
                    .description("Repere des ressources cachees")
                    .icon("O")
                    .cost(80)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.0,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.5, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // FORGE - Tour économique
        // ===================================================================
        TowerBuilder::new(TowerKind::Forge, "Forge", TowerElement::Earth)
            .description("Tour economique specialisee dans la production d'or")
            .cost(180)
            .skill(
                SkillBuilder::passive(0, "Production")
                    .description("Genere de l'or de base")
                    .icon("P")
                    .cost(50)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 2.0,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.8, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Raffinage")
                    .description("Production d'or amelioree")
                    .icon("R")
                    .cost(100)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 3.0,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 1.2, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(2, "Marteau")
                    .description("Peut aussi attaquer au corps a corps")
                    .icon("M")
                    .cost(80)
                    .range_fixed(50.0)
                    .attack_speed(0.5, 0.05, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(20.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 3.0, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // VOID - Tour anti-tank (% HP)
        // ===================================================================
        TowerBuilder::new(TowerKind::Void, "Void", TowerElement::Neutral)
            .description("Tour specialisee contre les ennemis a gros HP")
            .cost(200)
            .target_priority(TargetPriority::HighestHp)
            .skill(
                SkillBuilder::active(0, "Drain Vital")
                    .description("Inflige des degats bases sur les HP max")
                    .icon("D")
                    .cost(80)
                    .range(100.0, 12.0, 5)
                    .attack_speed(0.25, 0.02, 5)
                    .can_change_target()
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::PercentHp(10.0),
                        },
                        vec![("% Degats", ActionUpgradeTarget::Damage, 2.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Neant")
                    .description("Zone de degats en % HP")
                    .icon("N")
                    .cost(120)
                    .range_fixed(80.0)
                    .attack_speed_fixed(0.15)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Area(50.0),
                            damage: DamageType::PercentHp(5.0),
                        },
                        vec![
                            ("% Degats", ActionUpgradeTarget::Damage, 1.0, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 8.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Absorption")
                    .description("Absorbe l'energie pour generer de l'or")
                    .icon("A")
                    .cost(100)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 1.5,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.7, 5)],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // ALARME - Tour utilitaire (notifications)
        // ===================================================================
        TowerBuilder::new(TowerKind::Alarme, "Alarme", TowerElement::Electric)
            .description("Tour utilitaire pour les notifications")
            .cost(75)
            .skill(
                SkillBuilder::passive(0, "Alerte Bouclier")
                    .description("Notifie quand le bouclier est bas")
                    .icon("!")
                    .cost(25)
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Alerte Vague")
                    .description("Notifie au debut de chaque vague")
                    .icon("W")
                    .cost(25)
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Surveillance")
                    .description("Genere un peu d'or")
                    .icon("S")
                    .cost(50)
                    .action_with_upgrades(
                        TowerAction::GoldGen {
                            gold_per_second: 0.5,
                        },
                        vec![("Or/sec", ActionUpgradeTarget::GoldPerSecond, 0.25, 5)],
                    )
                    .build(),
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
