// inspector: Native viewer for author exports (commit platform, pkgs/inspector)
//! Desktop inspector (iced) for an activity export: shows commits per author
//! from `gitstat --json`-style input and lets you flip the sort order.
//!
//!   inspector authors.tsv     # lines of "<count>\t<author>"

use iced::widget::{button, column, row, text};
use iced::{Element, Fill};

#[derive(Debug, Default, PartialEq)]
struct Inspector {
    authors: Vec<(String, u32)>,
    ascending: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleOrder,
}

impl Inspector {
    fn from_tsv(text: &str) -> Self {
        let authors = text
            .lines()
            .filter_map(|line| {
                let (count, name) = line.trim().split_once('\t')?;
                Some((name.to_owned(), count.trim().parse().ok()?))
            })
            .collect();
        let mut inspector = Self {
            authors,
            ascending: false,
        };
        inspector.sort();
        inspector
    }

    fn sort(&mut self) {
        self.authors
            .sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        if self.ascending {
            self.authors.reverse();
        }
    }

    fn total(&self) -> u32 {
        self.authors.iter().map(|(_, n)| n).sum()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleOrder => {
                self.ascending = !self.ascending;
                self.sort();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let rows = self
            .authors
            .iter()
            .map(|(name, n)| row![text(*n).width(60), text(name.as_str())].into());
        column![
            text(format!(
                "{} commits by {} authors",
                self.total(),
                self.authors.len()
            ))
            .size(24),
            button(if self.ascending {
                "Most first"
            } else {
                "Fewest first"
            })
            .on_press(Message::ToggleOrder),
            column(rows).spacing(4).width(Fill),
        ]
        .spacing(12)
        .padding(20)
        .into()
    }
}

fn main() -> iced::Result {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "authors.tsv".into());
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    iced::application(
        move || Inspector::from_tsv(&text),
        Inspector::update,
        Inspector::view,
    )
    .run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_sorts_busiest_first() {
        let inspector = Inspector::from_tsv("3\tbob\n7\talice\nbad line\n3\tcarol\n");
        assert_eq!(
            inspector.authors,
            [
                ("alice".to_owned(), 7),
                ("bob".to_owned(), 3),
                ("carol".to_owned(), 3)
            ]
        );
        assert_eq!(inspector.total(), 13);
    }

    #[test]
    fn toggle_reverses_order() {
        let mut inspector = Inspector::from_tsv("1\ta\n2\tb\n");
        inspector.update(Message::ToggleOrder);
        assert_eq!(inspector.authors[0].0, "a");
        inspector.update(Message::ToggleOrder);
        assert_eq!(inspector.authors[0].0, "b");
    }
}
