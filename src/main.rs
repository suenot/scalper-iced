mod app;
mod dashboard;
mod hotkeys;
mod message;
mod model;
mod panel;
mod panel_message;
mod price_axis;
mod settings;
mod theme;
mod trading_panel;
mod widget;
mod ws;

use clap::Parser;

use crate::app::App;

#[derive(Parser, Debug)]
#[command(name = "marketmaker-iced", about = "Scalper terminal")]
struct Args {
    /// Backend WebSocket address (host:port)
    #[arg(long, default_value = "localhost:8080")]
    addr: String,

    /// Default trading symbol (used when no settings file exists)
    #[arg(long, default_value = "BTCUSDT")]
    symbol: String,

    /// Default price step for orderbook aggregation
    #[arg(long, default_value_t = 0.1)]
    price_step: f64,

    /// Number of cluster candles
    #[arg(long, default_value_t = 7)]
    clusters_n: usize,

    /// Number of tick candles
    #[arg(long, default_value_t = 30)]
    ticks_n: usize,
}

fn theme_fn(_app: &App) -> iced::Theme {
    iced::Theme::Dark
}

fn main() -> iced::Result {
    let args = Args::parse();

    let ws_base_url = format!("ws://{}", args.addr);
    let symbol = args.symbol.clone();
    let price_step = args.price_step;
    let clusters_n = args.clusters_n;
    let ticks_n = args.ticks_n;

    println!("Starting scalper terminal...");
    println!("  WS base: {}", ws_base_url);
    println!("  Default symbol: {}", symbol);
    println!("  Default price step: {}", price_step);

    let ws_base = ws_base_url.clone();
    iced::application(
        move || App::new(ws_base.clone(), symbol.clone(), price_step, clusters_n, ticks_n),
        App::update,
        App::view,
    )
    .title(App::title)
    .subscription(App::subscription)
    .theme(theme_fn)
    .window_size((1400.0, 800.0))
    .antialiasing(true)
    .run()
}
