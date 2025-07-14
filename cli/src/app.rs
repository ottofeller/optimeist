use crate::aws::install::install_extension;
use crate::aws::lambda::fetch_lambda_functions;
use crate::aws::secret;
use crate::event::{AppEvent, Event, EventHandler};
use crate::models::{Lambda, LoadState};
use crate::ui;
use crate::ui::install::{InstallView, InstallViewState};
use crate::ui::lambdas::{LambdasView, LambdasViewState};
use aws_config::BehaviorVersion;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    DefaultTerminal,
};

#[derive(Debug, Default)]
pub struct Data {
    pub lambdas: LoadState<Vec<Lambda>>,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub current_view: ui::View,
    pub events: EventHandler,
    pub data: Data,
    pub lambda_view_state: LambdasViewState,
    pub install_view_state: InstallViewState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_view: ui::View::default(),
            events: EventHandler::new(),
            data: Data::default(),
            lambda_view_state: LambdasViewState::default(),
            install_view_state: InstallViewState::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.events.send(AppEvent::FetchLambdas);
        let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        while self.running {
            terminal.draw(|frame| {
                let view = match &self.current_view {
                    ui::View::Lambdas => ui::Views::Lambdas(LambdasView::new(
                        &self.data,
                        &mut self.lambda_view_state,
                    )),
                    ui::View::Install => ui::Views::Install(InstallView::new(
                        &self.data,
                        &mut self.install_view_state,
                    )),
                };

                frame.render_widget(view, frame.area());
            })?;

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event)?
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::SwitchView(view) => self.current_view = view,
                    AppEvent::FetchLambdas => {
                        let sender = self.events.sender_cloned();
                        let aws_config = aws_config.clone();

                        tokio::spawn(async move {
                            EventHandler::send_static(&sender, AppEvent::FetchLambdasInProgress);

                            match fetch_lambda_functions(&aws_config).await {
                                Ok(lambdas) => EventHandler::send_static(
                                    &sender,
                                    AppEvent::FetchLambdasSuccess(lambdas),
                                ),

                                Err(e) => EventHandler::send_static(
                                    &sender,
                                    AppEvent::FetchLambdasError(e),
                                ),
                            };
                        });
                    }
                    AppEvent::FetchLambdasSuccess(lambdas) => {
                        self.lambda_view_state.lambda_list.select_first();
                        self.data.lambdas = LoadState::Loaded(lambdas)
                    }
                    AppEvent::FetchLambdasError(err) => self.data.lambdas = LoadState::Failed(err),
                    AppEvent::FetchLambdasInProgress => self.data.lambdas = LoadState::Loading,

                    AppEvent::Install(lambdas) => {
                        let total = lambdas.len();

                        EventHandler::send_static(
                            &self.events.sender_cloned(),
                            AppEvent::InstallProgress {
                                completed: 0,
                                total,
                            },
                        );

                        let secret_arn = secret::get_or_create_secret(&aws_config).await?;

                        for lambda in lambdas {
                            let aws_config = aws_config.clone();
                            let sender = self.events.sender_cloned();
                            let secret_arn = secret_arn.clone();

                            tokio::spawn(async move {
                                // TODO Handle errors
                                let result =
                                    install_extension(&aws_config, lambda, &secret_arn).await;

                                if let Err(e) = result {
                                    println!("Error installing extension: {e:?}");
                                }

                                EventHandler::send_static(
                                    &sender,
                                    AppEvent::InstallProgress {
                                        completed: 1,
                                        total,
                                    },
                                );
                            });
                        }
                    }
                    AppEvent::InstallProgress { completed, total } => {
                        self.install_view_state.install_progress.0 += completed;
                        self.install_view_state.install_progress.1 = total;
                    }
                },
            }
        }

        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // TODO Take into account self.view
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Enter => {
                if self.current_view != ui::View::Lambdas {
                    return Ok(());
                }

                if let LoadState::Loaded(lambdas) = &mut self.data.lambdas {
                    let selected_lambdas = lambdas
                        .iter()
                        .filter(|lambda| lambda.is_selected)
                        .cloned()
                        .collect::<Vec<_>>();

                    self.events.send(AppEvent::SwitchView(ui::View::Install));
                    self.events.send(AppEvent::Install(selected_lambdas));
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if self.current_view == ui::View::Lambdas && !self.is_loading() {
                    self.lambda_view_state.lambda_list.select_next()
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.current_view == ui::View::Lambdas && !self.is_loading() {
                    self.lambda_view_state.lambda_list.select_previous()
                }
            }
            KeyCode::Char(' ') => {
                if self.current_view == ui::View::Lambdas && !self.is_loading() {
                    let current_selection = self.lambda_view_state.lambda_list.selected();
                    self.toggle_lambda(current_selection)
                }
            }
            KeyCode::Char('a') => {
                if self.current_view == ui::View::Lambdas && !self.is_loading() {
                    self.toggle_all_lambdas()
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_loading(&self) -> bool {
        matches!(&self.data.lambdas, LoadState::Loading)
    }

    fn toggle_all_lambdas(&mut self) {
        if let LoadState::Loaded(lambdas) = &mut self.data.lambdas {
            self.lambda_view_state.is_selected_all = !self.lambda_view_state.is_selected_all;

            for lambda in lambdas {
                lambda.is_selected = self.lambda_view_state.is_selected_all;
            }
        }
    }

    fn toggle_lambda(&mut self, idx: Option<usize>) {
        if let LoadState::Loaded(lambdas) = &mut self.data.lambdas {
            if let Some(idx) = idx {
                let lambda = &mut lambdas[idx];
                lambda.is_selected = !lambda.is_selected;
            }
        }
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }
}
