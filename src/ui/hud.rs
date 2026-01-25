use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::divider::Divider;
use gpui_component::progress::Progress;
use gpui_component::{Disableable, Sizable, Size, h_flex, v_flex};

use crate::data::tower_defs::{SkillType, TowerKind, get_def};
use crate::game::tower::SkillState;
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

    let mut tower_icons: Vec<AnyElement> = Vec::new();
    for kind in TowerKind::all() {
        tower_icons.push(tower_icon(*kind, gold, slots_full, cx).into_any_element());
    }

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
        .child(div().flex().flex_wrap().gap_2().children(tower_icons))
}

fn selected_tower_section(
    game: &GameState,
    cx: &mut Context<PlayScreen>,
) -> Option<impl IntoElement + use<>> {
    let tower_idx = game.selected_tower?;
    let tower = game.towers.get(tower_idx)?;

    let color = tower.element.color();
    let name = tower.name;
    let sell_value = tower.sell_value();
    let gold = game.economy.gold;
    let has_notification_settings = tower.notification_settings.is_some();

    // Build skill icons row
    let def = get_def(tower.kind);
    let skill_icons: Vec<AnyElement> = (0..3)
        .map(|skill_idx| {
            let skill_def = &def.skills[skill_idx];
            let skill_state = &tower.skills[skill_idx];
            let skill_icon = skill_def.icon;
            let skill_name = skill_def.name;
            let skill_type = skill_def.skill_type;
            let purchase_cost = skill_def.purchase_cost;

            // Determine button state and color
            let (bg_alpha, border_alpha, is_disabled, tooltip_text) = match skill_state {
                SkillState::Locked => {
                    let can_afford = gold >= purchase_cost;
                    (
                        if can_afford { 0.1 } else { 0.05 },
                        if can_afford { 0.4 } else { 0.2 },
                        !can_afford,
                        format!("{} - {}g", skill_name, purchase_cost),
                    )
                }
                SkillState::Purchased(_) => (
                    0.2,
                    0.6,
                    false,
                    format!("{} (cliquer pour activer)", skill_name),
                ),
                SkillState::Active(_) => (0.4, 1.0, false, format!("{} (active)", skill_name)),
            };

            // Color based on skill type
            let base_color = match skill_type {
                SkillType::Active => rgb(0x44aaff),  // Blue for active skills
                SkillType::Passive => rgb(0xffaa44), // Orange for passive skills
            };
            let base_hsla: Hsla = base_color.into();

            let bg_color = Hsla {
                a: bg_alpha,
                ..base_hsla
            };
            let border_color = Hsla {
                a: border_alpha,
                ..base_hsla
            };
            let hover_color = Hsla {
                a: bg_alpha + 0.15,
                ..base_hsla
            };
            let fg_color = Hsla {
                a: if matches!(skill_state, SkillState::Locked) && gold < purchase_cost {
                    0.4
                } else {
                    1.0
                },
                ..base_hsla
            };

            // Highlight active skill with a ring
            let is_active = matches!(skill_state, SkillState::Active(_));

            Button::new(SharedString::from(format!(
                "skill_icon_{}_{}",
                tower_idx, skill_idx
            )))
            .custom(
                ButtonCustomVariant::new(cx)
                    .color(bg_color)
                    .foreground(fg_color)
                    .border(border_color)
                    .hover(hover_color)
                    .active(hover_color),
            )
            .label(skill_icon)
            .disabled(is_disabled)
            .tooltip(SharedString::from(tooltip_text))
            .when(is_active, |btn| {
                btn.custom(
                    ButtonCustomVariant::new(cx)
                        .color(Hsla {
                            a: 0.5,
                            ..base_hsla
                        })
                        .foreground(fg_color)
                        .border(Hsla {
                            a: 1.0,
                            ..base_hsla
                        })
                        .hover(Hsla {
                            a: 0.6,
                            ..base_hsla
                        })
                        .active(Hsla {
                            a: 0.6,
                            ..base_hsla
                        }),
                )
            })
            .on_click(cx.listener(move |screen, _, _window, _cx| {
                if let Some(idx) = screen.game_state.selected_tower {
                    let tower = &screen.game_state.towers[idx];
                    match &tower.skills[skill_idx] {
                        SkillState::Locked => {
                            screen.game_state.purchase_skill(idx, skill_idx);
                        }
                        SkillState::Purchased(_) => {
                            screen.game_state.activate_skill(idx, skill_idx);
                        }
                        SkillState::Active(_) => {
                            // Already active, do nothing
                        }
                    }
                }
            }))
            .into_any_element()
        })
        .collect();

    // Build stat rows: each stat shows its current value, and if upgradeable, a button with bonus
    // Skip stats for towers with notification settings (Alarme) as they don't attack
    let upgrades = if has_notification_settings {
        Vec::new()
    } else {
        tower.get_active_skill_upgrades()
    };

    let mut stat_elements: Vec<AnyElement> = Vec::new();
    for (upgrade_id, uname, prop) in &upgrades {
        let current_value = prop.value();
        let is_maxed = !prop.can_upgrade();
        let uid = *upgrade_id;

        // Format value
        let value_str = if current_value.fract() == 0.0 {
            format!("{:.0}", current_value)
        } else {
            format!("{:.2}", current_value)
        };

        let mut row = h_flex()
            .id(SharedString::from(format!("stat_row_{:?}", upgrade_id)))
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("{}: {}", uname, value_str)),
            );

        if !is_maxed {
            let cost = prop.cost();
            let bonus = prop.bonus_per_level;
            let can_afford = gold >= cost;
            let bonus_str = if bonus.fract() == 0.0 {
                format!("+{:.0}", bonus)
            } else {
                format!("+{:.1}", bonus)
            };

            row = row.child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0x888888)).child(bonus_str))
                    .child(
                        Button::new(SharedString::from(format!("btn_upgrade_{:?}", uid)))
                            .label(format!("{}g", cost))
                            .compact()
                            .with_size(Size::XSmall)
                            .disabled(!can_afford)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    screen.game_state.upgrade_tower(idx, uid);
                                }
                            })),
                    ),
            );
        }

        stat_elements.push(row.into_any_element());
    }

    // Target priority button (only if active skill allows it)
    let target_priority = tower.target_priority;
    let priority_btn = if tower.can_change_target() {
        Some(
            Button::new("sidebar_target_priority")
                .label(format!("Cible: {}", target_priority.display_name()))
                .compact()
                .with_size(Size::Small)
                .on_click(cx.listener(move |screen, _, _window, _cx| {
                    if let Some(idx) = screen.game_state.selected_tower {
                        if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                            tower.cycle_target_priority();
                        }
                    }
                })),
        )
    } else {
        None
    };

    let move_cost = game.move_cost(tower_idx);
    let can_move = gold >= move_cost;
    let move_btn = Button::new("sidebar_move_tower")
        .label(format!("Deplacer ({}g)", move_cost))
        .compact()
        .with_size(Size::Small)
        .disabled(!can_move)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if let Some(idx) = screen.game_state.selected_tower {
                screen.game_state.move_mode = Some(idx);
            }
        }));

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

    // Notification settings section (for Alarme tower)
    let notification_section = tower.notification_settings.as_ref().map(|settings| {
        let shield_broken = settings.shield_broken;
        let shield_low = settings.shield_low;

        v_flex()
            .gap_2()
            .child(Divider::horizontal().color(Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xaaaaaa))
                    .child("Notifications"),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("notif_shield_broken")
                            .checked(shield_broken)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                                        if let Some(settings) = &mut tower.notification_settings {
                                            settings.shield_broken = !settings.shield_broken;
                                        }
                                    }
                                }
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("Bouclier brise"),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("notif_shield_low")
                            .checked(shield_low)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                                        if let Some(settings) = &mut tower.notification_settings {
                                            settings.shield_low = !settings.shield_low;
                                        }
                                    }
                                }
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("Bouclier <= 25%"),
                    ),
            )
    });

    // Get active skill name for display
    let active_skill_name = tower
        .active_skill_index
        .map(|idx| def.skills[idx].name)
        .unwrap_or("Aucune");

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
            // Skill icons row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xaaaaaa))
                            .child("Competences"),
                    )
                    .child(h_flex().gap_1().children(skill_icons)),
            )
            // Active skill name
            .when(!has_notification_settings, |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x44aaff))
                        .child(format!("Active: {}", active_skill_name)),
                )
            })
            // Stats with inline upgrades (only for active skill)
            .children(stat_elements)
            // Target priority button (if tower attacks)
            .when_some(priority_btn, |this, btn| this.child(btn))
            // Notification settings (if available)
            .when_some(notification_section, |this, section| this.child(section))
            // Move
            .child(move_btn)
            // Sell
            .child(sell_btn),
    )
}

fn tower_icon(
    kind: TowerKind,
    gold: u32,
    slots_full: bool,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let def = get_def(kind);
    let cost = def.base_cost;
    let can_afford = gold >= cost && !slots_full;
    let color = def.element.color();
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

    Button::new(SharedString::from(format!("tower_icon_{:?}", kind)))
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
                screen.game_state.placement_mode = Some(kind);
            }
        }))
}
