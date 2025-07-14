use crate::app::Data;
use crate::ui;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Style, Widget};
use ratatui::widgets::{Gauge, Paragraph};

#[derive(Debug)]
#[derive(Default)]
pub struct InstallViewState {
    /// Completed, total
    pub install_progress: (usize, usize),
}


#[derive(Debug)]
pub struct InstallView<'a> {
    pub data: &'a Data,
    pub state: &'a mut InstallViewState,
}

impl<'a> InstallView<'a> {
    pub fn new(data: &'a Data, state: &'a mut InstallViewState) -> Self {
        Self { data, state }
    }
}

impl<'a> Widget for InstallView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, view, done] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

        let (completed, total) = self.state.install_progress;

        let progress_text = if total > 0 {
            format!("Installing... ({completed}/{total})")
        } else {
            "Installing...".to_string()
        };

        let paragraph = Paragraph::new(progress_text).style(Style::default().fg(ui::TEXT_COLOR));
        paragraph.render(header, buf);

        let ratio = if total > 0 {
            completed as f64 / total as f64
        } else {
            0.0
        };

        Gauge::default()
            .gauge_style(ui::CIRCLE_CHECKED_COLOR)
            .ratio(ratio)
            .label(format!(
                "{}/{} ({}%)",
                completed,
                total,
                (ratio * 100.0).round()
            ))
            .render(view, buf);

        if completed == total && total > 0 {
            let paragraph = Paragraph::new("Done! Press q or Esc to exit...")
                .style(Style::default().fg(ui::TEXT_COLOR));
            paragraph.render(done, buf);
        }
    }
}
