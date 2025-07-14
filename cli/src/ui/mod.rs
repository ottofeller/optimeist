use crate::ui::install::InstallView;
use crate::ui::lambdas::LambdasView;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

pub mod install;
pub mod lambdas;
pub mod widgets;

pub const ROW_SELECTED_STYLE: Style = Style::new().fg(tailwind::SKY.c500);
pub const ROW_BACKGROUND_COLOR: Color = tailwind::NEUTRAL.c900;
pub const LIST_CHECKED_COLOR: Color = tailwind::CYAN.c300;
pub const LIST_CHECKED_COLOR_DIMMED: Color = tailwind::NEUTRAL.c500;
pub const CIRCLE_CHECKED_COLOR: Color = tailwind::EMERALD.c500;
pub const CIRCLE_UNCHECKED_COLOR: Color = tailwind::NEUTRAL.c500;

pub const BLOCK_BACKGROUND_COLOR: Color = tailwind::NEUTRAL.c900;
pub const BLOCK_BORDER_COLOR: Color = tailwind::NEUTRAL.c700;

pub const TEXT_COLOR: Color = tailwind::NEUTRAL.c200;
pub const TEXT_ERROR_COLOR: Color = tailwind::RED.c500;
pub const TEXT_COLOR_DIMMED: Color = tailwind::NEUTRAL.c300;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum View {
    #[default]
    Lambdas,
    Install,
}

#[derive(Debug)]
pub enum Views<'a> {
    Lambdas(LambdasView<'a>),
    Install(InstallView<'a>),
}

impl<'a> Widget for Views<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Views::Lambdas(widget) => widget.render(area, buf),
            Views::Install(widget) => widget.render(area, buf),
        }
    }
}
