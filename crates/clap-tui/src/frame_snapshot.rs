use std::collections::HashMap;

use ratatui::layout::{Margin, Rect};

use crate::input::{ActiveTab, HoverTarget, UiState};
use crate::pipeline::ValidationState;
use crate::query::form::{self, OrderedArg};
use crate::spec::CommandPath;

pub(crate) const MAX_DROPDOWN_ROWS: u16 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropdownGeometry {
    pub(crate) rect: Rect,
    pub(crate) visible_rows: usize,
}

#[derive(Debug, Default, Clone)]
pub struct FrameLayout {
    pub sidebar: Option<Rect>,
    pub sidebar_list: Option<Rect>,
    pub form: Option<Rect>,
    pub search: Option<Rect>,
    pub preview: Option<Rect>,
    pub footer: Option<Rect>,
    pub dropdown: Option<Rect>,
    pub sidebar_items: Vec<SidebarItemLayout>,
    pub form_fields: Vec<FormFieldLayout>,
    pub form_inputs: HashMap<String, Rect>,
    pub form_view: Option<Rect>,
    pub form_tabs: Vec<TabButtonLayout>,
    pub footer_buttons: Vec<FooterButtonLayout>,
    pub invalid_field_ids: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct FrameSnapshot {
    pub layout: FrameLayout,
    pub form_scroll_max: u16,
    pub help_scroll_max: u16,
}

impl FrameSnapshot {
    pub fn first_invalid_field_id(&self) -> Option<&str> {
        self.layout.invalid_field_ids.first().map(String::as_str)
    }

    pub fn form_scroll(&self, requested_scroll: u16) -> u16 {
        requested_scroll.min(self.form_scroll_max)
    }

    pub fn help_scroll(&self, requested_scroll: u16) -> u16 {
        requested_scroll.min(self.help_scroll_max)
    }

    pub fn footer_target_at(&self, x: u16, y: u16) -> Option<HoverTarget> {
        self.layout
            .footer_buttons
            .iter()
            .find(|button| contains(button.rect, x, y))
            .map(|button| button.target)
    }

    pub fn tab_at(&self, x: u16, y: u16) -> Option<ActiveTab> {
        self.layout
            .form_tabs
            .iter()
            .find(|tab| contains(tab.rect, x, y))
            .map(|tab| tab.tab)
    }

    pub fn sidebar_item_at(&self, x: u16, y: u16) -> Option<&SidebarItemLayout> {
        self.layout
            .sidebar_items
            .iter()
            .find(|item| contains(item.row, x, y))
    }

    pub fn sidebar_caret_contains(item: &SidebarItemLayout, x: u16, y: u16) -> bool {
        item.caret.is_some_and(|caret| contains(caret, x, y))
    }

    pub fn form_input_rect(&self, arg_id: &str) -> Option<Rect> {
        self.layout.form_inputs.get(arg_id).copied()
    }

    pub fn form_field_layout(&self, arg_id: &str) -> Option<&FormFieldLayout> {
        self.layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == arg_id)
    }

    pub fn form_field_at(&self, x: u16, y: u16) -> Option<FormFieldHitLayout> {
        self.layout.form_fields.iter().find_map(|field| {
            let in_input = contains(field.input, x, y);
            let in_label = field.label.is_some_and(|label| contains(label, x, y));
            let in_description = field
                .description
                .is_some_and(|description| contains(description, x, y));

            (in_input || in_label || in_description).then(|| FormFieldHitLayout {
                arg_id: field.arg_id.clone(),
                in_input,
                in_label,
                in_description,
            })
        })
    }

    pub fn form_view_rect(&self) -> Option<Rect> {
        self.layout.form_view
    }

    pub fn dropdown_contains(&self, x: u16, y: u16) -> bool {
        self.layout
            .dropdown
            .is_some_and(|area| contains(area, x, y))
    }

    pub fn preview_contains(&self, x: u16, y: u16) -> bool {
        self.layout.preview.is_some_and(|area| contains(area, x, y))
    }

    pub fn search_contains(&self, x: u16, y: u16) -> bool {
        self.layout.search.is_some_and(|area| contains(area, x, y))
    }

    pub fn sidebar_contains(&self, x: u16, y: u16) -> bool {
        self.layout.sidebar.is_some_and(|area| contains(area, x, y))
    }

    pub fn sidebar_visible_rows(&self) -> Option<usize> {
        self.layout
            .sidebar_list
            .map(|area| usize::from(area.height))
    }

    pub fn form_contains(&self, x: u16, y: u16) -> bool {
        self.layout.form.is_some_and(|area| contains(area, x, y))
    }

    pub fn form_content_y(&self, row: u16, scroll: u16) -> Option<u16> {
        let form_view = self.layout.form_view?;
        if row < form_view.y || row >= form_view.y + form_view.height {
            return None;
        }
        Some(row.saturating_sub(form_view.y).saturating_add(scroll))
    }

    pub fn input_position_from_point(
        &self,
        arg_id: &str,
        x: u16,
        y: u16,
        clamp: bool,
    ) -> Option<(u16, u16)> {
        let input_rect = self.form_input_rect(arg_id)?;
        let inner_x = input_rect.x.saturating_add(1);
        let inner_y = input_rect.y.saturating_add(1);
        let inner_w = input_rect.width.saturating_sub(2);
        let inner_h = input_rect.height.saturating_sub(2);
        if inner_w == 0 || inner_h == 0 {
            return None;
        }
        if !clamp
            && (x < inner_x || y < inner_y || x >= inner_x + inner_w || y >= inner_y + inner_h)
        {
            return None;
        }
        let x = if clamp {
            x.clamp(inner_x, inner_x + inner_w - 1)
        } else {
            x
        };
        let y = if clamp {
            y.clamp(inner_y, inner_y + inner_h - 1)
        } else {
            y
        };
        Some((
            y.saturating_sub(inner_y).min(inner_h.saturating_sub(1)),
            x.saturating_sub(inner_x).min(inner_w.saturating_sub(1)),
        ))
    }

    pub fn dropdown_choice_index(&self, row: u16, scroll: usize) -> Option<usize> {
        let area = self.layout.dropdown?;
        if row <= area.y || row >= area.y + area.height - 1 {
            return None;
        }
        Some(usize::from(row.saturating_sub(area.y + 1)) + scroll)
    }

    pub fn dropdown_visible_rows(&self) -> Option<usize> {
        self.layout
            .dropdown
            .map(|dropdown| usize::from(dropdown.height.saturating_sub(2)))
    }

    pub fn dropdown_geometry_for_input(
        &self,
        arg_id: &str,
        total_options: usize,
    ) -> Option<DropdownGeometry> {
        self.layout
            .form_view
            .zip(self.form_input_rect(arg_id))
            .and_then(|(form_view, input_rect)| {
                dropdown_geometry(form_view, input_rect, total_options)
            })
    }
}

pub(crate) fn dropdown_geometry(
    form_view: Rect,
    input_rect: Rect,
    total_options: usize,
) -> Option<DropdownGeometry> {
    if total_options == 0 || form_view.height == 0 || form_view.width < 3 || input_rect.width < 3 {
        return None;
    }

    let desired_rows = total_options.min(MAX_DROPDOWN_ROWS as usize);
    let available_below = form_view
        .y
        .saturating_add(form_view.height)
        .saturating_sub(input_rect.y.saturating_add(input_rect.height));
    let available_above = input_rect.y.saturating_sub(form_view.y);

    let rows_below = available_below.saturating_sub(2) as usize;
    let rows_above = available_above.saturating_sub(2) as usize;

    let place_below = if rows_below >= desired_rows || rows_above == 0 {
        rows_below > 0
    } else if rows_above >= desired_rows || rows_below == 0 {
        false
    } else {
        rows_below >= rows_above
    };

    let visible_rows = if place_below { rows_below } else { rows_above }.min(desired_rows);
    if visible_rows == 0 {
        return None;
    }

    let popup_height = u16::try_from(visible_rows).unwrap_or(MAX_DROPDOWN_ROWS) + 2;
    let y = if place_below {
        input_rect.y.saturating_add(input_rect.height)
    } else {
        input_rect.y.saturating_sub(popup_height)
    };

    let width = input_rect.width.min(form_view.width);
    let max_x = form_view
        .x
        .saturating_add(form_view.width.saturating_sub(width));
    let x = input_rect.x.clamp(form_view.x, max_x);

    Some(DropdownGeometry {
        rect: Rect::new(x, y, width, popup_height),
        visible_rows,
    })
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn populate_form_layout(
    ui: &UiState,
    area: Rect,
    active_args: &[OrderedArg<'_>],
    help: &str,
    validation: &ValidationState,
    input_height_overrides: &std::collections::HashMap<String, u16>,
    label_height_overrides: &std::collections::HashMap<String, u16>,
    frame_snapshot: &mut FrameSnapshot,
) {
    let content_area = area;
    let content_height = form::measure_fields_height_with_layout_overrides(
        active_args,
        &validation.field_errors,
        input_height_overrides,
        label_height_overrides,
    );
    let help_height = form::measure_help_height(help);
    let viewport_height = content_area.height;
    let help_viewport_height = help_overlay_content_rect(content_area).height;
    frame_snapshot.form_scroll_max = content_height.saturating_sub(viewport_height);
    frame_snapshot.help_scroll_max = help_height.saturating_sub(help_viewport_height);
    let form_scroll = ui.form_scroll(frame_snapshot);
    let frame_layout = &mut frame_snapshot.layout;
    frame_layout.form = Some(area);
    frame_layout.dropdown = None;
    frame_layout.form_fields.clear();
    frame_layout.form_inputs.clear();
    frame_layout.form_tabs.clear();
    frame_layout.invalid_field_ids.clear();
    frame_layout.form_view = Some(content_area);
    let preferred_label_width = form::preferred_label_column_width(active_args);

    let mut y = i32::from(content_area.y) - i32::from(form_scroll);
    let mut previous_heading = None;
    for item in active_args {
        let heading = form::field_heading(previous_heading, item);
        let show_description = form::field_has_description(
            item.arg,
            validation
                .field_errors
                .get(&item.arg.id)
                .map(String::as_str),
        );
        let metrics = form::field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
        );
        let item_bottom =
            y + i32::from(u16::from(heading.is_some())) + i32::from(metrics.total_height);
        if y >= i32::from(content_area.y) + i32::from(content_area.height) {
            break;
        }
        if item_bottom <= i32::from(content_area.y) {
            y += i32::from(u16::from(heading.is_some())) + i32::from(metrics.total_height);
            previous_heading = item.section_heading.as_deref();
            continue;
        }
        let heading = if heading.is_some() && y >= i32::from(content_area.y) {
            let rect = clipped_rect(area.x, area.width, y, 1, content_area);
            y += 1;
            rect
        } else if heading.is_some() {
            y += 1;
            None
        } else {
            None
        };
        let (label_x, label_width, input_x, input_width) =
            field_content_geometry(area, form::field_is_in_section(item), preferred_label_width);
        let label = if metrics.label_height > 0 {
            clipped_rect(label_x, label_width, y, metrics.label_height, content_area)
        } else {
            None
        };
        let input_y = y + i32::from(form::field_input_offset_with_description(
            item.arg,
            show_description,
        ));
        let Some(input) = clipped_rect(
            input_x,
            input_width,
            input_y,
            metrics.input_height,
            content_area,
        ) else {
            y += i32::from(metrics.total_height);
            continue;
        };
        let description = form_description_rect(
            item,
            y,
            area,
            content_area,
            show_description,
            preferred_label_width,
            input_height_overrides.get(&item.arg.id).copied(),
            label_height_overrides.get(&item.arg.id).copied(),
        );

        frame_layout.form_inputs.insert(item.arg.id.clone(), input);
        if validation.field_errors.contains_key(&item.arg.id) {
            frame_layout.invalid_field_ids.push(item.arg.id.clone());
        }
        let input_clip_top =
            u16::try_from((i32::from(input.y) - input_y).max(0)).unwrap_or(u16::MAX);
        frame_layout.form_fields.push(FormFieldLayout {
            arg_id: item.arg.id.clone(),
            heading,
            section_rail: None,
            section_right_rail: None,
            section_cap: None,
            label,
            input,
            input_clip_top,
            description,
        });

        if ui.dropdown_open.as_deref() == Some(&item.arg.id) {
            frame_layout.dropdown = frame_layout
                .form_view
                .and_then(|form_view| dropdown_geometry(form_view, input, item.arg.choices.len()))
                .map(|geometry| geometry.rect);
        }

        y += i32::from(metrics.total_height);
        previous_heading = item.section_heading.as_deref();
    }
}

pub(crate) fn help_overlay_popup_rect(area: Rect) -> Rect {
    let horizontal = u16::from(area.width > 24);
    let vertical = u16::from(area.height > 8);
    area.inner(Margin {
        horizontal,
        vertical,
    })
}

pub(crate) fn help_overlay_content_rect(area: Rect) -> Rect {
    let popup = help_overlay_popup_rect(area);
    let horizontal = u16::from(popup.width > 4);
    let vertical = u16::from(popup.height > 4);
    popup.inner(Margin {
        horizontal,
        vertical,
    })
}

#[allow(clippy::too_many_arguments)]
fn form_description_rect(
    item: &OrderedArg<'_>,
    y: i32,
    area: Rect,
    content_area: Rect,
    show_description: bool,
    preferred_label_width: u16,
    input_height_override: Option<u16>,
    label_height_override: Option<u16>,
) -> Option<Rect> {
    show_description.then_some(())?;
    let description_y = y + i32::from(form::field_description_offset_with_layout_overrides(
        item.arg,
        show_description,
        input_height_override,
        label_height_override,
    )?);
    let (_, _, input_x, input_width) =
        field_content_geometry(area, form::field_is_in_section(item), preferred_label_width);
    clipped_rect(
        input_x,
        input_width,
        description_y,
        form::field_metrics_with_description_and_layout_overrides(
            item.arg,
            show_description,
            input_height_override,
            label_height_override,
        )
        .description_height
        .max(1),
        content_area,
    )
}

fn field_content_geometry(
    area: Rect,
    in_section: bool,
    preferred_label_width: u16,
) -> (u16, u16, u16, u16) {
    let content_x = if in_section && area.width > form::SECTION_FIELD_INDENT.saturating_add(1) {
        area.x.saturating_add(form::SECTION_FIELD_INDENT)
    } else {
        area.x
    };
    let content_width = if in_section && area.width > form::SECTION_FIELD_INDENT.saturating_add(1) {
        area.width.saturating_sub(form::SECTION_FIELD_INDENT)
    } else {
        area.width
    };
    let gap = form::COLUMN_GAP_WIDTH.min(content_width.saturating_sub(1));
    let label_width = preferred_label_width
        .min(content_width.saturating_sub(gap).saturating_sub(8))
        .max(form::LABEL_COLUMN_MIN_WIDTH);
    let input_x = content_x.saturating_add(label_width).saturating_add(gap);
    let input_width = content_width
        .saturating_sub(label_width)
        .saturating_sub(gap);
    (content_x, label_width, input_x, input_width)
}

fn intersect_rects(rect: Rect, bounds: Rect) -> Option<Rect> {
    let left = rect.x.max(bounds.x);
    let top = rect.y.max(bounds.y);
    let right = rect
        .x
        .saturating_add(rect.width)
        .min(bounds.x.saturating_add(bounds.width));
    let bottom = rect
        .y
        .saturating_add(rect.height)
        .min(bounds.y.saturating_add(bounds.height));

    if left >= right || top >= bottom {
        return None;
    }

    Some(Rect::new(
        left,
        top,
        right.saturating_sub(left),
        bottom.saturating_sub(top),
    ))
}

fn clipped_rect(x: u16, width: u16, top: i32, height: u16, bounds: Rect) -> Option<Rect> {
    let bounded_top = top.max(i32::from(bounds.y));
    let bounded_bottom = top
        .saturating_add(i32::from(height))
        .min(i32::from(bounds.y.saturating_add(bounds.height)));
    if bounded_top >= bounded_bottom {
        return None;
    }

    let y = u16::try_from(bounded_top).ok()?;
    let clipped_height = u16::try_from(bounded_bottom.saturating_sub(bounded_top)).ok()?;
    intersect_rects(Rect::new(x, y, width, clipped_height), bounds)
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::{
        FooterButtonLayout, FrameSnapshot, SidebarItemLayout, TabButtonLayout, dropdown_geometry,
    };
    use crate::input::{ActiveTab, HoverTarget};

    #[test]
    fn query_helpers_hit_expected_targets() {
        let mut snapshot = FrameSnapshot::default();
        snapshot.layout.footer_buttons = vec![FooterButtonLayout {
            target: HoverTarget::Run,
            rect: Rect::new(0, 10, 8, 1),
        }];
        snapshot.layout.form_tabs = vec![TabButtonLayout {
            tab: ActiveTab::Inputs,
            rect: Rect::new(0, 0, 8, 1),
        }];
        snapshot.layout.sidebar_items = vec![SidebarItemLayout {
            path: vec!["build".to_string()].into(),
            row: Rect::new(0, 2, 20, 1),
            caret: None,
            has_children: true,
        }];

        assert_eq!(snapshot.footer_target_at(1, 10), Some(HoverTarget::Run));
        assert_eq!(snapshot.tab_at(1, 0), Some(ActiveTab::Inputs));
        assert_eq!(
            snapshot
                .sidebar_item_at(1, 2)
                .map(|item| item.path.as_slice()),
            Some(&["build".to_string()][..])
        );
    }

    #[test]
    fn input_and_dropdown_queries_respect_inner_geometry() {
        let mut snapshot = FrameSnapshot::default();
        snapshot
            .layout
            .form_inputs
            .insert("name".to_string(), Rect::new(10, 5, 12, 3));
        snapshot.layout.form_view = Some(Rect::new(0, 5, 40, 10));
        snapshot.layout.dropdown = Some(Rect::new(10, 8, 12, 5));

        assert_eq!(
            snapshot.input_position_from_point("name", 11, 6, false),
            Some((0, 0))
        );
        assert_eq!(snapshot.form_content_y(7, 3), Some(5));
        assert!(snapshot.dropdown_contains(11, 9));
        assert_eq!(snapshot.dropdown_choice_index(10, 2), Some(3));
        assert_eq!(snapshot.dropdown_visible_rows(), Some(3));
    }

    #[test]
    fn dropdown_geometry_matches_expected_popup_layout() {
        let form_view = Rect::new(10, 5, 60, 20);
        let input_rect = Rect::new(14, 8, 24, 3);
        let geometry = dropdown_geometry(form_view, input_rect, 4).expect("geometry");

        assert_eq!(geometry.rect.x, input_rect.x);
        assert_eq!(geometry.rect.width, input_rect.width);
        assert_eq!(geometry.rect.y, input_rect.y + input_rect.height);
        assert_eq!(geometry.visible_rows, 4);
    }

    #[test]
    fn populate_form_layout_preserves_form_field_order_when_errors_are_present() {
        use crate::input::{ActiveTab, Focus, UiState};
        use crate::pipeline::ValidationState;
        use crate::query::form::visible_args;
        use crate::spec::{ArgKind, ArgSpec, CommandSpec, ValueCardinality};

        let command = CommandSpec {
            name: "tool".to_string(),
            version: None,
            about: None,
            help: String::new(),
            args: vec![
                ArgSpec {
                    id: "alpha".to_string(),
                    display_name: "--alpha".to_string(),
                    help: None,
                    required: true,
                    kind: ArgKind::Option,
                    default_values: Vec::new(),
                    choices: Vec::new(),
                    position: None,
                    value_cardinality: ValueCardinality::One,
                    value_hint: None,
                    ..ArgSpec::default()
                },
                ArgSpec {
                    id: "beta".to_string(),
                    display_name: "--beta".to_string(),
                    help: None,
                    required: true,
                    kind: ArgKind::Option,
                    default_values: Vec::new(),
                    choices: Vec::new(),
                    position: None,
                    value_cardinality: ValueCardinality::One,
                    value_hint: None,
                    ..ArgSpec::default()
                },
            ],
            subcommands: Vec::new(),
            ..CommandSpec::default()
        };
        let vm = crate::ui::screen::ScreenView {
            command: &command,
            root: &command,
            selected_path: crate::spec::CommandPath::default(),
            tree_rows: Vec::new(),
            sidebar_scroll: 0,
            active_args: visible_args(&command, ActiveTab::Inputs),
            preview_argv: Vec::new(),
            validation: ValidationState {
                is_valid: false,
                summary: Some("Missing required arguments: --alpha, --beta".to_string()),
                field_errors: std::collections::BTreeMap::from([
                    ("beta".to_string(), "Required argument".to_string()),
                    ("alpha".to_string(), "Required argument".to_string()),
                ]),
            },
            effective_values: std::collections::BTreeMap::new(),
            inputs: None,
        };
        let mut snapshot = FrameSnapshot::default();
        let ui = UiState {
            focus: Focus::Sidebar,
            active_tab: ActiveTab::Inputs,
            last_non_help_tab: ActiveTab::Inputs,
            help_open: false,
            help_scroll: 0,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            dropdown_cursor: 0,
            sidebar_scroll: 0,
            form_scroll: 0,
            hover: None,
            hover_tab: None,
            mouse_select: None,
        };

        crate::frame_snapshot::populate_form_layout(
            &ui,
            Rect::new(0, 0, 40, 12),
            &vm.active_args,
            &vm.command.help,
            &vm.validation,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &mut snapshot,
        );

        assert_eq!(
            snapshot
                .layout
                .form_fields
                .iter()
                .map(|field| field.arg_id.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
        assert!(matches!(ui.focus, Focus::Sidebar));
    }

    #[test]
    fn inherited_section_descriptions_indent_with_the_section_content() {
        use clap::{Arg, Command};

        use crate::input::{ActiveTab, Focus, UiState};
        use crate::pipeline::ValidationState;
        use crate::query::form::visible_args_for_path;
        use crate::spec::CommandPath;

        let root = crate::spec::CommandSpec::from_command(
            &Command::new("tool")
                .arg(Arg::new("config").long("config").help("Config file"))
                .subcommand(Command::new("kitchen-sink").subcommand(Command::new("child"))),
        );
        let selected_path =
            CommandPath::from(vec!["kitchen-sink".to_string(), "child".to_string()]);
        let active_args = visible_args_for_path(&root, &selected_path, ActiveTab::Inputs);
        let ui = UiState {
            focus: Focus::Sidebar,
            active_tab: ActiveTab::Inputs,
            last_non_help_tab: ActiveTab::Inputs,
            help_open: false,
            help_scroll: 0,
            selected_arg_index: 0,
            search_query: String::new(),
            editors: crate::editor_state::EditorState::default(),
            dropdown_open: None,
            dropdown_scroll: 0,
            dropdown_cursor: 0,
            sidebar_scroll: 0,
            form_scroll: 0,
            hover: None,
            hover_tab: None,
            mouse_select: None,
        };
        let mut snapshot = FrameSnapshot::default();

        crate::frame_snapshot::populate_form_layout(
            &ui,
            Rect::new(0, 0, 50, 12),
            &active_args,
            &root.help,
            &ValidationState::default(),
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            &mut snapshot,
        );

        let inherited = snapshot
            .layout
            .form_fields
            .iter()
            .find(|field| field.arg_id == "config")
            .expect("inherited config field");

        assert!(inherited.section_rail.is_none());
        assert!(inherited.section_right_rail.is_none());
        assert_eq!(inherited.label.expect("label rect").x, 1);
        assert_eq!(
            inherited.description.expect("description rect").x,
            inherited.input.x
        );
    }
}

#[derive(Debug, Clone)]
pub struct FooterButtonLayout {
    pub target: HoverTarget,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct SidebarItemLayout {
    pub path: CommandPath,
    pub row: Rect,
    pub caret: Option<Rect>,
    pub has_children: bool,
}

#[derive(Debug, Clone)]
pub struct TabButtonLayout {
    pub tab: ActiveTab,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FormFieldLayout {
    pub arg_id: String,
    pub heading: Option<Rect>,
    pub section_rail: Option<Rect>,
    pub section_right_rail: Option<Rect>,
    pub section_cap: Option<Rect>,
    pub label: Option<Rect>,
    pub input: Rect,
    pub input_clip_top: u16,
    pub description: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldHitLayout {
    pub arg_id: String,
    pub in_input: bool,
    pub in_label: bool,
    pub in_description: bool,
}
