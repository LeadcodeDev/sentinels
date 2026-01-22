use gpui::Hsla;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TowerElement {
    Neutral,
    Fire,
    Water,
    Electric,
    Earth,
}

impl TowerElement {
    pub fn color(&self) -> Hsla {
        match self {
            TowerElement::Neutral => Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.7,
                a: 1.0,
            },
            TowerElement::Fire => Hsla {
                h: 0.02,
                s: 0.9,
                l: 0.55,
                a: 1.0,
            },
            TowerElement::Water => Hsla {
                h: 0.58,
                s: 0.8,
                l: 0.55,
                a: 1.0,
            },
            TowerElement::Electric => Hsla {
                h: 0.14,
                s: 0.9,
                l: 0.6,
                a: 1.0,
            },
            TowerElement::Earth => Hsla {
                h: 0.08,
                s: 0.6,
                l: 0.4,
                a: 1.0,
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TowerElement::Neutral => "Neutre",
            TowerElement::Fire => "Feu",
            TowerElement::Water => "Eau",
            TowerElement::Electric => "Electrique",
            TowerElement::Earth => "Terre",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ElementalReaction {
    Steam,
    Overload,
    Magma,
    Conductor,
    Erosion,
    Magnetic,
}

impl ElementalReaction {
    pub fn from_elements(a: TowerElement, b: TowerElement) -> Option<Self> {
        match (a, b) {
            (TowerElement::Fire, TowerElement::Water)
            | (TowerElement::Water, TowerElement::Fire) => Some(Self::Steam),
            (TowerElement::Fire, TowerElement::Electric)
            | (TowerElement::Electric, TowerElement::Fire) => Some(Self::Overload),
            (TowerElement::Fire, TowerElement::Earth)
            | (TowerElement::Earth, TowerElement::Fire) => Some(Self::Magma),
            (TowerElement::Water, TowerElement::Electric)
            | (TowerElement::Electric, TowerElement::Water) => Some(Self::Conductor),
            (TowerElement::Water, TowerElement::Earth)
            | (TowerElement::Earth, TowerElement::Water) => Some(Self::Erosion),
            (TowerElement::Electric, TowerElement::Earth)
            | (TowerElement::Earth, TowerElement::Electric) => Some(Self::Magnetic),
            _ => None,
        }
    }
}
