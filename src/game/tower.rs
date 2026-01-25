use super::Point2D;
use super::elemental::TowerElement;
use crate::data::tower_defs::{
    Property, ResolvedAction, SkillType, TargetPriority, TowerKind, TowerSkillDef, UpgradeableProp,
    get_def,
};

#[derive(Clone, Default)]
pub struct NotificationSettings {
    pub shield_broken: bool,
    pub shield_low: bool,
}

// ============================================================================
// SKILL STATE - État runtime d'une compétence
// ============================================================================

/// État runtime des propriétés upgradables d'une compétence
#[derive(Clone)]
pub struct SkillRuntimeState {
    /// Portée (clonée depuis la définition, peut être upgradée)
    pub range: Option<Property>,
    /// Vitesse d'attaque
    pub attack_speed: Option<Property>,
    /// État des upgrades des actions
    pub action_states: Vec<SkillActionState>,
}

/// État runtime d'une action dans une compétence
#[derive(Clone)]
pub struct SkillActionState {
    pub action_index: usize,
    pub upgrades: Vec<UpgradeableProp>,
}

/// État d'une compétence sur une tourelle placée
#[derive(Clone)]
pub enum SkillState {
    /// Compétence non achetée
    Locked,
    /// Compétence achetée mais inactive (pour les skills Active uniquement)
    Purchased(SkillRuntimeState),
    /// Compétence active (une seule Active skill peut être dans cet état)
    Active(SkillRuntimeState),
}

impl SkillState {
    /// Crée un état runtime à partir d'une définition de skill
    fn create_runtime_state(skill_def: &TowerSkillDef) -> SkillRuntimeState {
        let action_states: Vec<SkillActionState> = skill_def
            .actions
            .iter()
            .enumerate()
            .map(|(i, action_def)| SkillActionState {
                action_index: i,
                upgrades: action_def.upgrades.iter().map(|u| u.prop.clone()).collect(),
            })
            .collect();

        SkillRuntimeState {
            range: skill_def.range.clone(),
            attack_speed: skill_def.attack_speed.clone(),
            action_states,
        }
    }

    pub fn is_purchased(&self) -> bool {
        !matches!(self, SkillState::Locked)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, SkillState::Active(_))
    }

    /// Récupère l'état runtime si acheté
    pub fn runtime_state(&self) -> Option<&SkillRuntimeState> {
        match self {
            SkillState::Locked => None,
            SkillState::Purchased(state) | SkillState::Active(state) => Some(state),
        }
    }

    /// Récupère l'état runtime mutable si acheté
    pub fn runtime_state_mut(&mut self) -> Option<&mut SkillRuntimeState> {
        match self {
            SkillState::Locked => None,
            SkillState::Purchased(state) | SkillState::Active(state) => Some(state),
        }
    }

    /// Retourne la valeur de portée si disponible
    pub fn range_value(&self) -> Option<f32> {
        self.runtime_state()
            .and_then(|s| s.range.as_ref())
            .map(|p| p.value())
    }

    /// Retourne la valeur de vitesse d'attaque si disponible
    pub fn attack_speed_value(&self) -> Option<f32> {
        self.runtime_state()
            .and_then(|s| s.attack_speed.as_ref())
            .map(|p| p.value())
    }
}

// ============================================================================
// SKILL UPGRADE ID - Identifie un upgrade spécifique
// ============================================================================

/// Identifie un upgrade spécifique sur une compétence
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkillUpgradeId {
    Range,
    AttackSpeed,
    Action {
        action_idx: usize,
        upgrade_idx: usize,
    },
}

// ============================================================================
// TOWER - Structure principale de la tourelle
// ============================================================================

#[derive(Clone)]
pub struct Tower {
    pub id: usize,
    pub position: Point2D,
    pub kind: TowerKind,
    pub element: TowerElement,
    pub name: &'static str,
    /// État des compétences
    pub skills: Vec<SkillState>,
    /// Index de la compétence Active actuellement sélectionnée
    pub active_skill_index: Option<usize>,
    /// Cooldown d'attaque partagé
    pub attack_cooldown: f32,
    /// Coût de base de la tourelle
    pub base_cost: u32,
    /// Rayon visuel
    pub radius: f32,
    /// Accumulateur d'or pour la génération passive
    pub gold_accumulator: f32,
    /// Paramètres de notification (pour la tour Alarme)
    pub notification_settings: Option<NotificationSettings>,
    /// Priorité de ciblage
    pub target_priority: TargetPriority,
    /// Cooldown pour les effets passifs à tick rate (ex: Glaciation)
    pub passive_cooldowns: Vec<f32>,
}

impl Tower {
    pub fn from_def(id: usize, kind: TowerKind, position: Point2D) -> Self {
        let def = get_def(kind);

        // Première compétence débloquée et active par défaut, les autres verrouillées
        let mut skills: Vec<SkillState> = Vec::with_capacity(def.skills.len());
        for (idx, _skill_def) in def.skills.iter().enumerate() {
            if idx == 0 {
                skills.push(SkillState::Active(SkillState::create_runtime_state(
                    &def.skills[0],
                )));
            } else {
                skills.push(SkillState::Locked);
            }
        }

        // Déterminer si la première skill est Active (attaque) ou Passive
        let active_skill_index = if def.skills[0].skill_type == SkillType::Active {
            Some(0)
        } else {
            None
        };

        // Alarme tower has notification settings (disabled by default)
        let notification_settings = if kind == TowerKind::Alarme {
            Some(NotificationSettings {
                shield_broken: false,
                shield_low: false,
            })
        } else {
            None
        };

        // Initialise les cooldowns passifs (un par skill)
        let passive_cooldowns = vec![0.0; def.skills.len()];

        Self {
            id,
            position,
            kind,
            element: def.element,
            name: def.name,
            skills,
            active_skill_index,
            attack_cooldown: 0.0,
            base_cost: def.base_cost,
            radius: 14.0,
            gold_accumulator: 0.0,
            notification_settings,
            target_priority: def.default_target_priority,
            passive_cooldowns,
        }
    }

    // ========================================================================
    // SKILL PURCHASE & ACTIVATION
    // ========================================================================

    /// Achète une compétence (retourne false si impossible)
    pub fn purchase_skill(&mut self, skill_idx: usize, gold: &mut u32) -> bool {
        if skill_idx >= 3 {
            return false;
        }

        // Déjà achetée ?
        if self.skills[skill_idx].is_purchased() {
            return false;
        }

        let def = get_def(self.kind);
        let skill_def = &def.skills[skill_idx];

        // Assez d'or ?
        if *gold < skill_def.purchase_cost {
            return false;
        }

        // Déduire l'or
        *gold -= skill_def.purchase_cost;

        // Créer l'état runtime
        let runtime_state = SkillState::create_runtime_state(skill_def);

        // Si c'est la première skill achetée, l'activer automatiquement
        // Sinon, la mettre en état Purchased
        let has_active_skill = self.active_skill_index.is_some();
        if !has_active_skill {
            self.skills[skill_idx] = SkillState::Active(runtime_state);
            self.active_skill_index = Some(skill_idx);
        } else {
            self.skills[skill_idx] = SkillState::Purchased(runtime_state);
        }

        true
    }

    /// Retourne le coût d'achat d'une compétence (None si déjà achetée)
    pub fn skill_purchase_cost(&self, skill_idx: usize) -> Option<u32> {
        if skill_idx >= 3 || self.skills[skill_idx].is_purchased() {
            return None;
        }
        let def = get_def(self.kind);
        Some(def.skills[skill_idx].purchase_cost)
    }

    /// Active une compétence (désactive la précédente)
    /// Fonctionne pour tous les types de skills (Active et Passive)
    pub fn activate_skill(&mut self, skill_idx: usize) -> bool {
        if skill_idx >= 3 {
            return false;
        }

        // Doit être achetée
        if !self.skills[skill_idx].is_purchased() {
            return false;
        }

        // Déjà active ?
        if self.skills[skill_idx].is_active() {
            return true;
        }

        // Désactiver la skill actuelle (si elle existe)
        if let Some(current_idx) = self.active_skill_index {
            if current_idx != skill_idx {
                // Passer de Active à Purchased
                if let SkillState::Active(state) =
                    std::mem::replace(&mut self.skills[current_idx], SkillState::Locked)
                {
                    self.skills[current_idx] = SkillState::Purchased(state);
                }
            }
        }

        // Activer la nouvelle skill
        if let SkillState::Purchased(state) =
            std::mem::replace(&mut self.skills[skill_idx], SkillState::Locked)
        {
            self.skills[skill_idx] = SkillState::Active(state);
        }

        self.active_skill_index = Some(skill_idx);
        true
    }

    // ========================================================================
    // COMBAT PROPERTIES
    // ========================================================================

    /// Retourne la portée de la skill active (0.0 si aucune)
    pub fn attack_range(&self) -> f32 {
        self.active_skill_index
            .and_then(|idx| self.skills.get(idx))
            .and_then(|s| s.range_value())
            .unwrap_or(0.0)
    }

    /// Retourne la vitesse d'attaque de la skill active (0.0 si aucune)
    pub fn attack_speed_value(&self) -> f32 {
        self.active_skill_index
            .and_then(|idx| self.skills.get(idx))
            .and_then(|s| s.attack_speed_value())
            .unwrap_or(0.0)
    }

    /// Retourne la taille du projectile de la skill active
    pub fn projectile_size(&self) -> f32 {
        self.active_skill_index
            .map(|idx| {
                let def = get_def(self.kind);
                def.skills[idx].projectile_size
            })
            .unwrap_or(4.0)
    }

    /// Vérifie si la tourelle peut attaquer (a une skill active avec range > 0)
    pub fn can_attack(&self) -> bool {
        self.attack_range() > 0.0 && self.attack_speed_value() > 0.0
    }

    /// Vérifie si la skill active permet de changer la priorité de ciblage
    pub fn can_change_target(&self) -> bool {
        self.active_skill_index
            .map(|idx| {
                let def = get_def(self.kind);
                def.skills[idx].can_change_target
            })
            .unwrap_or(false)
    }

    /// Retourne les actions résolues de la skill active
    pub fn resolved_actions(&self) -> Vec<ResolvedAction> {
        let Some(active_idx) = self.active_skill_index else {
            return Vec::new();
        };

        let Some(skill_state) = self.skills.get(active_idx) else {
            return Vec::new();
        };

        let Some(runtime) = skill_state.runtime_state() else {
            return Vec::new();
        };

        let mut def = get_def(self.kind);
        let skill_def = &mut def.skills[active_idx];

        // Appliquer les niveaux d'upgrade actuels
        for (action_idx, action_state) in runtime.action_states.iter().enumerate() {
            if let Some(action_def) = skill_def.actions.get_mut(action_idx) {
                for (upgrade_idx, upgrade_prop) in action_state.upgrades.iter().enumerate() {
                    if let Some(action_upgrade) = action_def.upgrades.get_mut(upgrade_idx) {
                        action_upgrade.prop.current_level = upgrade_prop.current_level;
                    }
                }
            }
        }

        skill_def.actions.iter().map(|a| a.resolve()).collect()
    }

    /// Retourne les zones d'aura actives (rayon, couleur HSL)
    /// Utilisé pour le rendu visuel des auras
    pub fn active_aura_zones(&self) -> Vec<(f32, (f32, f32, f32))> {
        use crate::data::tower_defs::ResolvedAction;

        let mut zones = Vec::new();
        let color = self.element.color();
        let base_color = (color.h, color.s, color.l);

        // Vérifier les actions de la skill active
        for action in self.resolved_actions() {
            match action {
                ResolvedAction::AuraEffect { radius, .. } => {
                    zones.push((radius, base_color));
                }
                ResolvedAction::ColdAura { radius, .. } => {
                    // Bleu clair pour le froid
                    zones.push((radius, (0.55, 0.7, 0.6)));
                }
                ResolvedAction::Glaciation { radius, .. } => {
                    // Cyan pour glaciation
                    zones.push((radius, (0.52, 0.8, 0.5)));
                }
                ResolvedAction::ConditionalDamage { radius, .. } => {
                    // Violet pour le void
                    zones.push((radius, (0.75, 0.6, 0.4)));
                }
                ResolvedAction::LifeSteal { radius, .. } => {
                    // Rouge sombre pour lifesteal
                    zones.push((radius, (0.0, 0.7, 0.3)));
                }
                ResolvedAction::PassiveBurnAura { radius, .. } => {
                    // Orange/rouge pour le feu
                    zones.push((radius, (0.05, 0.9, 0.5)));
                }
                _ => {}
            }
        }

        // Vérifier aussi les skills passives
        for action in self.passive_effects() {
            match action {
                ResolvedAction::AuraEffect { radius, .. } => {
                    zones.push((radius, base_color));
                }
                ResolvedAction::ColdAura { radius, .. } => {
                    zones.push((radius, (0.55, 0.7, 0.6)));
                }
                ResolvedAction::Glaciation { radius, .. } => {
                    zones.push((radius, (0.52, 0.8, 0.5)));
                }
                ResolvedAction::ConditionalDamage { radius, .. } => {
                    zones.push((radius, (0.75, 0.6, 0.4)));
                }
                ResolvedAction::LifeSteal { radius, .. } => {
                    zones.push((radius, (0.0, 0.7, 0.3)));
                }
                ResolvedAction::PassiveBurnAura { radius, .. } => {
                    // Orange/rouge pour le feu
                    zones.push((radius, (0.05, 0.9, 0.5)));
                }
                _ => {}
            }
        }

        zones
    }

    /// Retourne les effets passifs (actions de toutes les skills passives achetées)
    pub fn passive_effects(&self) -> Vec<ResolvedAction> {
        let def = get_def(self.kind);
        let mut results = Vec::new();

        for (idx, skill_state) in self.skills.iter().enumerate() {
            let skill_def = &def.skills[idx];

            // Seulement les skills passives et actives (achetées)
            if skill_def.skill_type != SkillType::Passive {
                continue;
            }

            let Some(runtime) = skill_state.runtime_state() else {
                continue;
            };

            let mut skill_def_clone = skill_def.clone();

            // Appliquer les niveaux d'upgrade
            for (action_idx, action_state) in runtime.action_states.iter().enumerate() {
                if let Some(action_def) = skill_def_clone.actions.get_mut(action_idx) {
                    for (upgrade_idx, upgrade_prop) in action_state.upgrades.iter().enumerate() {
                        if let Some(action_upgrade) = action_def.upgrades.get_mut(upgrade_idx) {
                            action_upgrade.prop.current_level = upgrade_prop.current_level;
                        }
                    }
                }
            }

            for action_def in &skill_def_clone.actions {
                results.push(action_def.resolve());
            }
        }

        results
    }

    // ========================================================================
    // UPGRADES
    // ========================================================================

    /// Retourne les upgrades disponibles pour la skill active
    pub fn get_active_skill_upgrades(
        &self,
    ) -> Vec<(SkillUpgradeId, &'static str, &UpgradeableProp)> {
        let Some(active_idx) = self.active_skill_index else {
            return Vec::new();
        };

        self.get_skill_upgrades(active_idx)
    }

    /// Retourne les upgrades de toutes les skills achetées, groupées par skill
    /// Retourne: Vec<(skill_idx, skill_name, is_active, upgrades)>
    pub fn get_all_purchased_skill_upgrades(
        &self,
    ) -> Vec<(
        usize,
        &'static str,
        bool,
        Vec<(SkillUpgradeId, &'static str, &UpgradeableProp)>,
    )> {
        let def = get_def(self.kind);
        let mut result = Vec::new();

        for (skill_idx, skill_state) in self.skills.iter().enumerate() {
            // Seulement les skills achetées (Purchased ou Active)
            if !skill_state.is_purchased() {
                continue;
            }

            let skill_def = &def.skills[skill_idx];
            let is_active = skill_state.is_active();
            let upgrades = self.get_skill_upgrades(skill_idx);

            // Inclure même si pas d'upgrades pour montrer le nom de la skill
            result.push((skill_idx, skill_def.name, is_active, upgrades));
        }

        result
    }

    /// Retourne les upgrades pour une skill spécifique
    pub fn get_skill_upgrades(
        &self,
        skill_idx: usize,
    ) -> Vec<(SkillUpgradeId, &'static str, &UpgradeableProp)> {
        if skill_idx >= 3 {
            return Vec::new();
        }

        let Some(runtime) = self.skills[skill_idx].runtime_state() else {
            return Vec::new();
        };

        let def = get_def(self.kind);
        let skill_def = &def.skills[skill_idx];
        let mut result = Vec::new();

        // Range upgrade (si upgradable)
        if let Some(Property::Upgradable(ref prop)) = runtime.range {
            if prop.max_level > 0 {
                result.push((SkillUpgradeId::Range, "Portee", prop));
            }
        }

        // Attack speed upgrade (si upgradable)
        if let Some(Property::Upgradable(ref prop)) = runtime.attack_speed {
            if prop.max_level > 0 {
                result.push((SkillUpgradeId::AttackSpeed, "Vitesse", prop));
            }
        }

        // Action upgrades
        for (action_idx, action_state) in runtime.action_states.iter().enumerate() {
            if let Some(action_def) = skill_def.actions.get(action_idx) {
                for (upgrade_idx, upgrade_prop) in action_state.upgrades.iter().enumerate() {
                    if let Some(action_upgrade) = action_def.upgrades.get(upgrade_idx) {
                        result.push((
                            SkillUpgradeId::Action {
                                action_idx,
                                upgrade_idx,
                            },
                            action_upgrade.name,
                            upgrade_prop,
                        ));
                    }
                }
            }
        }

        result
    }

    /// Applique un upgrade à la skill active
    pub fn apply_skill_upgrade(&mut self, upgrade_id: SkillUpgradeId) -> bool {
        let Some(active_idx) = self.active_skill_index else {
            return false;
        };

        self.apply_upgrade_to_skill(active_idx, upgrade_id)
    }

    /// Applique un upgrade à une skill spécifique
    pub fn apply_upgrade_to_skill(&mut self, skill_idx: usize, upgrade_id: SkillUpgradeId) -> bool {
        if skill_idx >= 3 {
            return false;
        }

        let Some(runtime) = self.skills[skill_idx].runtime_state_mut() else {
            return false;
        };

        match upgrade_id {
            SkillUpgradeId::Range => {
                if let Some(Property::Upgradable(ref mut prop)) = runtime.range {
                    if prop.can_upgrade() {
                        prop.upgrade();
                        return true;
                    }
                }
                false
            }
            SkillUpgradeId::AttackSpeed => {
                if let Some(Property::Upgradable(ref mut prop)) = runtime.attack_speed {
                    if prop.can_upgrade() {
                        prop.upgrade();
                        return true;
                    }
                }
                false
            }
            SkillUpgradeId::Action {
                action_idx,
                upgrade_idx,
            } => {
                if let Some(action_state) = runtime.action_states.get_mut(action_idx) {
                    if let Some(prop) = action_state.upgrades.get_mut(upgrade_idx) {
                        if prop.can_upgrade() {
                            prop.upgrade();
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    /// Retourne le coût d'un upgrade sur la skill active
    pub fn upgrade_cost(&self, upgrade_id: SkillUpgradeId) -> Option<u32> {
        let active_idx = self.active_skill_index?;
        self.skill_upgrade_cost(active_idx, upgrade_id)
    }

    /// Alias pour skill_upgrade_cost
    pub fn upgrade_cost_for_skill(
        &self,
        skill_idx: usize,
        upgrade_id: SkillUpgradeId,
    ) -> Option<u32> {
        self.skill_upgrade_cost(skill_idx, upgrade_id)
    }

    /// Retourne le coût d'un upgrade sur une skill spécifique
    pub fn skill_upgrade_cost(&self, skill_idx: usize, upgrade_id: SkillUpgradeId) -> Option<u32> {
        if skill_idx >= 3 {
            return None;
        }

        let runtime = self.skills[skill_idx].runtime_state()?;

        match upgrade_id {
            SkillUpgradeId::Range => {
                if let Some(Property::Upgradable(ref prop)) = runtime.range {
                    if prop.can_upgrade() {
                        return Some(prop.cost());
                    }
                }
                None
            }
            SkillUpgradeId::AttackSpeed => {
                if let Some(Property::Upgradable(ref prop)) = runtime.attack_speed {
                    if prop.can_upgrade() {
                        return Some(prop.cost());
                    }
                }
                None
            }
            SkillUpgradeId::Action {
                action_idx,
                upgrade_idx,
            } => runtime
                .action_states
                .get(action_idx)
                .and_then(|a| a.upgrades.get(upgrade_idx))
                .and_then(|p| {
                    if p.can_upgrade() {
                        Some(p.cost())
                    } else {
                        None
                    }
                }),
        }
    }

    // ========================================================================
    // UTILITY
    // ========================================================================

    /// Change la priorité de ciblage (cycle vers la suivante)
    pub fn cycle_target_priority(&mut self) {
        self.target_priority = self.target_priority.next();
    }

    /// Calcule le niveau total de la tourelle
    pub fn level(&self) -> u32 {
        let mut level = 1u32;

        // Compter les skills achetées et leurs upgrades
        for skill_state in &self.skills {
            if let Some(runtime) = skill_state.runtime_state() {
                level += 1; // +1 pour chaque skill achetée

                // Compter les upgrades de range
                if let Some(Property::Upgradable(ref prop)) = runtime.range {
                    level += prop.current_level;
                }

                // Compter les upgrades d'attack speed
                if let Some(Property::Upgradable(ref prop)) = runtime.attack_speed {
                    level += prop.current_level;
                }

                // Compter les upgrades d'actions
                for action_state in &runtime.action_states {
                    for upgrade in &action_state.upgrades {
                        level += upgrade.current_level;
                    }
                }
            }
        }

        level
    }

    /// Calcule la valeur de revente
    pub fn sell_value(&self) -> u32 {
        let mut total_cost = self.base_cost;

        // Ajouter le coût des skills achetées
        let def = get_def(self.kind);
        for (idx, skill_state) in self.skills.iter().enumerate() {
            if skill_state.is_purchased() {
                total_cost += def.skills[idx].purchase_cost;
            }
        }

        // 50% de la valeur totale
        total_cost / 2
    }
}
