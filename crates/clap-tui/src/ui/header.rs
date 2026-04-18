use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::config::TuiConfig;
use crate::spec::CommandSpec;

use super::{screen::ScreenView, styles};

pub(crate) fn has_header_content(_command: &CommandSpec) -> bool {
    true
}

pub(crate) fn header_height(command: &CommandSpec, compact: bool) -> u16 {
    if compact {
        1 + u16::from(
            command
                .about
                .as_ref()
                .is_some_and(|about| !about.is_empty()),
        )
    } else {
        2 + u16::from(
            command
                .about
                .as_ref()
                .is_some_and(|about| !about.is_empty()),
        )
    }
}

pub(crate) fn render_header(
    frame: &mut Frame<'_>,
    config: &TuiConfig,
    area: Rect,
    _focused: bool,
    vm: &ScreenView<'_>,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(header_lines(config, vm, area.height))
            .style(styles::surface(config, styles::Surface::Workspace))
            .block(
                ratatui::widgets::Block::default()
                    .style(styles::surface(config, styles::Surface::Workspace)),
            ),
        area,
    );
}

fn header_lines(config: &TuiConfig, vm: &ScreenView<'_>, height: u16) -> Vec<Line<'static>> {
    let mut breadcrumb = Vec::new();
    breadcrumb.push(Span::styled(
        vm.root.name.clone(),
        Style::default()
            .fg(config.theme.result_accent)
            .add_modifier(Modifier::BOLD),
    ));
    for (index, segment) in vm.selected_path.as_slice().iter().enumerate() {
        breadcrumb.push(Span::styled(" > ", Style::default().fg(config.theme.dim)));
        let is_last = index + 1 == vm.selected_path.as_slice().len();
        breadcrumb.push(Span::styled(
            segment.clone(),
            if is_last {
                Style::default()
                    .fg(config.theme.text)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(config.theme.metadata)
            },
        ));
    }

    let mut lines = vec![Line::from(breadcrumb)];
    if let Some(about) = vm.command.about.as_ref().filter(|about| !about.is_empty()) {
        lines.push(Line::from(Span::styled(
            about.clone(),
            Style::default().fg(config.theme.metadata),
        )));
    }
    while lines.len() < usize::from(height) {
        lines.push(Line::default());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{header_height, header_lines};
    use crate::config::TuiConfig;
    use crate::spec::{CommandPath, CommandSpec};
    use crate::ui::screen::ScreenView;

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

    fn view(selected: &CommandSpec) -> ScreenView<'_> {
        let root = Box::leak(Box::new(command("tool", Some("Root command"))));
        ScreenView {
            command: selected,
            root,
            selected_path: CommandPath::from(vec!["serve".to_string()]),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: Vec::new(),
            preview_argv: vec!["tool".to_string(), "serve".to_string()],
            validation: crate::pipeline::ValidationState::default(),
            effective_values: std::collections::BTreeMap::new(),
            inputs: None,
        }
    }

    #[test]
    fn header_renders_breadcrumb_context_on_first_line() {
        let cmd = command("serve", Some("Run server"));
        let lines = header_lines(&TuiConfig::default(), &view(&cmd), 3);

        let combined = lines[0]
            .spans
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();
        assert!(combined.contains("tool"));
        assert!(combined.contains("serve"));
    }

    #[test]
    fn header_renders_description_on_second_line() {
        let cmd = command("serve", Some("Run server"));
        let lines = header_lines(&TuiConfig::default(), &view(&cmd), 3);

        assert_eq!(lines[1].spans[0].content.as_ref(), "Run server");
    }

    #[test]
    fn header_surfaces_breadcrumb_text_for_nested_commands() {
        let cmd = command("deploy", Some("Ship the selected release"));
        let lines = header_lines(&TuiConfig::default(), &view(&cmd), 3);

        let combined = lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|span| span.content.to_string()))
            .collect::<String>();
        assert!(combined.contains("tool"));
        assert!(combined.contains("serve"));
    }

    #[test]
    fn header_keeps_path_line_when_about_is_missing() {
        let cmd = command("serve", None);
        let lines = header_lines(&TuiConfig::default(), &view(&cmd), 2);

        assert_eq!(lines.len(), 2);
        assert!(lines[1].spans.is_empty());
    }

    #[test]
    fn roomy_header_leaves_a_spacer_row_below_context() {
        let cmd = command("serve", Some("Run server"));
        let lines = header_lines(&TuiConfig::default(), &view(&cmd), 3);

        assert!(lines[2].spans.is_empty());
    }

    #[test]
    fn header_height_adds_room_for_description_and_spacer() {
        let cmd = command("serve", Some("Run server"));

        assert_eq!(header_height(&cmd, true), 2);
        assert_eq!(header_height(&cmd, false), 3);
    }
}
