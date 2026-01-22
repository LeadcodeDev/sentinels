use gpui::*;
use std::time::{Duration, Instant};

use crate::data::SaveData;
use crate::game::GameState;
use crate::render;
use crate::ui::hud;

pub enum PlayScreenEvent {
    ReturnToLobby,
}

pub struct PlayScreen {
    pub game_state: GameState,
    pub game_running: bool,
    cursor_pos: Option<Point<Pixels>>,
    loop_started: bool,
}

impl EventEmitter<PlayScreenEvent> for PlayScreen {}

impl PlayScreen {
    pub fn new(save_data: &SaveData) -> Self {
        Self {
            game_state: GameState::new(save_data),
            game_running: true,
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
                            screen.game_state.tick(dt);
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
}

impl Render for PlayScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_game_loop(cx);

        let viewport_size = window.viewport_size();
        self.game_state.viewport_size = (
            f32::from(viewport_size.width),
            f32::from(viewport_size.height),
        );

        let game_canvas = render::render_game(&self.game_state, viewport_size);
        let hud_overlay = hud::render_hud(&self.game_state, cx);

        div()
            .size_full()
            .relative()
            .child(game_canvas)
            .child(hud_overlay)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    let viewport_size = this.game_state.viewport_size;
                    let center_x = viewport_size.0 / 2.0;
                    let center_y = viewport_size.1 / 2.0;
                    let game_x = f32::from(event.position.x) - center_x;
                    let game_y = f32::from(event.position.y) - center_y;

                    if let Some(element) = this.game_state.placement_mode.take() {
                        this.game_state.try_place_tower(element, game_x, game_y);
                    } else {
                        this.game_state.try_select_at(game_x, game_y);
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, _window, _cx| {
                    this.game_state.placement_mode = None;
                    this.game_state.selected_tower = None;
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, _cx| {
                this.cursor_pos = Some(event.position);
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, _cx| {
                if event.keystroke.key.as_str() == "escape" {
                    this.game_state.placement_mode = None;
                    this.game_state.selected_tower = None;
                }
            }))
    }
}
