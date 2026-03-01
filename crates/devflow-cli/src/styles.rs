use anstyle::{AnsiColor, Color, Style};

pub const HEADER: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Green)))
    .bold();

pub const USAGE: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Green)))
    .bold();

#[allow(dead_code)]
pub const COMMAND: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Cyan)))
    .bold();

pub const LITERAL: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Cyan)));

#[allow(dead_code)]
pub const DESC: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::White)))
    .dimmed();

#[allow(dead_code)]
pub const ERROR: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
    .bold();

#[allow(dead_code)]
pub const BANNER: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Blue)))
    .bold();

pub fn get_clap_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(HEADER)
        .usage(USAGE)
        .literal(LITERAL)
        .placeholder(LITERAL)
}
