use crate::app::Data;
use crate::models::{Lambda, LoadState};
use crate::ui;
use crate::ui::widgets::loader::{Loader, LoaderState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Widget, Wrap};

#[derive(Debug)]
#[derive(Default)]
pub struct LambdasViewState {
    pub loader: LoaderState,
    pub lambda_list: ListState,
    pub is_selected_all: bool,
    pub current_user: Option<String>,
}


#[derive(Debug)]
pub struct LambdasView<'a> {
    /// Loaded data
    pub data: &'a Data,

    pub state: &'a mut LambdasViewState,
}

impl<'a> LambdasView<'a> {
    pub fn new(data: &'a Data, state: &'a mut LambdasViewState) -> Self {
        Self { data, state }
    }
}

impl<'a> Widget for LambdasView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header, view] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .areas(area);

        match &self.data.lambdas {
            LoadState::Loading => {
                Loader::new("Loading lambdas...").render(header, buf, &mut self.state.loader);
            }

            LoadState::Loaded(lambdas) => {
                let instructions = Paragraph::new("Choose lambdas to install:")
                    .style(Style::default().fg(ui::TEXT_COLOR));

                instructions.render(header, buf);

                let items: Vec<ListItem> = lambdas
                    .iter()
                    .map(|lambda| ListItem::from(lambda).bg(ui::ROW_BACKGROUND_COLOR))
                    .collect();

                let list = List::new(items).highlight_symbol("›");
                StatefulWidget::render(list, view, buf, &mut self.state.lambda_list);
            }

            LoadState::Failed(err) => {
                let error = Text::styled(
                    format!("Failed to fetch lambdas\n{err:?}"),
                    Style::default(),
                );

                let paragraph = Paragraph::new(error).wrap(Wrap { trim: false });
                paragraph.render(view, buf);
            }
        }
    }
}

impl From<&Lambda> for ListItem<'_> {
    fn from(value: &Lambda) -> Self {
        let (checkbox, name) = if value.is_selected {
            (
                Span::styled(" ◉", Style::default().fg(ui::CIRCLE_CHECKED_COLOR)),
                Span::styled(
                    format!(" {}", value.name.clone()),
                    Style::default().fg(ui::LIST_CHECKED_COLOR),
                ),
            )
        } else {
            (
                Span::styled(" ◯", Style::default().fg(ui::CIRCLE_UNCHECKED_COLOR)),
                Span::styled(
                    format!(" {}", value.name.clone()),
                    Style::default().fg(ui::TEXT_COLOR),
                ),
            )
        };

        let name_line = Line::from(vec![checkbox, name]);

        ListItem::new(vec![name_line])
    }
}
