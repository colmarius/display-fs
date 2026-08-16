use crate::image::{calculate_max_lines, measure_text_with_font_size, Orientation};

/// Split text into pages that fit on the display.
/// Uses word-aware splitting and only breaks a word when it cannot fit on one line.
pub fn split_into_pages(
    text: &str,
    font_size: f32,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> Vec<String> {
    let max_lines = calculate_max_lines(font_size, orientation, physical_width, physical_height);

    if max_lines == 0 {
        return vec![];
    }

    let lines = wrap_text(
        text,
        font_size,
        orientation,
        physical_width,
        physical_height,
    );

    if lines.is_empty() {
        return vec![];
    }

    lines
        .chunks(max_lines)
        .map(|chunk| chunk.join("\n"))
        .collect()
}

/// Wrap text into lines that fit within the display width.
/// Respects word boundaries and existing newlines.
fn wrap_text(
    text: &str,
    font_size: f32,
    orientation: Orientation,
    physical_width: u32,
    physical_height: u32,
) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let (max_width, _) = orientation.dimensions(physical_width, physical_height);
    let mut result = Vec::new();

    for paragraph in text.split('\n') {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() {
            result.push(String::new());
            continue;
        }

        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                // First word on line - check if it fits
                if fits_in_width(word, font_size, max_width) {
                    current_line = word.to_string();
                } else {
                    // Preserve oversized words by splitting them across lines.
                    result.extend(split_word_to_fit(word, font_size, max_width));
                }
            } else {
                // Try adding word to current line
                let test_line = format!("{} {}", current_line, word);
                if fits_in_width(&test_line, font_size, max_width) {
                    current_line = test_line;
                } else {
                    // Start new line
                    result.push(current_line);
                    if fits_in_width(word, font_size, max_width) {
                        current_line = word.to_string();
                    } else {
                        result.extend(split_word_to_fit(word, font_size, max_width));
                        current_line = String::new();
                    }
                }
            }
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }
    }

    result
}

/// Check if text fits within the given pixel width
fn fits_in_width(text: &str, font_size: f32, max_width: u32) -> bool {
    let (width, _) = measure_text_with_font_size(text, font_size);
    width <= max_width
}

/// Split an oversized word into the longest chunks that fit the display width.
/// A glyph wider than the display is kept on its own line rather than discarded.
fn split_word_to_fit(word: &str, font_size: f32, max_width: u32) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for character in word.chars() {
        let mut candidate = current.clone();
        candidate.push(character);

        if fits_in_width(&candidate, font_size, max_width) {
            current = candidate;
        } else {
            if !current.is_empty() {
                chunks.push(current);
            }

            current = character.to_string();
            if !fits_in_width(&current, font_size, max_width) {
                chunks.push(current);
                current = String::new();
            }
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Physical dimensions of the small (0.96") display, used as the test fixture.
    const SMALL_W: u32 = 80;
    const SMALL_H: u32 = 160;
    const LANDSCAPE: Orientation = Orientation::Landscape;

    fn pages(text: &str, font_size: f32) -> Vec<String> {
        split_into_pages(text, font_size, LANDSCAPE, SMALL_W, SMALL_H)
    }

    #[test]
    fn test_split_empty_string() {
        assert!(pages("", 14.0).is_empty());
    }

    #[test]
    fn test_split_single_word_that_fits() {
        let pages = pages("Hello", 14.0);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0], "Hello");
    }

    #[test]
    fn test_split_text_requiring_multiple_pages() {
        // Long text that should span multiple pages at font size 14
        let long_text = "This is a long message that will definitely need to be split across multiple pages because it contains many words and the display is only 160x80 pixels which is quite small for displaying lengthy text content.";
        let pages = pages(long_text, 14.0);
        assert!(
            pages.len() > 1,
            "Expected multiple pages, got {}",
            pages.len()
        );
    }

    #[test]
    fn test_word_boundary_splitting() {
        // Verify words aren't broken mid-word
        let text = "Hello World Test";
        let pages = pages(text, 14.0);

        for page in &pages {
            // Check no partial words (assuming these are complete words)
            let words: Vec<&str> = page.split_whitespace().collect();
            for word in words {
                assert!(
                    text.contains(word),
                    "Found unexpected word fragment: {}",
                    word
                );
            }
        }
    }

    #[test]
    fn test_split_with_newlines() {
        let text = "Line one\nLine two\nLine three";
        let pages = pages(text, 14.0);

        // Should preserve line structure
        assert!(!pages.is_empty());

        // The combined pages should contain all original lines
        let combined: String = pages.join("\n");
        assert!(combined.contains("Line one"));
        assert!(combined.contains("Line two"));
        assert!(combined.contains("Line three"));
    }

    #[test]
    fn test_long_wide_glyph_word_terminates() {
        // Regression: a long word of wide glyphs used to hang while truncating.
        let word = "W".repeat(60);
        let pages = pages(&word, 14.0);
        assert!(!pages.is_empty());

        let displayed: String = pages
            .iter()
            .flat_map(|page| page.chars())
            .filter(|character| *character != '\n')
            .collect();
        assert_eq!(displayed, word, "Long words must not lose characters");

        // Every produced line must actually fit the display width.
        let (max_width, _) = LANDSCAPE.dimensions(SMALL_W, SMALL_H);
        for page in &pages {
            for line in page.lines() {
                assert!(
                    fits_in_width(line, 14.0, max_width),
                    "Line '{}' exceeds display width",
                    line
                );
            }
        }
    }

    #[test]
    fn test_fits_in_width_short_text() {
        assert!(fits_in_width("Hi", 14.0, 160));
    }

    #[test]
    fn test_fits_in_width_long_text() {
        let long = "This is a very long line that definitely won't fit on a 160 pixel wide display";
        assert!(!fits_in_width(long, 14.0, 160));
    }
}
