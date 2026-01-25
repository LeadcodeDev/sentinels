use gpui::*;

use crate::data::SaveData;
use crate::screens::play::{PlayScreen, PlayScreenEvent};
use crate::screens::{lobby, shop, welcome};

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Welcome,
    Lobby,
    Shop,
    Play,
}

pub struct SentinelsApp {
    current_screen: Screen,
    pub save_data: SaveData,
    play_screen: Option<Entity<PlayScreen>>,
}

impl SentinelsApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            current_screen: Screen::Welcome,
            save_data: SaveData::load(),
            play_screen: None,
        }
    }

    pub fn navigate_to(&mut self, screen: Screen, cx: &mut Context<Self>) {
        if screen == Screen::Play && self.play_screen.is_none() {
            let save_data = self.save_data.clone();
            let play = cx.new(|_cx| PlayScreen::new(&save_data));
            cx.subscribe(&play, |this, _, event: &PlayScreenEvent, cx| match event {
                PlayScreenEvent::ReturnToLobby => {
                    this.navigate_to(Screen::Lobby, cx);
                }
            })
            .detach();
            self.play_screen = Some(play);
        }
        if screen != Screen::Play {
            self.play_screen = None;
        }
        self.current_screen = screen;
        cx.notify();
    }
}

impl Render for SentinelsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x1a1a2e))
            .child(match self.current_screen {
                Screen::Welcome => welcome::render(cx).into_any_element(),
                Screen::Lobby => lobby::render(&self.save_data, cx).into_any_element(),
                Screen::Shop => shop::render(&mut self.save_data, cx).into_any_element(),
                Screen::Play => self
                    .play_screen
                    .clone()
                    .expect("PlayScreen should be created by navigate_to()")
                    .into_any_element(),
            })
    }
}
