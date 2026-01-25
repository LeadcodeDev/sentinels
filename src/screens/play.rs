use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::{Sizable, Size, h_flex, v_flex};
use std::time::{Duration, Instant};

use crate::data::SaveData;
use crate::data::tower_defs::get_def;
use crate::game::Point2D;
use crate::game::{GamePhase, GameState};
use crate::render::{self, PlacementPreview};
use crate::ui::hud;

pub enum PlayScreenEvent {
    ReturnToLobby,
}

pub struct PlayScreen {
    pub game_state: GameState,
    pub game_running: bool,
    pub selected_skill_tab: Option<usize>,
    cursor_pos: Option<Point<Pixels>>,
    loop_started: bool,
}

impl EventEmitter<PlayScreenEvent> for PlayScreen {}

impl PlayScreen {
    pub fn new(save_data: &SaveData) -> Self {
        Self {
            game_state: GameState::new(save_data),
            game_running: true,
            selected_skill_tab: None,
            cursor_pos: None,
            loop_started: false,
        }
    }

    fn start_game_loop(&mut self, cx: &mut Context<Self>) {
        if self.loop_started {
            return;
        }
        self.loop_started = true;

        cx.spawn(async |this: WeakEntity<PlayScreen>, cx| {
            let mut last = Instant::now();
            loop {
                Timer::after(Duration::from_millis(16)).await;
                let now = Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                let should_continue = this
                    .update(cx, |screen, cx| {
                        if screen.game_running {
                            let adjusted_dt = dt * screen.game_state.speed_multiplier;
                            screen.game_state.tick(adjusted_dt);
                            cx.notify();
                        }
                        screen.game_running
                    })
                    .unwrap_or(false);

                if !should_continue {
                    break;
                }
            }
        })
        .detach();
    }

    fn get_placement_preview(&self) -> Option<PlacementPreview> {
        let cursor = self.cursor_pos?;
        let viewport = self.game_state.viewport_size;
        let sidebar_w = hud::sidebar_width(viewport.0);
        let canvas_width = viewport.0 - sidebar_w;
        let canvas_height = viewport.1;
        let center_x = canvas_width / 2.0;
        let center_y = canvas_height / 2.0;
        let game_x = f32::from(cursor.x) - center_x;
        let game_y = f32::from(cursor.y) - center_y;

        if let Some(tower_idx) = self.game_state.move_mode {
            let tower = self.game_state.towers.get(tower_idx)?;
            return Some(PlacementPreview {
                element: tower.element,
                game_pos: Point2D::new(game_x, game_y),
                range: tower.attack_range(),
            });
        }

        let kind = self.game_state.placement_mode?;
        let def = get_def(kind);

        // Get range from first active skill if available, otherwise no preview range
        let range = def
            .skills
            .iter()
            .find(|s| s.skill_type == crate::data::tower_defs::SkillType::Active)
            .and_then(|s| s.range.as_ref())
            .map(|p| p.value())
            .unwrap_or(0.0);

        Some(PlacementPreview {
            element: def.element,
            game_pos: Point2D::new(game_x, game_y),
            range,
        })
    }
}

impl Render for PlayScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_game_loop(cx);

        let viewport_size = window.viewport_size();
        self.game_state.viewport_size = (
            f32::from(viewport_size.width),
            f32::from(viewport_size.height),
        );

        let placement_preview = self.get_placement_preview();
        let game_canvas = render::render_game(&self.game_state, viewport_size, placement_preview);

        let left_click = cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
            let viewport_size = this.game_state.viewport_size;
            let sidebar_w = hud::sidebar_width(viewport_size.0);
            let canvas_width = viewport_size.0 - sidebar_w;
            let canvas_height = viewport_size.1;
            let center_x = canvas_width / 2.0;
            let center_y = canvas_height / 2.0;
            let game_x = f32::from(event.position.x) - center_x;
            let game_y = f32::from(event.position.y) - center_y;

            if let Some(tower_idx) = this.game_state.move_mode {
                this.game_state.try_move_tower(tower_idx, game_x, game_y);
            } else if let Some(kind) = this.game_state.placement_mode.take() {
                this.game_state.try_place_tower(kind, game_x, game_y);
                // Reset skill tab when placing a new tower
                this.selected_skill_tab = None;
            } else {
                // Reset skill tab when selecting/deselecting towers
                this.selected_skill_tab = None;
                this.game_state.try_select_at(game_x, game_y);
            }
        });
        let right_click = cx.listener(|this, _, _window, _cx| {
            this.game_state.placement_mode = None;
            this.game_state.move_mode = None;
            this.game_state.selected_tower = None;
            this.selected_skill_tab = None;
        });
        let mouse_move = cx.listener(|this, event: &MouseMoveEvent, _window, _cx| {
            this.cursor_pos = Some(event.position);
        });
        let key_down = cx.listener(|this, event: &KeyDownEvent, _window, _cx| {
            if event.keystroke.key.as_str() == "escape" {
                this.game_state.placement_mode = None;
                this.game_state.move_mode = None;
                this.game_state.selected_tower = None;
                this.selected_skill_tab = None;
            }
        });

        let sidebar = hud::render_sidebar(&self.game_state, self.selected_skill_tab, cx);
        let speed_buttons = render_speed_buttons(&self.game_state, cx);
        let top_left_stats = hud::render_top_left_stats(&self.game_state);
        let is_game_over = self.game_state.phase == GamePhase::GameOver;
        let score = self.game_state.economy.score;
        let wave = self.game_state.economy.wave_number;

        div()
            .size_full()
            .relative()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .relative()
                            .child(game_canvas)
                            .on_mouse_down(MouseButton::Left, left_click)
                            .on_mouse_down(MouseButton::Right, right_click)
                            .on_mouse_move(mouse_move)
                            .child(speed_buttons)
                            .child(top_left_stats),
                    )
                    .child(sidebar)
                    .on_key_down(key_down),
            )
            .when(is_game_over, |this| {
                this.child(
                    div()
                        .id("game_over_overlay")
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .bg(Hsla {
                            h: 0.0,
                            s: 0.0,
                            l: 0.0,
                            a: 0.75,
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_mouse_down(MouseButton::Left, |_, _, _| {})
                        .child(
                            v_flex()
                                .items_center()
                                .gap_4()
                                .child(div().text_xl().text_color(rgb(0xff4444)).child("GAME OVER"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0xcccccc))
                                        .child(format!("Score: {} | Vague: {}", score, wave)),
                                )
                                .child(
                                    Button::new("back_lobby")
                                        .danger()
                                        .label("Retour au lobby")
                                        .on_click(cx.listener(|screen, _, _window, cx| {
                                            screen.game_running = false;
                                            cx.emit(PlayScreenEvent::ReturnToLobby);
                                        })),
                                ),
                        ),
                )
            })
    }
}

fn render_speed_buttons(
    game: &GameState,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let current_speed = game.speed_multiplier as u32;

    h_flex()
        .id("speed_buttons")
        .absolute()
        .bottom_3()
        .left_3()
        .gap_1()
        .child(speed_button("x1", 1, current_speed, cx))
        .child(speed_button("x2", 2, current_speed, cx))
        .child(speed_button("x3", 3, current_speed, cx))
}

fn speed_button(
    label: &'static str,
    speed: u32,
    current_speed: u32,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let is_active = current_speed == speed;

    let bg_color = if is_active {
        Hsla {
            h: 0.55,
            s: 0.7,
            l: 0.4,
            a: 1.0,
        }
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.15,
            a: 1.0,
        }
    };

    let hover_color = if is_active {
        Hsla {
            h: 0.55,
            s: 0.7,
            l: 0.5,
            a: 1.0,
        }
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.25,
            a: 1.0,
        }
    };

    let fg_color = if is_active {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        }
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.67,
            a: 1.0,
        }
    };

    let border_color = if is_active {
        Hsla {
            h: 0.55,
            s: 0.7,
            l: 0.6,
            a: 1.0,
        }
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.3,
            a: 1.0,
        }
    };

    Button::new(SharedString::from(format!("speed_{}", speed)))
        .custom(
            ButtonCustomVariant::new(cx)
                .color(bg_color)
                .foreground(fg_color)
                .border(border_color)
                .hover(hover_color)
                .active(hover_color),
        )
        .label(label)
        .compact()
        .with_size(Size::XSmall)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            screen.game_state.speed_multiplier = speed as f32;
        }))
}
