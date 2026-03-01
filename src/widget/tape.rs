use iced::widget::{column, container, row, scrollable, text, Column};
use iced::{Element, Length};

use crate::message::Message;
use crate::model::TickCandle;
use crate::theme as t;

pub struct Tape {
    pub ticks: Vec<TickCandle>,
    pub volume_filter: f64,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            ticks: Vec::new(),
            volume_filter: 0.0,
        }
    }

    pub fn update_ticks(&mut self, ticks: Vec<TickCandle>) {
        self.ticks = ticks;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("Time").size(11).color(t::TEXT_DIM).width(Length::Fixed(70.0)),
            text("Price").size(11).color(t::TEXT_DIM).width(Length::Fixed(80.0)),
            text("Vol").size(11).color(t::TEXT_DIM).width(Length::Fixed(60.0)),
            text("Delta").size(11).color(t::TEXT_DIM).width(Length::Fixed(60.0)),
        ]
        .spacing(4)
        .padding(4);

        let entries: Vec<Element<Message>> = self
            .ticks
            .iter()
            .rev()
            .filter(|tick| tick.volume >= self.volume_filter)
            .take(100)
            .map(|tick| {
                let color = if tick.is_bullish() {
                    t::BID_GREEN
                } else {
                    t::ASK_RED
                };

                let ts = format_timestamp(tick.timestamp);
                let delta = tick.delta();
                let delta_color = if delta >= 0.0 {
                    t::BID_GREEN
                } else {
                    t::ASK_RED
                };

                let is_big = tick.volume > self.volume_filter * 5.0 && self.volume_filter > 0.0;
                let size = if is_big { 13 } else { 11 };

                row![
                    text(ts).size(size).color(t::TEXT_DIM).width(Length::Fixed(70.0)),
                    text(format_price(tick.close))
                        .size(size)
                        .color(color)
                        .width(Length::Fixed(80.0)),
                    text(format_volume(tick.volume))
                        .size(size)
                        .color(color)
                        .width(Length::Fixed(60.0)),
                    text(format_volume_signed(delta))
                        .size(size)
                        .color(delta_color)
                        .width(Length::Fixed(60.0)),
                ]
                .spacing(4)
                .padding([1, 4])
                .into()
            })
            .collect();

        let list = Column::with_children(entries).spacing(0);

        container(
            column![header, scrollable(list).height(Length::Fill)]
                .spacing(0)
                .width(Length::Fill),
        )
        .style(|_theme: &_| container::Style {
            background: Some(t::PANEL_BG.into()),
            ..Default::default()
        })
        .width(Length::Fixed(280.0))
        .height(Length::Fill)
        .into()
    }
}

fn format_timestamp(ts: i64) -> String {
    let secs = ts / 1000;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        format!("{:.1}", price)
    } else if price >= 1.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    }
}

fn format_volume(vol: f64) -> String {
    if vol >= 1000.0 {
        format!("{:.0}", vol)
    } else if vol >= 100.0 {
        format!("{:.1}", vol)
    } else if vol >= 1.0 {
        format!("{:.2}", vol)
    } else if vol >= 0.01 {
        format!("{:.3}", vol)
    } else if vol >= 0.001 {
        format!("{:.4}", vol)
    } else {
        format!("{:.5}", vol)
    }
}

fn format_volume_signed(vol: f64) -> String {
    let sign = if vol >= 0.0 { "+" } else { "" };
    let abs = vol.abs();
    if abs >= 1000.0 {
        format!("{}{:.0}", sign, vol)
    } else if abs >= 100.0 {
        format!("{}{:.1}", sign, vol)
    } else if abs >= 1.0 {
        format!("{}{:.2}", sign, vol)
    } else if abs >= 0.01 {
        format!("{}{:.3}", sign, vol)
    } else if abs >= 0.001 {
        format!("{}{:.4}", sign, vol)
    } else {
        format!("{}{:.5}", sign, vol)
    }
}
