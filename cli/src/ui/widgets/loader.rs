use crate::ui;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

#[derive(Debug)]
pub struct LoaderState {
    loader_state: usize,
    frame_counter: usize,
    frames_per_update: usize,
}

impl Default for LoaderState {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderState {
    pub fn new() -> Self {
        Self {
            loader_state: 0,
            frame_counter: 0,
            frames_per_update: 30,
        }
    }
}

pub struct Loader {
    text: String,
}

impl Loader {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }
}

impl StatefulWidget for Loader {
    type State = LoaderState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        const DOTS: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        state.frame_counter += 1;

        if state.frame_counter >= state.frames_per_update {
            state.loader_state = (state.loader_state + 1) % DOTS.len();
            state.frame_counter = 0;
        }

        let spinner = DOTS[state.loader_state];
        let paragraph = Paragraph::new(format!("{} {}", spinner, self.text))
            .alignment(Alignment::Left)
            .fg(ui::TEXT_COLOR);

        paragraph.render(area, buf);
    }
}
