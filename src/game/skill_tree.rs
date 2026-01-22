#[allow(dead_code)]
use super::elemental::TowerElement;

#[derive(Clone)]
pub enum SkillEffect {
    DamageBonus(f32),
    RangeBonus(f32),
    AttackSpeedBonus(f32),
    AoEUnlock,
    AoERadiusBonus(f32),
    ElementChange(TowerElement),
    SlowOnHit(f32),
    CritChance(f32),
    CritDamage(f32),
    GoldBonus(f32),
    ChainLightning(u32),
    LifeSteal(f32),
}

#[derive(Clone)]
pub struct SkillNode {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub effect: SkillEffect,
    pub cost: u32,
    pub unlocked: bool,
    pub prerequisites: Vec<usize>,
}

#[derive(Clone)]
pub struct SkillTree {
    pub nodes: Vec<SkillNode>,
}

impl SkillTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn can_unlock(&self, node_id: usize, gold: u32) -> bool {
        if let Some(node) = self.nodes.get(node_id) {
            if node.unlocked {
                return false;
            }
            if gold < node.cost {
                return false;
            }
            node.prerequisites
                .iter()
                .all(|&prereq| self.nodes.get(prereq).map_or(false, |n| n.unlocked))
        } else {
            false
        }
    }

    pub fn unlock(&mut self, node_id: usize) -> Option<SkillEffect> {
        if let Some(node) = self.nodes.get_mut(node_id) {
            if !node.unlocked {
                node.unlocked = true;
                return Some(node.effect.clone());
            }
        }
        None
    }
}
