use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::spec::CommandSpec;

use super::{screen::ScreenView, styles};

pub(crate) fn render_header(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    vm: &ScreenView<'_>,
) {
    frame.render_widget(
        Paragraph::new(header_lines(config, vm.command)).style(styles::panel(config)),
        area,
    );
}

fn header_lines(config: &TuiConfig, command: &CommandSpec) -> Vec<Line<'static>> {
    let description = command
        .about
        .as_ref()
        .filter(|about| !about.is_empty())
        .map(|about| {
            Line::from(Span::styled(
                about.clone(),
                Style::default().fg(config.theme.dim),
            ))
        })
        .unwrap_or_default();
    vec![description, Line::default()]
}

#[cfg(test)]
mod tests {
    use super::header_lines;
    use crate::config::TuiConfig;
    use crate::spec::CommandSpec;

    fn command(name: &str, about: Option<&str>) -> CommandSpec {
        CommandSpec {
            name: name.to_string(),
            version: None,
            about: about.map(str::to_string),
            help: String::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
            ..CommandSpec::default()
        }
    }

    #[test]
    fn header_renders_description_on_first_line() {
        let lines = header_lines(&TuiConfig::default(), &command("serve", Some("Run server")));

        assert_eq!(lines[0].spans[0].content.as_ref(), "Run server");
    }

    #[test]
    fn header_does_not_render_breadcrumb_text_for_nested_commands() {
        let lines = header_lines(
            &TuiConfig::default(),
            &command("deploy", Some("Ship the selected release")),
        );

        let combined = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|span| span.content.to_string()))
            .collect::<String>();
        assert!(!combined.contains('>'));
        assert!(!combined.contains("root"));
    }

    #[test]
    fn header_renders_blank_first_line_when_about_is_missing() {
        let lines = header_lines(&TuiConfig::default(), &command("serve", None));

        assert!(lines[0].spans.is_empty());
    }
}
