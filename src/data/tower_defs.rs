use crate::game::elemental::TowerElement;
use crate::game::enemy::ElementalStateKind;

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
    Ocean,
    Tesla,
    Seisme,
    Sniper,
    Armurerie,
    Alarme,
    Void,
    Glace,
}

impl TowerKind {
    pub fn all() -> &'static [TowerKind] {
        &[
            TowerKind::Sentinelle,
            TowerKind::Inferno,
            TowerKind::Ocean,
            TowerKind::Glace,
            TowerKind::Tesla,
            TowerKind::Seisme,
            TowerKind::Sniper,
            TowerKind::Armurerie,
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

    // ========================================================================
    // NOUVELLES ACTIONS
    // ========================================================================
    /// Applique un état élémentaire (Brûlé, Trempé, Sismique)
    ApplyElementalState {
        target: EffectTarget,
        state: ElementalStateKind,
        duration: f32,
        strength: f32, // dps pour Burned, ratio pour Soaked, stun_duration pour Seismic
    },

    /// Zone passive autour de la tour (aura) - applique un état aux ennemis proches
    AuraEffect {
        radius: f32,
        state: ElementalStateKind,
        duration: f32,
        strength: f32,
    },

    /// Annihilation: kill instant si 4 états, sinon %HP sur boss
    Annihilate {
        required_states: u8,
        boss_damage_percent: f32,
    },

    /// Dégâts conditionnels basés sur le nombre d'états (zone)
    ConditionalDamage {
        min_states: u8,
        damage_percent: f32,
        radius: f32,
    },

    /// Tir aléatoire sur la carte (bombardement)
    RandomBombard {
        damage: f32,
        radius: f32, // zone d'impact
    },

    /// Vol de vie (dégâts aux ennemis proches + heal joueur)
    LifeSteal {
        radius: f32,
        damage_per_second: f32,
        heal_ratio: f32, // % des dégâts convertis en heal
    },

    /// Applique l'état Froid avec dégâts et zone
    /// Si la cible est Trempée, elle devient Gelée
    ApplyCold {
        target: EffectTarget,
        damage: f32,
        cold_duration: f32,
        freeze_duration: f32, // Durée du gel si combiné avec Trempé
        aoe_radius: f32,      // Rayon d'application aux ennemis proches
    },

    /// Aura de froid (zone passive)
    ColdAura {
        radius: f32,
        cold_duration: f32,
        freeze_duration: f32,
    },

    /// Glaciation: dégâts %HP + Froid à tous les ennemis dans la zone
    Glaciation {
        radius: f32,
        damage_percent: f32,
        cold_duration: f32,
        freeze_duration: f32,
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
    GoldPerSecond,
    // Nouvelles cibles d'upgrade
    StateDuration,
    StateStrength,
    BossDamagePercent,
    AuraRadius,
    BombardDamage,
    BombardRadius,
    LifeStealDps,
    LifeStealRadius,
    HealRatio,
    ConditionalDamagePercent,
    // Glace
    ColdDuration,
    FreezeDuration,
    GlaciationDamagePercent,
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
    // Nouvelles actions résolues
    ApplyElementalState {
        target: EffectTarget,
        state: ElementalStateKind,
        duration: f32,
        strength: f32,
    },
    AuraEffect {
        radius: f32,
        state: ElementalStateKind,
        duration: f32,
        strength: f32,
    },
    Annihilate {
        required_states: u8,
        boss_damage_percent: f32,
    },
    ConditionalDamage {
        min_states: u8,
        damage_percent: f32,
        radius: f32,
    },
    RandomBombard {
        damage: f32,
        radius: f32,
    },
    LifeSteal {
        radius: f32,
        damage_per_second: f32,
        heal_ratio: f32,
    },
    // Glace
    ApplyCold {
        target: EffectTarget,
        damage: f32,
        cold_duration: f32,
        freeze_duration: f32,
        aoe_radius: f32,
    },
    ColdAura {
        radius: f32,
        cold_duration: f32,
        freeze_duration: f32,
    },
    Glaciation {
        radius: f32,
        damage_percent: f32,
        cold_duration: f32,
        freeze_duration: f32,
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
            // Nouvelles actions
            TowerAction::ApplyElementalState {
                target,
                state,
                duration,
                strength,
            } => {
                let mut dur = *duration;
                let mut str = *strength;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::StateDuration => {
                            dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::StateStrength => {
                            str += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ApplyElementalState {
                    target: target.clone(),
                    state: *state,
                    duration: dur,
                    strength: str,
                }
            }
            TowerAction::AuraEffect {
                radius,
                state,
                duration,
                strength,
            } => {
                let mut rad = *radius;
                let mut dur = *duration;
                let mut str = *strength;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::AuraRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::StateDuration => {
                            dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::StateStrength => {
                            str += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::AuraEffect {
                    radius: rad,
                    state: *state,
                    duration: dur,
                    strength: str,
                }
            }
            TowerAction::Annihilate {
                required_states,
                boss_damage_percent,
            } => {
                let mut bdp = *boss_damage_percent;
                for u in &self.upgrades {
                    if u.applies_to == ActionUpgradeTarget::BossDamagePercent {
                        bdp += u.prop.bonus_per_level * u.prop.current_level as f32;
                    }
                }
                ResolvedAction::Annihilate {
                    required_states: *required_states,
                    boss_damage_percent: bdp,
                }
            }
            TowerAction::ConditionalDamage {
                min_states,
                damage_percent,
                radius,
            } => {
                let mut dp = *damage_percent;
                let mut rad = *radius;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::ConditionalDamagePercent => {
                            dp += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::AoeRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ConditionalDamage {
                    min_states: *min_states,
                    damage_percent: dp,
                    radius: rad,
                }
            }
            TowerAction::RandomBombard { damage, radius } => {
                let mut dmg = *damage;
                let mut rad = *radius;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::BombardDamage => {
                            dmg += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::BombardRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::RandomBombard {
                    damage: dmg,
                    radius: rad,
                }
            }
            TowerAction::LifeSteal {
                radius,
                damage_per_second,
                heal_ratio,
            } => {
                let mut rad = *radius;
                let mut dps = *damage_per_second;
                let mut hr = *heal_ratio;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::LifeStealRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::LifeStealDps => {
                            dps += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::HealRatio => {
                            hr += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::LifeSteal {
                    radius: rad,
                    damage_per_second: dps,
                    heal_ratio: hr,
                }
            }
            // Glace
            TowerAction::ApplyCold {
                target,
                damage,
                cold_duration,
                freeze_duration,
                aoe_radius,
            } => {
                let mut dmg = *damage;
                let mut cold_dur = *cold_duration;
                let mut freeze_dur = *freeze_duration;
                let mut aoe = *aoe_radius;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::Damage => {
                            dmg += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::ColdDuration => {
                            cold_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::FreezeDuration => {
                            freeze_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::AoeRadius => {
                            aoe += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ApplyCold {
                    target: target.clone(),
                    damage: dmg,
                    cold_duration: cold_dur,
                    freeze_duration: freeze_dur,
                    aoe_radius: aoe,
                }
            }
            TowerAction::ColdAura {
                radius,
                cold_duration,
                freeze_duration,
            } => {
                let mut rad = *radius;
                let mut cold_dur = *cold_duration;
                let mut freeze_dur = *freeze_duration;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::AuraRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::ColdDuration => {
                            cold_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::FreezeDuration => {
                            freeze_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::ColdAura {
                    radius: rad,
                    cold_duration: cold_dur,
                    freeze_duration: freeze_dur,
                }
            }
            TowerAction::Glaciation {
                radius,
                damage_percent,
                cold_duration,
                freeze_duration,
            } => {
                let mut rad = *radius;
                let mut dmg_pct = *damage_percent;
                let mut cold_dur = *cold_duration;
                let mut freeze_dur = *freeze_duration;
                for u in &self.upgrades {
                    match u.applies_to {
                        ActionUpgradeTarget::AoeRadius => {
                            rad += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::GlaciationDamagePercent => {
                            dmg_pct += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::ColdDuration => {
                            cold_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        ActionUpgradeTarget::FreezeDuration => {
                            freeze_dur += u.prop.bonus_per_level * u.prop.current_level as f32;
                        }
                        _ => {}
                    }
                }
                ResolvedAction::Glaciation {
                    radius: rad,
                    damage_percent: dmg_pct,
                    cold_duration: cold_dur,
                    freeze_duration: freeze_dur,
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
        // Nouvelles actions
        (TowerAction::ApplyElementalState { duration, .. }, ActionUpgradeTarget::StateDuration) => {
            *duration
        }
        (TowerAction::ApplyElementalState { strength, .. }, ActionUpgradeTarget::StateStrength) => {
            *strength
        }
        (TowerAction::AuraEffect { radius, .. }, ActionUpgradeTarget::AuraRadius) => *radius,
        (TowerAction::AuraEffect { duration, .. }, ActionUpgradeTarget::StateDuration) => *duration,
        (TowerAction::AuraEffect { strength, .. }, ActionUpgradeTarget::StateStrength) => *strength,
        (
            TowerAction::Annihilate {
                boss_damage_percent,
                ..
            },
            ActionUpgradeTarget::BossDamagePercent,
        ) => *boss_damage_percent,
        (
            TowerAction::ConditionalDamage { damage_percent, .. },
            ActionUpgradeTarget::ConditionalDamagePercent,
        ) => *damage_percent,
        (TowerAction::ConditionalDamage { radius, .. }, ActionUpgradeTarget::AoeRadius) => *radius,
        (TowerAction::RandomBombard { damage, .. }, ActionUpgradeTarget::BombardDamage) => *damage,
        (TowerAction::RandomBombard { radius, .. }, ActionUpgradeTarget::BombardRadius) => *radius,
        (TowerAction::LifeSteal { radius, .. }, ActionUpgradeTarget::LifeStealRadius) => *radius,
        (
            TowerAction::LifeSteal {
                damage_per_second, ..
            },
            ActionUpgradeTarget::LifeStealDps,
        ) => *damage_per_second,
        (TowerAction::LifeSteal { heal_ratio, .. }, ActionUpgradeTarget::HealRatio) => *heal_ratio,
        // Glace
        (TowerAction::ApplyCold { damage, .. }, ActionUpgradeTarget::Damage) => *damage,
        (TowerAction::ApplyCold { cold_duration, .. }, ActionUpgradeTarget::ColdDuration) => {
            *cold_duration
        }
        (
            TowerAction::ApplyCold {
                freeze_duration, ..
            },
            ActionUpgradeTarget::FreezeDuration,
        ) => *freeze_duration,
        (TowerAction::ApplyCold { aoe_radius, .. }, ActionUpgradeTarget::AoeRadius) => *aoe_radius,
        (TowerAction::ColdAura { radius, .. }, ActionUpgradeTarget::AuraRadius) => *radius,
        (TowerAction::ColdAura { cold_duration, .. }, ActionUpgradeTarget::ColdDuration) => {
            *cold_duration
        }
        (
            TowerAction::ColdAura {
                freeze_duration, ..
            },
            ActionUpgradeTarget::FreezeDuration,
        ) => *freeze_duration,
        (TowerAction::Glaciation { radius, .. }, ActionUpgradeTarget::AoeRadius) => *radius,
        (
            TowerAction::Glaciation { damage_percent, .. },
            ActionUpgradeTarget::GlaciationDamagePercent,
        ) => *damage_percent,
        (TowerAction::Glaciation { cold_duration, .. }, ActionUpgradeTarget::ColdDuration) => {
            *cold_duration
        }
        (
            TowerAction::Glaciation {
                freeze_duration, ..
            },
            ActionUpgradeTarget::FreezeDuration,
        ) => *freeze_duration,
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
        // SENTINELLE - Tour basique polyvalente
        // ===================================================================
        TowerBuilder::new(TowerKind::Sentinelle, "Sentinelle", TowerElement::Neutral)
            .description("Tour basique equilibree avec 3 modes de tir")
            .cost(50)
            .skill(
                SkillBuilder::active(0, "Tir Standard")
                    .description("Attaque equilibree a cible unique")
                    .icon("S")
                    .cost(0) // Gratuit, c'est la skill de base
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
            .skill(
                SkillBuilder::active(1, "Tir Rapide")
                    .description("Cadence elevee mais degats reduits")
                    .icon("R")
                    .cost(60)
                    .range_fixed(100.0)
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
        // INFERNO - Tour de feu (Brûlé)
        // ===================================================================
        TowerBuilder::new(TowerKind::Inferno, "Inferno", TowerElement::Fire)
            .description("Tour de feu qui applique l'etat Brule")
            .cost(80)
            .skill(
                SkillBuilder::active(0, "Flamme Ardente")
                    .description("Cible la plus proche non brulee, applique Brule")
                    .icon("F")
                    .cost(0)
                    .range(120.0, 10.0, 5)
                    .attack_speed(0.8, 0.08, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(8.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 2.0, 5)],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyElementalState {
                            target: EffectTarget::Single,
                            state: ElementalStateKind::Burned,
                            duration: 4.0,
                            strength: 5.0, // 5 dps
                        },
                        vec![
                            ("Duree", ActionUpgradeTarget::StateDuration, 0.5, 5),
                            ("Brulure", ActionUpgradeTarget::StateStrength, 1.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Aura Incendiaire")
                    .description("Zone autour de la tour, applique Brule aux non-brules")
                    .icon("A")
                    .cost(75)
                    .action_with_upgrades(
                        TowerAction::AuraEffect {
                            radius: 100.0,
                            state: ElementalStateKind::Burned,
                            duration: 4.0,
                            strength: 5.0,
                        },
                        vec![
                            ("Rayon", ActionUpgradeTarget::AuraRadius, 15.0, 5),
                            ("Brulure", ActionUpgradeTarget::StateStrength, 1.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Brasier")
                    .description("Genere de l'or grace a la forge")
                    .icon("B")
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
        // OCEAN - Tour des océans (Trempé)
        // ===================================================================
        TowerBuilder::new(TowerKind::Ocean, "Ocean", TowerElement::Water)
            .description("Tour d'eau qui applique l'etat Trempe (ralentissement)")
            .cost(75)
            .skill(
                SkillBuilder::active(0, "Vague Deferlante")
                    .description("Degats eau + applique Trempe")
                    .icon("V")
                    .cost(0)
                    .range(130.0, 10.0, 5)
                    .attack_speed(1.0, 0.1, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(6.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 1.5, 5)],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyElementalState {
                            target: EffectTarget::Single,
                            state: ElementalStateKind::Soaked,
                            duration: 4.0,
                            strength: 0.6, // 40% slow (1.0 - 0.6 = 0.4)
                        },
                        vec![("Duree", ActionUpgradeTarget::StateDuration, 0.5, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Maree Montante")
                    .description("Zone autour de la tour, applique Trempe aux entrants")
                    .icon("M")
                    .cost(80)
                    .action_with_upgrades(
                        TowerAction::AuraEffect {
                            radius: 90.0,
                            state: ElementalStateKind::Soaked,
                            duration: 4.0,
                            strength: 0.6,
                        },
                        vec![("Rayon", ActionUpgradeTarget::AuraRadius, 15.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Courant Marin")
                    .description("Genere de l'or")
                    .icon("C")
                    .cost(90)
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
        // TESLA - Tour électrique, multi-cibles
        // ===================================================================
        TowerBuilder::new(TowerKind::Tesla, "Tesla", TowerElement::Electric)
            .description("Tour electrique a cibles multiples")
            .cost(100)
            .skill(
                SkillBuilder::active(0, "Arc Electrique")
                    .description("Frappe plusieurs ennemis simultanement")
                    .icon("A")
                    .cost(0)
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
        // SEISME - Tour sismique (Sismique = stun on hit)
        // ===================================================================
        TowerBuilder::new(TowerKind::Seisme, "Seisme", TowerElement::Earth)
            .description("Tour de terre qui applique l'etat Sismique")
            .cost(90)
            .skill(
                SkillBuilder::active(0, "Secousse")
                    .description("Degats terre + stun court a la cible")
                    .icon("S")
                    .cost(0)
                    .range(100.0, 8.0, 5)
                    .attack_speed(0.6, 0.05, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(12.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 2.5, 5)],
                    )
                    .action_with_upgrades(
                        TowerAction::ApplyEffect {
                            target: EffectTarget::Single,
                            effect: EffectType::Stun { duration: 0.3 },
                        },
                        vec![("Stun", ActionUpgradeTarget::EffectDuration, 0.1, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Onde Tellurique")
                    .description("Zone (1.5/s), applique Sismique aux entrants")
                    .icon("O")
                    .cost(85)
                    .action_with_upgrades(
                        TowerAction::AuraEffect {
                            radius: 80.0,
                            state: ElementalStateKind::Seismic,
                            duration: 4.0,
                            strength: 0.15, // 0.15s stun per hit
                        },
                        vec![
                            ("Rayon", ActionUpgradeTarget::AuraRadius, 12.0, 5),
                            ("Stun", ActionUpgradeTarget::StateStrength, 0.03, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Resonance")
                    .description("Genere de l'or")
                    .icon("R")
                    .cost(100)
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
                    .cost(0)
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
        // ARMURERIE - Tour de dégâts
        // ===================================================================
        TowerBuilder::new(TowerKind::Armurerie, "Armurerie", TowerElement::Neutral)
            .description("Tour de degats avec plusieurs modes de tir")
            .cost(120)
            .skill(
                SkillBuilder::active(0, "Tir Rafale")
                    .description("Faibles degats, tres haute cadence")
                    .icon("R")
                    .cost(0)
                    .range(100.0, 8.0, 5)
                    .attack_speed(4.0, 0.3, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(3.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 0.5, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(1, "Artillerie Lourde")
                    .description("Lourds degats, haute cadence")
                    .icon("A")
                    .cost(100)
                    .range(150.0, 10.0, 5)
                    .attack_speed(1.5, 0.1, 5)
                    .can_change_target()
                    .action_with_upgrades(
                        TowerAction::ApplyDamage {
                            target: EffectTarget::Single,
                            damage: DamageType::Fixed(20.0),
                        },
                        vec![("Degats", ActionUpgradeTarget::Damage, 4.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::active(2, "Bombardement")
                    .description("Obus aleatoire, tres lourds degats")
                    .icon("B")
                    .cost(150)
                    .range_fixed(300.0) // Grande portée mais position aléatoire
                    .attack_speed_fixed(0.33) // 1 tir / 3s
                    .action_with_upgrades(
                        TowerAction::RandomBombard {
                            damage: 50.0,
                            radius: 60.0,
                        },
                        vec![
                            ("Degats", ActionUpgradeTarget::BombardDamage, 10.0, 5),
                            ("Zone", ActionUpgradeTarget::BombardRadius, 10.0, 5),
                        ],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // VOID - Tour du vide (combinaisons d'états)
        // ===================================================================
        TowerBuilder::new(TowerKind::Void, "Void", TowerElement::Neutral)
            .description("Tour specialisee contre les ennemis avec etats elementaires")
            .cost(250)
            .target_priority(TargetPriority::HighestHp)
            .skill(
                SkillBuilder::active(0, "Annihilation")
                    .description("Kill si 4 etats, boss: 10% HP + supprime etats")
                    .icon("X")
                    .cost(0)
                    .range(100.0, 10.0, 5)
                    .attack_speed(0.3, 0.02, 5)
                    .can_change_target()
                    .action_with_upgrades(
                        TowerAction::Annihilate {
                            required_states: 4,
                            boss_damage_percent: 10.0,
                        },
                        vec![("% Boss", ActionUpgradeTarget::BossDamagePercent, 2.0, 5)],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "The Void")
                    .description("Zone: si 3+ etats, 10% HP")
                    .icon("V")
                    .cost(150)
                    .action_with_upgrades(
                        TowerAction::ConditionalDamage {
                            min_states: 3,
                            damage_percent: 10.0,
                            radius: 120.0,
                        },
                        vec![
                            (
                                "% Degats",
                                ActionUpgradeTarget::ConditionalDamagePercent,
                                2.0,
                                5,
                            ),
                            ("Rayon", ActionUpgradeTarget::AoeRadius, 15.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Absorption")
                    .description("Vole de la vie aux ennemis, heal le joueur")
                    .icon("A")
                    .cost(120)
                    .action_with_upgrades(
                        TowerAction::LifeSteal {
                            radius: 80.0,
                            damage_per_second: 3.0,
                            heal_ratio: 50.0, // 50% des dégâts convertis en heal
                        },
                        vec![
                            ("DPS", ActionUpgradeTarget::LifeStealDps, 1.0, 5),
                            ("Heal%", ActionUpgradeTarget::HealRatio, 10.0, 5),
                        ],
                    )
                    .build(),
            )
            .build(),
        // ===================================================================
        // GLACE - Tour de glace (Froid -> Gelé)
        // ===================================================================
        TowerBuilder::new(TowerKind::Glace, "Glace", TowerElement::Water)
            .description("Tour de glace qui applique Froid et peut geler les ennemis Trempes")
            .cost(85)
            .skill(
                SkillBuilder::active(0, "Souffle Glacial")
                    .description("Faibles degats et applique Froid en zone")
                    .icon("G")
                    .cost(0)
                    .range(110.0, 10.0, 5)
                    .attack_speed(0.9, 0.08, 5)
                    .action_with_upgrades(
                        TowerAction::ApplyCold {
                            target: EffectTarget::Single,
                            damage: 5.0,
                            cold_duration: 4.0,
                            freeze_duration: 2.0, // Si Trempe -> 2s de gel
                            aoe_radius: 50.0,     // Zone d'application du froid
                        },
                        vec![
                            ("Degats", ActionUpgradeTarget::Damage, 1.0, 5),
                            ("Froid", ActionUpgradeTarget::ColdDuration, 0.5, 5),
                            ("Gel", ActionUpgradeTarget::FreezeDuration, 0.3, 5),
                            ("Zone", ActionUpgradeTarget::AoeRadius, 10.0, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(1, "Vent Gele")
                    .description("Aura de froid autour de la tour")
                    .icon("V")
                    .cost(80)
                    .action_with_upgrades(
                        TowerAction::ColdAura {
                            radius: 80.0,
                            cold_duration: 4.0,
                            freeze_duration: 2.0,
                        },
                        vec![
                            ("Rayon", ActionUpgradeTarget::AuraRadius, 12.0, 5),
                            ("Gel", ActionUpgradeTarget::FreezeDuration, 0.3, 5),
                        ],
                    )
                    .build(),
            )
            .skill(
                SkillBuilder::passive(2, "Glaciation")
                    .description("Degats %HP + Froid a tous les ennemis en zone")
                    .icon("!")
                    .cost(150)
                    .action_with_upgrades(
                        TowerAction::Glaciation {
                            radius: 100.0,
                            damage_percent: 3.0, // 3% HP par tick
                            cold_duration: 4.0,
                            freeze_duration: 2.5,
                        },
                        vec![
                            ("Rayon", ActionUpgradeTarget::AoeRadius, 15.0, 5),
                            (
                                "% Degats",
                                ActionUpgradeTarget::GlaciationDamagePercent,
                                0.5,
                                5,
                            ),
                            ("Gel", ActionUpgradeTarget::FreezeDuration, 0.3, 5),
                        ],
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
                    .cost(0)
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
