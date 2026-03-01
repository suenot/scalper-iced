mod app;
mod hotkeys;
mod message;
mod model;
mod panel;
mod price_axis;
mod theme;
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

    /// Trading symbol
    #[arg(long, default_value = "BTCUSDT")]
    symbol: String,

    /// Price step for orderbook aggregation
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

    let ws_url = format!(
        "ws://{}/ws?symbol={}&priceStep={}&clustersN={}&ticksN={}",
        args.addr, args.symbol, args.price_step, args.clusters_n, args.ticks_n
    );

    let symbol = args.symbol.clone();
    let price_step = args.price_step;

    println!("Starting scalper terminal...");
    println!("  WS URL: {}", ws_url);
    println!("  Symbol: {}", symbol);
    println!("  Price step: {}", price_step);

    iced::application(
        move || App::new(ws_url.clone(), symbol.clone(), price_step),
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
