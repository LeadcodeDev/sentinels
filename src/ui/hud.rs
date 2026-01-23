use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::divider::Divider;
use gpui_component::progress::Progress;
use gpui_component::{Disableable, Sizable, Size, h_flex, v_flex};

use crate::data::tower_defs::get_def;
use crate::game::elemental::TowerElement;
use crate::game::tower::TowerUpgradeId;
use crate::game::{GamePhase, GameState};
use crate::screens::play::PlayScreen;

const SIDEBAR_WIDTH: f32 = 200.0;

pub fn sidebar_width() -> f32 {
    SIDEBAR_WIDTH
}

pub fn render_sidebar(game: &GameState, cx: &mut Context<PlayScreen>) -> impl IntoElement + use<> {
    let hp = game.player.hp;
    let max_hp = game.player.max_hp;
    let gold = game.economy.gold;
    let pepites = game.economy.pepites;
    let wave = game.economy.wave_number;
    let score = game.economy.score;
    let phase = game.phase;
    let tower_count = game.towers.len();
    let max_towers = game.max_towers;
    let player_gold = game.economy.gold;
    let shield = game.shield.clone();

    let selected_section = selected_tower_section(game, cx);

    v_flex()
        .w(px(SIDEBAR_WIDTH))
        .h_full()
        .flex_shrink_0()
        .bg(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.08,
            a: 0.95,
        })
        .border_l_1()
        .border_color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.2,
            a: 1.0,
        })
        .p_3()
        .gap_3()
        // Stats section
        .child(stats_section(
            hp, max_hp, gold, pepites, wave, score, &shield,
        ))
        // Tower grid section
        .child(tower_grid_section(player_gold, tower_count, max_towers, cx))
        // Selected tower section (scrollable)
        .when_some(selected_section, |this, section| this.child(section))
        // Bottom: wave button or game over
        .child(
            v_flex()
                .flex_shrink_0()
                .when(phase == GamePhase::Preparing, |this| {
                    this.child(
                        Button::new("start_wave")
                            .primary()
                            .label("Lancer la vague")
                            .on_click(cx.listener(|screen, _, _window, _cx| {
                                screen.game_state.start_wave();
                            })),
                    )
                }),
        )
}

fn stat_row(label: &'static str, value: String, color: impl Into<Hsla>) -> impl IntoElement {
    h_flex()
        .items_center()
        .justify_between()
        .child(div().text_xs().text_color(rgb(0xaaaaaa)).child(label))
        .child(div().text_sm().text_color(color.into()).child(value))
}

fn stats_section(
    hp: f32,
    max_hp: f32,
    gold: u32,
    pepites: u32,
    wave: u32,
    score: u32,
    shield: &crate::game::Shield,
) -> impl IntoElement {
    let shield_unlocked = shield.is_unlocked();
    let shield_active = shield.active;
    let shield_hp = shield.hp;
    let shield_max_hp = shield.max_hp;
    let shield_regen_timer = shield.regen_timer;

    v_flex()
        .gap_2()
        // HP bar using Progress component
        .child(
            v_flex()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("PV"))
                .child(
                    Progress::new()
                        .value((hp / max_hp) * 100.0)
                        .bg(rgb(0xff4444)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xff4444))
                        .child(format!("{:.0}/{:.0}", hp, max_hp)),
                ),
        )
        // Shield bar (only if unlocked)
        .when(shield_unlocked, |this| {
            this.child(
                v_flex()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Bouclier"))
                    .child(
                        Progress::new()
                            .value(if shield_active {
                                (shield_hp / shield_max_hp) * 100.0
                            } else {
                                0.0
                            })
                            .bg(rgb(0x44ccdd)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x44ccdd))
                            .child(if shield_active {
                                format!("{:.0}/{:.0}", shield_hp, shield_max_hp)
                            } else {
                                format!("Regen {:.0}s", shield_regen_timer)
                            }),
                    ),
            )
        })
        // Stats using consistent stat_row helper
        .child(stat_row("Or", format!("{}", gold), rgb(0xffd700)))
        .child(stat_row("Pepites", format!("{}", pepites), rgb(0xcc66ff)))
        .child(stat_row("Vague", format!("{}", wave), rgb(0xffffff)))
        .child(stat_row("Score", format!("{}", score), rgb(0xffffff)))
}

fn tower_grid_section(
    gold: u32,
    tower_count: usize,
    max_towers: u32,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let slots_full = tower_count >= max_towers as usize;

    let neutral = tower_icon(TowerElement::Neutral, gold, slots_full, cx);
    let fire = tower_icon(TowerElement::Fire, gold, slots_full, cx);
    let water = tower_icon(TowerElement::Water, gold, slots_full, cx);
    let electric = tower_icon(TowerElement::Electric, gold, slots_full, cx);
    let earth = tower_icon(TowerElement::Earth, gold, slots_full, cx);

    v_flex()
        .gap_2()
        .child(Divider::horizontal().color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.25,
            a: 1.0,
        }))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Tours"))
                .child(
                    div()
                        .text_xs()
                        .text_color(if slots_full {
                            rgb(0xff6666)
                        } else {
                            rgb(0x888888)
                        })
                        .child(format!("{}/{}", tower_count, max_towers)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(neutral)
                .child(fire)
                .child(water)
                .child(electric)
                .child(earth),
        )
}

fn selected_tower_section(
    game: &GameState,
    cx: &mut Context<PlayScreen>,
) -> Option<impl IntoElement + use<>> {
    let tower_idx = game.selected_tower?;
    let tower = game.towers.get(tower_idx)?;

    let color = tower.element.color();
    let name = tower.name;
    let range = tower.attack_range();
    let speed = tower.attack_speed_value();
    let sell_value = tower.sell_value();
    let gold = game.economy.gold;

    // Collect upgrades
    let upgrades: Vec<(TowerUpgradeId, &'static str, u32, u32, u32, f32, bool)> = tower
        .get_upgrades()
        .into_iter()
        .map(|(id, name, prop)| {
            let can_afford = prop.can_upgrade() && gold >= prop.cost();
            (
                id,
                name,
                prop.current_level,
                prop.max_level,
                prop.cost(),
                prop.bonus_per_level,
                can_afford,
            )
        })
        .collect();

    let mut upgrade_elements: Vec<AnyElement> = Vec::new();
    for (upgrade_id, uname, ulevel, max_level, cost, bonus, can_afford) in &upgrades {
        let is_maxed = *ulevel >= *max_level;
        let bonus_str = if bonus.fract() == 0.0 {
            format!("{:.0}", bonus)
        } else {
            format!("{:.1}", bonus)
        };
        let label = if is_maxed {
            format!("{} MAX", uname)
        } else {
            format!("{} +{} ({}g)", uname, bonus_str, cost)
        };
        let id_str = SharedString::from(format!("sidebar_upgrade_{:?}", upgrade_id));
        let uid = *upgrade_id;

        let row = h_flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("{}/{}", ulevel, max_level)),
            )
            .child(
                Button::new(id_str)
                    .label(label)
                    .compact()
                    .with_size(Size::Small)
                    .disabled(is_maxed || !can_afford)
                    .on_click(cx.listener(move |screen, _, _window, _cx| {
                        if let Some(idx) = screen.game_state.selected_tower {
                            screen.game_state.upgrade_tower(idx, uid);
                        }
                    })),
            );
        upgrade_elements.push(row.into_any_element());
    }

    let sell_btn = Button::new("sidebar_sell_tower")
        .danger()
        .label(format!("Vendre ({}g)", sell_value))
        .compact()
        .with_size(Size::Small)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if let Some(idx) = screen.game_state.selected_tower {
                screen.game_state.sell_tower(idx);
            }
        }));

    Some(
        div()
            .id("selected_tower_panel")
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .pt_2()
            .child(Divider::horizontal().color(Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }))
            // Header
            .child(div().text_sm().text_color(color).child(name))
            // Stats
            .child(
                v_flex()
                    .gap_1()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("Portee: {:.0}", range))
                    .child(format!("Vitesse: {:.2}/s", speed)),
            )
            // Upgrades header
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xaaaaaa))
                    .child("Ameliorations"),
            )
            .children(upgrade_elements)
            // Sell
            .child(sell_btn),
    )
}

fn tower_icon(
    element: TowerElement,
    gold: u32,
    slots_full: bool,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let def = get_def(element);
    let cost = def.base_cost;
    let can_afford = gold >= cost && !slots_full;
    let color = element.color();
    let name = def.name;

    let bg_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.2 } else { 0.05 },
    };
    let border_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.8 } else { 0.3 },
    };
    let hover_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.35 } else { 0.05 },
    };
    let fg_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 1.0 } else { 0.4 },
    };

    Button::new(SharedString::from(format!("tower_icon_{:?}", element)))
        .custom(
            ButtonCustomVariant::new(cx)
                .color(bg_color)
                .foreground(fg_color)
                .border(border_color)
                .hover(hover_color)
                .active(hover_color),
        )
        .label("\u{25C6}")
        .disabled(!can_afford)
        .tooltip(SharedString::from(format!("{} - {} or", name, cost)))
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if can_afford {
                screen.game_state.placement_mode = Some(element);
            }
        }))
}
