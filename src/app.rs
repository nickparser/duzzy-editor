use std::{io::Write, time::Duration};

use anyhow::Result;
use crossterm::ExecutableCommand;
use tui::{backend::Backend, Terminal};

use crate::{buffer::Buffer, editor::Editor};

pub struct App<B: Backend + Write> {
    editor: Editor,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(mut args: impl Iterator<Item = String>, backend: B) -> Result<Self> {
        let filepath = args.nth(1);
        // todo: one file at this moment
        let buffer = if let Some(path) = filepath {
            Buffer::from_path(path)?
        } else {
            Buffer::default()
        };

        let editor = Editor::new(buffer);
        let mut terminal = Terminal::new(backend).expect("terminal");

        if cfg!(feature = "crossterm") {
            crossterm::terminal::enable_raw_mode().expect("enable raw mode");
            crossterm::execute!(
                &mut terminal.backend_mut(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::event::EnableMouseCapture
            )
            .expect("enable rules");
        }

        Ok(Self { editor, terminal })
    }

    fn setup_panic() {
        let hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |info| {
            let mut stdout = std::io::stdout();
            stdout
                .execute(crossterm::terminal::LeaveAlternateScreen)
                .ok();
            crossterm::terminal::disable_raw_mode().ok();

            hook(info);
        }));
    }

    pub fn run(&mut self) -> Result<()> {
        Self::setup_panic();
        loop {
            let exit = {
                if crossterm::event::poll(Duration::from_millis(200))? {
                    if let crossterm::event::Event::Key(event) = crossterm::event::read()? {
                        self.editor.handle_event(event.into())?
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if exit {
                break;
            }

            let widget = self.editor.widget();
            self.terminal.draw(|ui| {
                ui.render_widget(widget, ui.size());
            })?;

            let (x, y) = self.editor.cursor();
            self.terminal.set_cursor(x as u16, y as u16)?;
            self.terminal.show_cursor()?;
        }

        Ok(())
    }
}

impl<B: Backend + Write> Drop for App<B> {
    fn drop(&mut self) {
        self.terminal.show_cursor().expect("show cursor");
        if cfg!(feature = "crossterm") {
            crossterm::terminal::disable_raw_mode().expect("disable raw mode");
            crossterm::execute!(
                self.terminal.backend_mut(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::event::DisableMouseCapture
            )
            .expect("disable rules");
        }
    }
}
