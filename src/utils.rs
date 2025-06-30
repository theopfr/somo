use crate::sout;
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

/// Prints out Markdown formatted text using a custom appearance / termimad "skin".
///
/// # Appearance
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not struck out and green
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_info(text: &str) {
    let mut skin: MadSkin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(Cyan), None, Encircled.into());

    let markdown: String = format!("~~Info~~: *{}*", text);
    sout!("{}", skin.term_text(&markdown));
}

/// Prints out Markdown formatted text using a custom appearance / termimad "skin".
///
/// # Appearance
/// * **bold** text -> bold and white
/// * *italic* text -> not italic and gray
/// * ~~strikeout~~ text -> not struck out and red
///
/// # Arguments
/// * `text`: The text to print to the console.
///
/// # Returns
/// None
pub fn pretty_print_error(text: &str) {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(White);
    skin.italic = CompoundStyle::new(Some(gray(11)), None, Encircled.into());
    skin.strikeout = CompoundStyle::new(Some(Red), None, Encircled.into());

    let markdown: String = format!("~~Error~~: *{}*", text);
    sout!("{}", skin.term_text(&markdown));
}
