use std::fmt;
use termimad::crossterm::style::Color::*;
use termimad::*;

/// Used to decide wether to add padding to the table cells or not
const SMALL_TERMINAL_THRESHOLD: u16 = 100;

/// Represents the alignment of a Markdown table row
#[derive(Clone, Copy)]
pub enum MarkdownRowAlignment {
    Left,
    Center,
}

/// Represents possibilities of padding added to table cells
///
/// # Variants:
/// * `Auto`: Automatically decide wether to add padding based on terminal width
/// * `NoPad`: Add no padding.
#[derive(Clone, Copy)]
pub enum Padding {
    Auto,
    NoPad,
}

/// Represents a cell in a Markdown table
///
/// # Fields
/// * `text`: The text within the cell.
/// * `secondary_text`: Optional secondary text shown next to the main text.
/// * `padded`: Wether to add a padding around the text.
/// * `is_header`: Wether this cell is a header cell, ie. column name
pub struct TableCell {
    text: String,
    secondary_text: Option<String>,
    padding: Padding,
    is_header: bool,
}

impl TableCell {
    pub fn header(text: &str, secondary_text: Option<String>, padding: Padding) -> Self {
        Self {
            text: text.into(),
            secondary_text,
            padding,
            is_header: true,
        }
    }

    pub fn body(text: &str, secondary_text: Option<String>, padding: Padding) -> Self {
        Self {
            text: text.into(),
            secondary_text,
            padding,
            is_header: false,
        }
    }
}

impl fmt::Display for TableCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Headers are formatted as bold, ie. using `**`
        let mut content = if self.is_header {
            format!("**{}**", self.text)
        } else {
            self.text.clone()
        };

        // Add optional secondary text formatted as cursive, ie. using `*`
        if let Some(ref sec) = self.secondary_text {
            content.push_str(&format!(" *{sec}*"));
        }

        // Add optional padding around the cell content, if set to `Auto` padding depends on the terminal width
        let (terminal_width, _) = terminal_size();
        let should_pad = match self.padding {
            Padding::NoPad => false,
            Padding::Auto => terminal_width > SMALL_TERMINAL_THRESHOLD,
        };

        if should_pad {
            write!(f, "\u{2800}{content}\u{2800}")
        } else {
            write!(f, "{content}")
        }
    }
}

/// Returns the alignment of the Markdown table needed based on wether the table is shown in compact mode or not.
///
/// # Arguments
/// * `is_compact`: Wether the table should be shown in compact mode or not
///
/// # Returns
/// The corresponding `MarkdownRowAlignment` enum variant.
pub fn get_row_alignment(is_compact: bool) -> MarkdownRowAlignment {
    if is_compact {
        MarkdownRowAlignment::Left
    } else {
        MarkdownRowAlignment::Center
    }
}

/// Represents a Markdown table with styling, alignment, and content formatting.
///
/// # Fields
/// * `markdown_skin`: A `MadSkin` instance from the Termimad crate used to render and style the markdown table
/// * `content`: The actual string storing the ascii Markdown table representation
/// * `row_seperator`: A string representing a Markdown table format row (eg. `|:-:|:-:|`)
pub struct Table {
    markdown_skin: MadSkin,
    content: String,
    row_separator: String,
}

impl Table {
    pub fn new(num_cols: usize, alignment: MarkdownRowAlignment) -> Self {
        let mut markdown_skin = MadSkin::default();
        markdown_skin.bold.set_fg(Cyan);
        markdown_skin.italic.set_fg(gray(11));

        let row_separator = Table::markdown_fmt_row(num_cols, alignment);

        Self {
            markdown_skin,
            content: String::new(),
            row_separator,
        }
    }

    /// Creates a markdown row format string (eg. `|:-:|:-:|` for centered or `|:--|:--|` for left aligned rows).
    ///
    /// # Arguments:
    /// * `num_cols`: The amount of columns the row has.
    /// * `alignment`: The alignment the row should have
    ///
    /// # Returns:
    /// A Markdown row format string with the specified amount of columns and alignment.
    fn markdown_fmt_row(num_cols: usize, alignment: MarkdownRowAlignment) -> String {
        let cell = match alignment {
            MarkdownRowAlignment::Left => ":--",
            MarkdownRowAlignment::Center => ":-:",
        };

        let row = std::iter::repeat_n(cell, num_cols)
            .collect::<Vec<_>>()
            .join(" | ");

        format!("| {row} |\n")
    }

    /// Builds a markdown row string from the provided table cells
    ///
    /// # Arguments
    /// * `row_cells`: A slice of `TableCell`s to build the row from.
    ///
    /// # Returns
    /// A string representing the Markdown-formatted row.
    fn build_table_row(row_cells: &[TableCell]) -> String {
        format!(
            "|{}|\n",
            row_cells
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("|")
        )
    }

    pub fn add_header(&mut self, header_cells: Vec<TableCell>) {
        self.content.push_str(&self.row_separator);
        self.content
            .push_str(&Table::build_table_row(&header_cells));
        self.content.push_str(&self.row_separator);
    }

    pub fn add_row(&mut self, row_cells: Vec<TableCell>, include_row_separator: bool) {
        self.content.push_str(&Table::build_table_row(&row_cells));
        if include_row_separator {
            self.content.push_str(&self.row_separator);
        }
    }

    pub fn build(&self) -> String {
        format!("{}", self.markdown_skin.term_text(&self.content))
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row_alignment_compact_mode() {
        assert!(matches!(
            get_row_alignment(true),
            MarkdownRowAlignment::Left
        ));
        assert!(matches!(
            get_row_alignment(false),
            MarkdownRowAlignment::Center
        ));
    }

    #[test]
    fn test_table_cell_header() {
        let cell = TableCell::header("text", None, Padding::NoPad);
        let output = format!("{cell}");
        assert_eq!(output, "**text**");

        // With secondary text
        let cell = TableCell::header("text", Some("secondary text".to_string()), Padding::NoPad);
        let output = format!("{cell}");
        assert_eq!(output, "**text** *secondary text*");
    }

    #[test]
    fn test_table_cell_body() {
        let cell = TableCell::body("text", None, Padding::NoPad);
        let output = format!("{cell}");
        assert_eq!(output, "text");

        // With secondary text
        let cell = TableCell::body("text", Some("secondary text".to_string()), Padding::NoPad);
        let output = format!("{cell}");
        assert_eq!(output, "text *secondary text*");
    }

    #[test]
    fn test_markdown_fmt_row_left_alignment() {
        let row = Table::markdown_fmt_row(3, MarkdownRowAlignment::Left);
        assert_eq!(row, "| :-- | :-- | :-- |\n");
    }

    #[test]
    fn test_markdown_fmt_row_center_alignment() {
        let row = Table::markdown_fmt_row(2, MarkdownRowAlignment::Center);
        assert_eq!(row, "| :-: | :-: |\n");
    }

    #[test]
    fn test_table_add_header_and_row() {
        let mut table = Table::new(2, MarkdownRowAlignment::Center);

        let headers = vec![
            TableCell::header("ID", None, Padding::NoPad),
            TableCell::header("Value", None, Padding::NoPad),
        ];

        let row = vec![
            TableCell::body("1", None, Padding::NoPad),
            TableCell::body("foo", None, Padding::NoPad),
        ];

        table.add_header(headers);
        table.add_row(row, false);

        let output = table.to_string();
        assert!(output.contains("**ID**"));
        assert!(output.contains("**Value**"));
        assert!(output.contains("1"));
        assert!(output.contains("foo"));
        assert!(output.contains("|"), "Expected table formatting with pipes");
    }
}
