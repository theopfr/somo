use termimad::crossterm::style::{Color::*};
use termimad::*;
use termimad::terminal_size;


static SMALL_TERMINAL_THRESHOLD: u16 = 100;

pub enum MarkdownRowAlignment {
    LEFT,
    CENTER,
    RIGHT
}


pub fn get_row_alignment(compact_mode: bool) -> MarkdownRowAlignment {
    if compact_mode {
        MarkdownRowAlignment::LEFT
    }
    else {
        MarkdownRowAlignment::CENTER
    }
}


pub fn markdown_fmt_row(num_cols: usize, alignment: MarkdownRowAlignment) -> String {
    let cell = match alignment {
        MarkdownRowAlignment::LEFT => ":--",
        MarkdownRowAlignment::CENTER => ":-:",
        MarkdownRowAlignment::RIGHT => "--:",
    };

    let row = std::iter::repeat(cell)
        .take(num_cols)
        .collect::<Vec<_>>()
        .join(" | ");

    format!("| {} |\n", row)
}


fn pad(cell: &str) -> String {
    format!("\u{2800}{}\u{2800}", cell.to_string())
}


pub fn build_table_header(column_names: &[&str]) -> String {
    let (terminal_width, _) = terminal_size();
    let add_padding = terminal_width > SMALL_TERMINAL_THRESHOLD;

    format!(
        "|{}|\n",
        column_names
            .iter()
            .map(|c| if add_padding { pad(c) } else { c.to_string() })
            .collect::<Vec<_>>()
            .join("|")
    )
}

pub fn build_table_row(idx: usize, row_cells: &[&str]) -> String {
    let (terminal_width, _) = terminal_size();
    let add_padding = terminal_width > SMALL_TERMINAL_THRESHOLD;

    format!(
        "| *{idx}* | {}|\n",
        row_cells
            .iter()
            .map(|c| if add_padding { pad(c) } else { c.to_string() })
            .collect::<Vec<_>>()
            .join("|")
    )
}


/// Uses the termimad crate to create a custom appearance for Markdown text in the console.
///
/// # Appearance
/// * **bold** text -> bold and cyan
/// * *italic* text -> italic and light gray
/// * ~~strikeout~~ text -> not struck out, red and blinking
/// * `inline code` text -> not code formatted, yellow
///
/// # Arguments
/// None
///
/// # Returns
/// A custom markdown "skin".
pub fn set_table_style() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(gray(11));

    skin
}