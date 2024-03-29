use ratatui::backend::CrosstermBackend;

mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);

    let mut app = app::App::new(std::env::args(), backend)?;
    app.run().await?;

    Ok(())
}
