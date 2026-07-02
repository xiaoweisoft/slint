// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use std::borrow::Cow;

#[derive(Clone)]
pub struct VirtualFile {
    pub canon_path: std::path::PathBuf,
    pub builtin_contents: Option<&'static [u8]>,
}

impl VirtualFile {
    pub fn read(&self) -> Cow<'static, [u8]> {
        match self.builtin_contents {
            Some(static_data) => Cow::Borrowed(static_data),
            None => Cow::Owned(std::fs::read(&self.canon_path).unwrap()),
        }
    }

    pub fn is_builtin(&self) -> bool {
        self.builtin_contents.is_some()
    }
}

pub fn styles() -> Vec<&'static str> {
    builtin_library::styles()
}

pub fn load_file(path: &std::path::Path) -> Option<VirtualFile> {
    match path.strip_prefix("builtin:/") {
        Ok(builtin_path) => builtin_library::load_builtin_file(builtin_path),
        Err(_) => path.exists().then(|| {
            let path =
                crate::pathutils::join(&std::env::current_dir().ok().unwrap_or_default(), path)
                    .unwrap_or_else(|| path.to_path_buf());
            VirtualFile { canon_path: crate::pathutils::clean_path(&path), builtin_contents: None }
        }),
    }
}

#[test]
fn test_load_file() {
    let builtin = load_file(&std::path::PathBuf::from(
        "builtin:/foo/../common/./MadeWithSlint-logo-dark.svg",
    ))
    .unwrap();
    assert!(builtin.is_builtin());
    assert_eq!(
        builtin.canon_path,
        std::path::PathBuf::from("builtin:/common/MadeWithSlint-logo-dark.svg")
    );

    let dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().to_string();
    let dir_path = std::path::PathBuf::from(dir);

    let non_existing = dir_path.join("XXXCargo.tomlXXX");
    assert!(load_file(&non_existing).is_none());

    assert!(dir_path.exists()); // We need some existing path for all the rest

    let cargo_toml = dir_path.join("Cargo.toml");
    let abs_cargo_toml = load_file(&cargo_toml).unwrap();
    assert!(!abs_cargo_toml.is_builtin());
    assert!(crate::pathutils::is_absolute(&abs_cargo_toml.canon_path));
    assert!(abs_cargo_toml.canon_path.exists());

    let current = std::env::current_dir().unwrap();
    assert!(current.ends_with("compiler")); // This test is run in .../internal/compiler

    let cargo_toml = std::path::PathBuf::from("./tests/../Cargo.toml");
    let rel_cargo_toml = load_file(&cargo_toml).unwrap();
    assert!(!rel_cargo_toml.is_builtin());
    assert!(crate::pathutils::is_absolute(&rel_cargo_toml.canon_path));
    assert!(rel_cargo_toml.canon_path.exists());

    assert_eq!(abs_cargo_toml.canon_path, rel_cargo_toml.canon_path);
}

#[test]
fn test_fluent2_style_is_discoverable() {
    let styles = styles();
    assert!(styles.contains(&"fluent2"));
    assert!(styles.contains(&"fluent2-light"));
    assert!(styles.contains(&"fluent2-dark"));

    let fluent2 = load_file(&std::path::PathBuf::from("builtin:/fluent2/std-widgets.slint"))
        .expect("fluent2 std widgets should be built in");
    assert!(fluent2.is_builtin());
    assert_eq!(fluent2.canon_path, std::path::PathBuf::from("builtin:/fluent2/std-widgets.slint"));

    let fluent2_light =
        load_file(&std::path::PathBuf::from("builtin:/fluent2-light/std-widgets.slint"))
            .expect("fluent2-light should alias fluent2");
    assert!(fluent2_light.is_builtin());
    assert_eq!(
        fluent2_light.canon_path,
        std::path::PathBuf::from("builtin:/fluent2/std-widgets.slint")
    );

    let fluent2_dark =
        load_file(&std::path::PathBuf::from("builtin:/fluent2-dark/std-widgets.slint"))
            .expect("fluent2-dark should alias fluent2");
    assert!(fluent2_dark.is_builtin());
    assert_eq!(
        fluent2_dark.canon_path,
        std::path::PathBuf::from("builtin:/fluent2/std-widgets.slint")
    );
}

#[test]
fn test_fluent2_style_compiles_representative_controls() {
    let source = r#"
        import {
            AboutSlint, Button, CheckBox, ComboBox, DatePickerPopup, GroupBox, LineEdit,
            ListView, ProgressIndicator, Slider, SpinBox, Spinner, StandardButton,
            StandardListView, StandardTableView, Switch, TabWidget, TextEdit,
            TimePickerPopup, VerticalBox
        } from "std-widgets.slint";

        export component Main inherits Window {
            VerticalBox {
                Button { text: "Save"; primary: true; }
                StandardButton { kind: ok; }
                CheckBox { text: "Enabled"; checked: true; }
                Switch { text: "Mode"; checked: true; }
                Slider { value: 42; minimum: 0; maximum: 100; }
                LineEdit { text: "Input"; }
                TextEdit { text: "Notes"; }
                SpinBox { value: 3; minimum: 0; maximum: 10; }
                ComboBox { model: ["One", "Two"]; current-index: 0; }
                ListView { for item in ["One", "Two"]: Rectangle { height: 24px; } }
                StandardListView {
                    model: [ { text: "Alpha" }, { text: "Beta" } ];
                    current-item: 0;
                }
                ProgressIndicator { progress: 0.5; }
                Spinner { indeterminate: true; }
                GroupBox { title: "Options"; Button { text: "Apply"; } }
                TabWidget {
                    Tab { title: "One"; Button { text: "First"; } }
                    Tab { title: "Two"; Button { text: "Second"; } }
                }
                StandardTableView {
                    columns: [
                        { title: "Name", min-width: 80px, horizontal-stretch: 1, sort-order: SortOrder.unsorted },
                        { title: "State", min-width: 80px, horizontal-stretch: 1, sort-order: SortOrder.unsorted },
                    ];
                    rows: [
                        [ { text: "Alpha" }, { text: "Ready" } ],
                        [ { text: "Beta" }, { text: "Paused" } ],
                    ];
                }
                DatePickerPopup { title: "Select date"; }
                TimePickerPopup { title: "Select time"; }
                AboutSlint { }
            }

            MenuBar {
                Menu {
                    title: "File";
                    MenuItem { title: "Open"; }
                    MenuItem { title: "Pinned"; checkable: true; checked: true; }
                    MenuSeparator { }
                    Menu {
                        title: "Recent";
                        MenuItem { title: "One"; }
                    }
                }
            }
        }
    "#;

    let mut diagnostics = crate::diagnostics::BuildDiagnostics::default();
    let path = std::path::PathBuf::from("fluent2-controls.slint");
    let syntax_node = crate::parser::parse(source.into(), Some(&path), &mut diagnostics);

    let mut compiler_config =
        crate::CompilerConfiguration::new(crate::generator::OutputFormat::Llr);
    compiler_config.style = Some("fluent2".into());
    compiler_config.embed_resources = crate::EmbedResourcesKind::OnlyBuiltinResources;

    let (_, diagnostics, _) =
        spin_on::spin_on(crate::compile_syntax_node(syntax_node, diagnostics, compiler_config));

    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.to_string_vec());
}

#[test]
fn test_fluent2_light_dark_aliases_select_color_scheme() {
    let source = r#"
        import { ColorSchemeSelector } from "color-scheme.slint";

        export component Main inherits Window {
            out property <bool> is-light: ColorSchemeSelector.color-scheme == ColorScheme.light;
            out property <bool> is-dark: ColorSchemeSelector.color-scheme == ColorScheme.dark;
        }
    "#;

    for (style, expected_light, expected_dark) in
        [("fluent2-light", true, false), ("fluent2-dark", false, true)]
    {
        let mut diagnostics = crate::diagnostics::BuildDiagnostics::default();
        let path = std::path::PathBuf::from(format!("builtin:/{style}/color-scheme-check.slint"));
        let syntax_node = crate::parser::parse(source.into(), Some(&path), &mut diagnostics);

        let mut compiler_config =
            crate::CompilerConfiguration::new(crate::generator::OutputFormat::Llr);
        compiler_config.style = Some(style.into());

        let (doc, diagnostics, _) =
            spin_on::spin_on(crate::compile_syntax_node(syntax_node, diagnostics, compiler_config));

        assert!(
            !diagnostics.has_errors(),
            "{style} should compile: {:?}",
            diagnostics.to_string_vec()
        );

        let root = doc.inner_components.last().unwrap().root_element.borrow();
        let is_light = root.bindings.get("is-light").expect("is-light binding").borrow();
        let is_dark = root.bindings.get("is-dark").expect("is-dark binding").borrow();

        match &is_light.expression {
            crate::expression_tree::Expression::BoolLiteral(value) => {
                assert_eq!(*value, expected_light, "{style} light scheme")
            }
            expr => panic!("{style} light scheme was not constant-folded: {expr:?}"),
        }
        match &is_dark.expression {
            crate::expression_tree::Expression::BoolLiteral(value) => {
                assert_eq!(*value, expected_dark, "{style} dark scheme")
            }
            expr => panic!("{style} dark scheme was not constant-folded: {expr:?}"),
        }
    }
}

#[test]
fn test_fluent2_owns_style_specific_bases() {
    let local_bases =
        ["combobox-base.slint", "spinbox-base.slint", "spinner-base.slint", "tabwidget-base.slint"];

    for base in local_bases {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{base}")))
            .unwrap_or_else(|| panic!("fluent2 should own {base}"));
        assert!(source.is_builtin(), "fluent2 {base} should be embedded");
    }

    let tab_base = load_file(&std::path::PathBuf::from("builtin:/fluent2/tabwidget-base.slint"))
        .expect("fluent2 should own tabwidget-base.slint");
    let tab_base_contents = tab_base.read();
    let tab_base = std::str::from_utf8(&tab_base_contents).unwrap();
    assert!(
        !tab_base.contains("event.delta-y > -root.scroll-delta"),
        "fluent2 tab scroll should use the positive threshold for previous-tab navigation"
    );

    for control in ["combobox.slint", "spinbox.slint", "spinner.slint", "tabwidget.slint"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        assert!(
            !source.contains("../common/"),
            "fluent2 {control} should not import style-specific common bases"
        );
    }
}

#[test]
fn test_fluent2_style_metrics_use_fluent2_tokens() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/style-base.slint"))
        .expect("fluent2 should embed style-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("Fluent2SizeSettings"),
        "fluent2 StyleMetrics should import Fluent2SizeSettings"
    );

    for literal_metric in ["layout-spacing: 8px", "layout-padding: 8px", "text-cursor-width: 1px"] {
        assert!(
            !source.contains(literal_metric),
            "fluent2 StyleMetrics should not hardcode {literal_metric}"
        );
    }
}

#[test]
fn test_fluent2_icons_do_not_use_material_icon_viewbox() {
    let fluent2_icon_files = [
        "_arrow_back.svg",
        "_arrow_forward.svg",
        "_calendar.svg",
        "_clock.svg",
        "_edit.svg",
        "_keyboard.svg",
    ];

    for icon in fluent2_icon_files {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{icon}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {icon}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        assert!(
            !source.contains("viewBox=\"0 -960 960 960\""),
            "fluent2 {icon} should use Fluent-style 24px icon geometry, not Material icon geometry"
        );
    }

    for icon in ["_dismiss.svg", "_eye_show.svg", "_eye_hide.svg"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{icon}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {icon}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        assert!(
            !source.contains("fill=\"#212121\""),
            "fluent2 {icon} should be theme-colorable through Image.colorize, not hardcoded to a dark fill"
        );
    }
}

#[test]
fn test_fluent2_progress_indicator_motion_uses_tokens() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/progressindicator.slint"))
        .expect("fluent2 should embed progressindicator.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("Fluent2SizeSettings.progress-indeterminate-cycle-duration"),
        "fluent2 progress indicator should use the indeterminate cycle duration token"
    );
    assert!(
        source.contains("Fluent2SizeSettings.progress-indeterminate-sweep-duration"),
        "fluent2 progress indicator should use the indeterminate sweep duration token"
    );
    assert!(
        !source.contains("mod(animation-tick(), 2s) / 1s"),
        "fluent2 progress indicator should not hardcode indeterminate timing"
    );
}

#[test]
fn test_fluent2_surface_elevation_uses_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "elevation-shadow-rest-color",
        "elevation-shadow-rest-offset-y",
        "elevation-shadow-rest-blur",
        "elevation-shadow-flyout-color",
        "elevation-shadow-flyout-offset-y",
        "elevation-shadow-flyout-blur",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for surface in ["components.slint", "menu.slint"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{surface}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {surface}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("Fluent2Palette.elevation-shadow-flyout-color"),
            "fluent2 {surface} should use the semantic flyout shadow color token"
        );
        assert!(
            source.contains("Fluent2SizeSettings.elevation-shadow-flyout-offset-y"),
            "fluent2 {surface} should use the semantic flyout shadow offset token"
        );
        assert!(
            source.contains("Fluent2SizeSettings.elevation-shadow-flyout-blur"),
            "fluent2 {surface} should use the semantic flyout shadow blur token"
        );
        assert!(
            !source.contains("menu-shadow-"),
            "fluent2 {surface} should not tie shared elevation to menu-specific tokens"
        );
        assert!(
            !source.contains("Fluent2Palette.shadow"),
            "fluent2 {surface} should not use the generic copied shadow bridge for elevation"
        );
    }
}

#[test]
fn test_fluent2_focus_ring_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["focus-ring-gap", "focus-ring-inner-stroke-width", "focus-ring-radius-adjust"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let components = load_file(&std::path::PathBuf::from("builtin:/fluent2/components.slint"))
        .expect("fluent2 should embed components.slint");
    let components_contents = components.read();
    let components = std::str::from_utf8(&components_contents).unwrap();
    let focus_border = components
        .split("export component FocusBorder")
        .nth(1)
        .and_then(|after| after.split("export component MenuBorder").next())
        .expect("fluent2 components should define FocusBorder before MenuBorder");

    for expected in [
        "x: -Fluent2SizeSettings.focus-ring-gap",
        "y: -Fluent2SizeSettings.focus-ring-gap",
        "width: parent.width + 2 * Fluent2SizeSettings.focus-ring-gap",
        "height: parent.height + 2 * Fluent2SizeSettings.focus-ring-gap",
        "border-width: Fluent2SizeSettings.focus-stroke-width",
        "border-width: Fluent2SizeSettings.focus-ring-inner-stroke-width",
        "root.border-radius + Fluent2SizeSettings.focus-ring-radius-adjust",
        "parent.border-radius - Fluent2SizeSettings.focus-stroke-width",
    ] {
        assert!(focus_border.contains(expected), "fluent2 FocusBorder should use {expected}");
    }

    for copied_literal in ["border-width: Fluent2SizeSettings.stroke-width"] {
        assert!(
            !focus_border.contains(copied_literal),
            "fluent2 FocusBorder should not use copied focus geometry literal {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_state_layer_opacity_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["state-layer-hover-opacity", "state-layer-active-opacity"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let state_layer = source
        .split("export component StateLayer")
        .nth(1)
        .and_then(|after| after.split("export struct IconButtonStyle").next())
        .expect("fluent2 internal components should define StateLayer before IconButtonStyle");

    for expected in [
        "root.state-brush.with_alpha(Fluent2SizeSettings.state-layer-active-opacity)",
        "root.state-brush.with_alpha(Fluent2SizeSettings.state-layer-hover-opacity)",
    ] {
        assert!(state_layer.contains(expected), "fluent2 StateLayer should use {expected}");
    }

    for copied_literal in ["with_alpha(0.12)", "with_alpha(0.08)"] {
        assert!(
            !state_layer.contains(copied_literal),
            "fluent2 StateLayer should not hardcode state opacity as {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_scrollbar_opacity_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["scrollbar-button-hidden-opacity", "scrollbar-button-visible-opacity"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/scrollview.slint"))
        .expect("fluent2 should embed scrollview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "opacity: Fluent2SizeSettings.scrollbar-button-visible-opacity",
        "up-scroll-button.opacity: Fluent2SizeSettings.scrollbar-button-visible-opacity",
        "down-scroll-button.opacity: Fluent2SizeSettings.scrollbar-button-visible-opacity",
        "opacity: Fluent2SizeSettings.scrollbar-button-hidden-opacity",
    ] {
        assert!(source.contains(expected), "fluent2 scrollview should use {expected}");
    }

    for copied_literal in
        ["opacity: 0", "opacity: 1", "up-scroll-button.opacity: 1", "down-scroll-button.opacity: 1"]
    {
        assert!(
            !source.contains(copied_literal),
            "fluent2 scrollview should not hardcode scrollbar opacity as {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_selection_indicator_hidden_height_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("selection-indicator-hidden-height"),
        "fluent2 styling should expose selection-indicator-hidden-height"
    );

    for control in ["components.slint", "tableview.slint"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("Fluent2SizeSettings.selection-indicator-hidden-height"),
            "fluent2 {control} should use the hidden selection indicator height token"
        );
        assert!(
            !source.contains("selection-indicator-height : 0"),
            "fluent2 {control} should not hardcode hidden selection indicator height"
        );
    }
}

#[test]
fn test_fluent2_hidden_strokes_use_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("control-hidden-stroke-width"),
        "fluent2 styling should expose control-hidden-stroke-width"
    );

    for control in ["checkbox.slint", "switch.slint", "tabwidget.slint"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("Fluent2SizeSettings.control-hidden-stroke-width"),
            "fluent2 {control} should use the hidden stroke token"
        );
        for copied_literal in [
            "? 0 : Fluent2SizeSettings.stroke-width",
            "? Fluent2SizeSettings.stroke-width : 0",
            "? Fluent2SizeSettings.stroke-width : 0px",
        ] {
            assert!(
                !source.contains(copied_literal),
                "fluent2 {control} should not hardcode hidden stroke width as {copied_literal}"
            );
        }
    }
}

#[test]
fn test_fluent2_spinbox_buttons_use_state_tokens() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinbox.slint"))
        .expect("fluent2 should embed spinbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let button = source
        .split("component SpinBoxButton")
        .nth(1)
        .and_then(|after| after.split("export component SpinBox").next())
        .expect("fluent2 spinbox should define SpinBoxButton before SpinBox");

    for expected in [
        "in property <bool> enabled",
        "enabled: root.enabled",
        "out property <bool> has-focus: touch-area.has-focus",
        "FocusTouchArea",
        "disabled when !root.enabled",
        "hover when touch-area.has-hover",
        "background.background: Fluent2Palette.subtle-secondary",
        "background.background: Fluent2Palette.subtle-tertiary",
        "icon.colorize: Fluent2Palette.text-disabled",
        "animate background, border-color",
    ] {
        assert!(button.contains(expected), "fluent2 SpinBoxButton should use {expected}");
    }

    assert!(
        !button.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 SpinBoxButton should use FocusTouchArea instead of a bare TouchArea"
    );

    assert!(
        source.contains("enabled: root.enabled && !root.read-only"),
        "fluent2 SpinBox buttons should derive enabled state from the SpinBox state"
    );
    assert!(
        source.contains("visible: !root.read-only"),
        "fluent2 SpinBox buttons should remain visible while disabled so disabled states can render"
    );
    assert!(
        !source.contains("visible: self.enabled"),
        "fluent2 SpinBox buttons should not hide their disabled state"
    );
}

#[test]
fn test_fluent2_switch_disabled_thumb_uses_single_token_path() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["switch-thumb-disabled", "switch-thumb-checked-disabled"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let disabled_state = source
        .split("disabled when !root.enabled")
        .nth(1)
        .and_then(|after| after.split("pressed when touch-area.pressed").next())
        .expect("fluent2 switch should define a disabled state before pressed state");

    assert_eq!(
        disabled_state.matches("thumb.background:").count(),
        1,
        "fluent2 Switch disabled state should not assign thumb.background twice"
    );
    assert!(
        disabled_state.contains(
            "thumb.background: root.checked ? Fluent2Palette.switch-thumb-checked-disabled : Fluent2Palette.switch-thumb-disabled"
        ),
        "fluent2 Switch disabled thumb should use explicit checked and unchecked disabled tokens"
    );
    assert!(
        disabled_state.contains(
            "rail.background: root.checked ? Fluent2Palette.accent-disabled : transparent"
        ),
        "fluent2 Switch disabled rail should keep checked accent-disabled and unchecked transparent treatment"
    );
}

#[test]
fn test_fluent2_text_controls_use_font_family_token() {
    let controls = [
        ("button.slint", "Text"),
        ("checkbox.slint", "Text"),
        ("switch.slint", "Text"),
        ("groupbox.slint", "Text"),
        ("combobox.slint", "Text"),
        ("menu.slint", "MenuBarItemBase"),
        ("components.slint", "ListItem"),
        ("tableview.slint", "StandardTableView"),
        ("tabwidget.slint", "TabWidget"),
        ("lineedit.slint", "LineEditBase"),
        ("textedit.slint", "TextEditBase"),
        ("spinbox.slint", "SpinBoxBase"),
        ("datepicker.slint", "DatePickerPopup"),
        ("time-picker.slint", "TimePickerPopup"),
        ("about-slint.slint", "AboutSlint"),
    ];

    for (control, label) in controls {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("Fluent2Palette.font-family"),
            "fluent2 {control} {label} text should use the theme font-family token"
        );
    }

    for base in ["lineedit-base.slint", "textedit-base.slint"] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{base}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {base}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("font-family <=> text-input.font-family"),
            "fluent2 {base} should preserve public font-family forwarding"
        );
    }
}

#[test]
fn test_fluent2_table_column_resize_uses_tokens() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("Fluent2SizeSettings.table-min-column-width"),
        "fluent2 table column sizing should use the minimum column width token"
    );
    assert!(
        !source.contains("column.width >= 1px"),
        "fluent2 table column width checks should not hardcode 1px"
    );
    assert!(
        !source.contains("max(1px, self.width + diff)"),
        "fluent2 table column resizing should not hardcode the minimum width"
    );
    assert!(
        source.contains("Fluent2SizeSettings.table-unconstrained-column-width"),
        "fluent2 table unconstrained column width should use a token"
    );
    assert!(
        !source.contains("100000px"),
        "fluent2 table unconstrained column width should not hardcode a sentinel width"
    );
}

#[test]
fn test_fluent2_lineedit_password_icon_uses_tokens() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit.slint"))
        .expect("fluent2 should embed lineedit.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let password_icon = source
        .split("if root.input-type == InputType.password")
        .nth(1)
        .expect("fluent2 line edit should define a password visibility icon");

    assert!(
        password_icon.contains("icon-size: Fluent2SizeSettings.lineedit-icon-size"),
        "fluent2 password visibility icon should use the line edit icon size token"
    );
    assert!(
        !source.contains("width: self.source.width * 1px"),
        "fluent2 password icon sizing should not derive visual geometry from image source width"
    );
}

#[test]
fn test_fluent2_lineedit_icons_use_state_layers() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit-base.slint"))
        .expect("fluent2 should embed lineedit-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("from \"internal-components.slint\""),
        "fluent2 line edit icon helpers should import Fluent2 interaction components"
    );

    for component in ["LineEditClearIcon", "LineEditPasswordIcon"] {
        let block = source
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|after| after.split("export component ").next())
            .unwrap_or_else(|| panic!("fluent2 should define {component}"));

        for expected in [
            "in property <bool> enabled",
            "in property <length> icon-size",
            "FocusTouchArea",
            "StateLayer",
            "icon := Image",
            "enabled: root.enabled",
            "pressed: touch-area.pressed",
            "has-hover: touch-area.has-hover",
            "state-brush: root.state-brush",
            "min-width: Fluent2SizeSettings.lineedit-icon-touch-target",
            "min-height: Fluent2SizeSettings.lineedit-icon-touch-target",
        ] {
            assert!(block.contains(expected), "fluent2 {component} should use {expected}");
        }

        assert!(
            !block.lines().any(|line| line.trim_start().starts_with("TouchArea {")),
            "fluent2 {component} should use FocusTouchArea instead of a bare TouchArea"
        );
        assert!(
            !source.contains(&format!("export component {component} inherits Image")),
            "fluent2 {component} should be a touch-target component with an inner Image"
        );
    }

    let lineedit = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit.slint"))
        .expect("fluent2 should embed lineedit.slint");
    let lineedit_contents = lineedit.read();
    let lineedit = std::str::from_utf8(&lineedit_contents).unwrap();
    assert!(
        lineedit.contains("enabled: root.enabled && !root.read-only"),
        "fluent2 line edit clear icon should receive enabled state from LineEdit"
    );
    assert!(
        lineedit.contains("enabled: root.enabled"),
        "fluent2 line edit password icon should receive enabled state from LineEdit"
    );
}

#[test]
fn test_fluent2_time_picker_selectors_use_state_layers() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker-base.slint"))
        .expect("fluent2 should embed time-picker-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea") && source.contains("StateLayer"),
        "fluent2 time picker should import Fluent2 interaction helpers"
    );

    for component in ["TimeSelector", "PeriodSelectorItem"] {
        let block = source
            .split(&format!("component {component}"))
            .nth(1)
            .and_then(|after| after.split("component ").next())
            .unwrap_or_else(|| panic!("fluent2 time picker should define {component}"));

        for expected in [
            "in property <bool> enabled",
            "FocusTouchArea",
            "StateLayer",
            "enabled: root.enabled",
            "pressed: touch-area.pressed",
            "has-hover: touch-area.has-hover",
            "has-focus: touch-area.has-focus",
            "state-brush: root.style.state-brush",
        ] {
            assert!(block.contains(expected), "fluent2 {component} should use {expected}");
        }

        assert!(
            !block.lines().any(|line| line.trim_start().starts_with("TouchArea")),
            "fluent2 {component} should use FocusTouchArea instead of a bare TouchArea"
        );
    }
}

#[test]
fn test_fluent2_date_picker_delegates_use_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/datepicker-base.slint"))
        .expect("fluent2 should embed datepicker-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea") && source.contains("StateLayer"),
        "fluent2 date picker should import Fluent2 interaction helpers"
    );

    let block = source
        .split("component CalendarDelegate")
        .nth(1)
        .and_then(|after| after.split("export struct CalendarStyle").next())
        .expect("fluent2 date picker should define CalendarDelegate");

    for expected in [
        "in property <bool> enabled",
        "FocusTouchArea",
        "StateLayer",
        "enabled: root.enabled",
        "pressed: touch-area.pressed",
        "has-hover: touch-area.has-hover",
        "has-focus: touch-area.has-focus",
        "state-brush: root.style.state-brush",
    ] {
        assert!(block.contains(expected), "fluent2 CalendarDelegate should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("TouchArea")),
        "fluent2 CalendarDelegate should use FocusTouchArea instead of a bare TouchArea"
    );
    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("FocusScope")),
        "fluent2 CalendarDelegate should use FocusTouchArea instead of a separate FocusScope"
    );
}

#[test]
fn test_fluent2_combobox_popup_items_use_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/combobox.slint"))
        .expect("fluent2 should embed combobox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 ComboBox popup items should import Fluent2 interaction helpers"
    );

    let block = source
        .split("for value[index] in root.model : ListItem")
        .nth(1)
        .and_then(|after| after.split("                    }").next())
        .expect("fluent2 ComboBox should define popup ListItem delegates");

    for expected in [
        "has-focus: touch-area.has-focus",
        "has-hover: touch-area.has-hover",
        "pressed: touch-area.pressed",
        "touch-area := FocusTouchArea",
        "enabled: root.enabled",
        "clicked =>",
        "base.select(index)",
    ] {
        assert!(block.contains(expected), "fluent2 ComboBox popup item should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 ComboBox popup items should use FocusTouchArea instead of a bare TouchArea"
    );
}

#[test]
fn test_fluent2_list_items_use_focus_touch_area() {
    let helper = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let helper_contents = helper.read();
    let helper = std::str::from_utf8(&helper_contents).unwrap();
    let helper_block = helper
        .split("export component FocusTouchArea")
        .nth(1)
        .and_then(|after| after.split("export component IconButton").next())
        .expect("fluent2 should define FocusTouchArea");

    for expected in [
        "out property <length> pressed-x <=> touch-area.pressed-x",
        "out property <length> pressed-y <=> touch-area.pressed-y",
        "out property <length> mouse-x <=> touch-area.mouse-x",
        "out property <length> mouse-y <=> touch-area.mouse-y",
        "callback pointer-event <=> touch-area.pointer-event",
    ] {
        assert!(helper_block.contains(expected), "fluent2 FocusTouchArea should expose {expected}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/listview.slint"))
        .expect("fluent2 should embed listview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 StandardListView should import the Fluent2 interaction helper"
    );

    let block = source
        .split("for item[index] in root.model : ListItem")
        .nth(1)
        .and_then(|after| after.split("export component StandardListView").next())
        .expect("fluent2 should define StandardListView item delegates");

    for expected in [
        "has-focus: i-touch-area.has-focus || (root.has-focus && index == root.focus-item)",
        "has-hover: i-touch-area.has-hover",
        "pressed: i-touch-area.pressed",
        "pressed-x: i-touch-area.pressed-x",
        "pressed-y: i-touch-area.pressed-y",
        "i-touch-area := FocusTouchArea",
        "enabled: true",
        "clicked =>",
        "root.set-current-item(index)",
        "pointer-event(pe) =>",
        "self.mouse-x - root.absolute-position.x",
        "self.mouse-y - root.absolute-position.y",
    ] {
        assert!(block.contains(expected), "fluent2 StandardListView item should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("i-touch-area := TouchArea")),
        "fluent2 StandardListView items should use FocusTouchArea instead of a bare TouchArea"
    );
}

#[test]
fn test_fluent2_table_rows_use_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 TableView rows should import the Fluent2 interaction helper"
    );

    let block = source
        .split("component TableViewRow")
        .nth(1)
        .and_then(|after| after.split("export component StandardTableView").next())
        .expect("fluent2 should define TableViewRow");

    for expected in [
        "callback clicked <=> touch-area.clicked",
        "touch-area := FocusTouchArea",
        "enabled: true",
        "pressed when touch-area.pressed",
        "hover when touch-area.has-hover",
        "pointer-event(pe) =>",
        "self.absolute-position.x + self.mouse-x",
        "self.absolute-position.y + self.mouse-y",
    ] {
        assert!(block.contains(expected), "fluent2 TableViewRow should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 TableViewRow should use FocusTouchArea instead of a bare TouchArea"
    );
}

#[test]
fn test_fluent2_button_uses_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/button.slint"))
        .expect("fluent2 should embed button.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 Button should import the Fluent2 interaction helper"
    );

    let block =
        source.split("export component Button").nth(1).expect("fluent2 should define Button");

    for expected in [
        "out property <bool> has-focus: i-touch-area.has-focus",
        "out property <bool> pressed: self.enabled && i-touch-area.pressed",
        "forward-focus: i-touch-area",
        "i-touch-area := FocusTouchArea",
        "enabled: root.enabled",
        "clicked =>",
        "root.checkable",
        "root.clicked()",
    ] {
        assert!(block.contains(expected), "fluent2 Button should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("i-touch-area := TouchArea")),
        "fluent2 Button should use FocusTouchArea instead of a bare TouchArea"
    );
    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("i-focus-scope := FocusScope")),
        "fluent2 Button should use FocusTouchArea instead of a separate FocusScope"
    );
}

#[test]
fn test_fluent2_checkbox_uses_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 CheckBox should import the Fluent2 interaction helper"
    );

    let block =
        source.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    for expected in [
        "in property <bool> enabled: true",
        "out property <bool> has-focus: touch-area.has-focus",
        "forward-focus: touch-area",
        "touch-area := FocusTouchArea",
        "enabled: root.enabled",
        "clicked =>",
        "root.checked = !root.checked",
        "root.toggled()",
    ] {
        assert!(block.contains(expected), "fluent2 CheckBox should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 CheckBox should use FocusTouchArea instead of a bare TouchArea"
    );
    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("focus-scope := FocusScope")),
        "fluent2 CheckBox should use FocusTouchArea instead of a separate FocusScope"
    );
}

#[test]
fn test_fluent2_switch_uses_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 Switch should import the Fluent2 interaction helper"
    );

    let block =
        source.split("export component Switch").nth(1).expect("fluent2 should define Switch");

    for expected in [
        "out property <bool> has-focus: touch-area.has-focus",
        "forward-focus: touch-area",
        "touch-area := FocusTouchArea",
        "enabled: root.enabled",
        "clicked =>",
        "root.toggle-checked()",
    ] {
        assert!(block.contains(expected), "fluent2 Switch should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 Switch should use FocusTouchArea instead of a bare TouchArea"
    );
    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("focus-scope := FocusScope")),
        "fluent2 Switch should use FocusTouchArea instead of a separate FocusScope"
    );
}

#[test]
fn test_fluent2_tabs_use_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tabwidget.slint"))
        .expect("fluent2 should embed tabwidget.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 tabs should import the Fluent2 interaction helper"
    );

    let block = source
        .split("export component TabImpl")
        .nth(1)
        .and_then(|after| after.split("component FluentTabBarBase").next())
        .expect("fluent2 should define TabImpl");

    for expected in [
        "out property <bool> has-focus: root.current-focused == root.tab-index || i-touch-area.has-focus",
        "forward-focus: i-touch-area",
        "i-touch-area := FocusTouchArea",
        "enabled <=> root.enabled",
        "clicked =>",
        "root.current = root.tab-index",
    ] {
        assert!(block.contains(expected), "fluent2 TabImpl should use {expected}");
    }

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("i-touch-area := TouchArea")),
        "fluent2 TabImpl should use FocusTouchArea instead of a bare TouchArea"
    );
}

#[test]
fn test_fluent2_owns_lineedit_base_geometry() {
    let base = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit-base.slint"))
        .expect("fluent2 should own lineedit-base.slint");
    assert!(base.is_builtin(), "fluent2 lineedit-base.slint should be embedded");

    let base_contents = base.read();
    let base = std::str::from_utf8(&base_contents).unwrap();
    assert!(
        base.contains("Fluent2SizeSettings.input-min-focusable-width"),
        "fluent2 line edit base should use the minimum focusable width token"
    );
    assert!(
        !base.contains("min-width: 1px"),
        "fluent2 line edit base should not hardcode the focusable minimum width"
    );

    let lineedit = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit.slint"))
        .expect("fluent2 should embed lineedit.slint");
    let lineedit_contents = lineedit.read();
    let lineedit = std::str::from_utf8(&lineedit_contents).unwrap();
    assert!(
        lineedit.contains("from \"lineedit-base.slint\""),
        "fluent2 line edit should import the Fluent2-local line edit base"
    );
    assert!(
        !lineedit.contains("../common/lineedit-base.slint"),
        "fluent2 line edit should not import the shared common line edit base"
    );
}

#[test]
fn test_fluent2_owns_slider_base_interaction_geometry() {
    let base = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider-base.slint"))
        .expect("fluent2 should own slider-base.slint");
    assert!(base.is_builtin(), "fluent2 slider-base.slint should be embedded");

    let base_contents = base.read();
    let base = std::str::from_utf8(&base_contents).unwrap();
    assert!(
        base.contains("Fluent2SizeSettings.slider-focus-scope-size"),
        "fluent2 slider base should use the focus scope size token"
    );
    assert!(
        !base.contains("width: 0;"),
        "fluent2 slider base should not hardcode focus scope width"
    );
    assert!(
        !base.contains("height: 0;"),
        "fluent2 slider base should not hardcode focus scope height"
    );

    let slider = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let slider_contents = slider.read();
    let slider = std::str::from_utf8(&slider_contents).unwrap();
    assert!(
        slider.contains("from \"slider-base.slint\""),
        "fluent2 slider should import the Fluent2-local slider base"
    );
    assert!(
        !slider.contains("../common/slider-base.slint"),
        "fluent2 slider should not import the shared common slider base"
    );
}

#[test]
fn test_fluent2_interaction_receivers_use_tokens() {
    let controls = [
        "button.slint",
        "checkbox.slint",
        "components.slint",
        "datepicker-base.slint",
        "internal-components.slint",
        "listview.slint",
        "menus.slint",
        "switch.slint",
        "tableview.slint",
        "tabwidget.slint",
    ];

    for control in controls {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let expected_token = match control {
            "components.slint" | "tableview.slint" => {
                "Fluent2SizeSettings.selection-indicator-hidden-height"
            }
            "button.slint" | "checkbox.slint" | "datepicker-base.slint" | "switch.slint" => {
                "FocusTouchArea"
            }
            _ => "Fluent2SizeSettings.interaction-receiver-size",
        };

        assert!(
            source.contains(expected_token),
            "fluent2 {control} should use a token-backed helper for hidden interaction geometry"
        );
        for raw_geometry in ["width: 0;", "height: 0;", "width: 0px;", "height: 0px;"] {
            assert!(
                !source.contains(raw_geometry),
                "fluent2 {control} should not hardcode hidden interaction geometry as {raw_geometry}"
            );
        }
    }
}

mod builtin_library {
    include!(env!("SLINT_WIDGETS_LIBRARY"));

    pub type BuiltinDirectory<'a> = [&'a BuiltinFile<'a>];

    pub struct BuiltinFile<'a> {
        pub path: &'a str,
        pub contents: &'static [u8],
    }

    use super::VirtualFile;

    const ALIASES: &[(&str, &str)] = &[
        ("cosmic-light", "cosmic"),
        ("cosmic-dark", "cosmic"),
        ("fluent-light", "fluent"),
        ("fluent-dark", "fluent"),
        ("fluent2-light", "fluent2"),
        ("fluent2-dark", "fluent2"),
        ("material-light", "material"),
        ("material-dark", "material"),
        ("cupertino-light", "cupertino"),
        ("cupertino-dark", "cupertino"),
        ("retrotone-light", "retrotone"),
        ("retrotone-dark", "retrotone"),
    ];

    pub(crate) fn styles() -> Vec<&'static str> {
        widget_library()
            .iter()
            .filter_map(|(style, directory)| {
                if directory.iter().any(|f| f.path == "std-widgets.slint") {
                    Some(*style)
                } else {
                    None
                }
            })
            .chain(ALIASES.iter().map(|x| x.0))
            .collect()
    }

    pub(crate) fn load_builtin_file(builtin_path: &std::path::Path) -> Option<VirtualFile> {
        let mut components = Vec::new();
        for part in builtin_path.iter() {
            if part == ".." {
                components.pop();
            } else if part != "." {
                components.push(part);
            }
        }
        if let Some(f) = components.first_mut()
            && let Some((_, x)) = ALIASES.iter().find(|x| x.0 == *f)
        {
            *f = std::ffi::OsStr::new(x);
        }
        if let &[folder, file] = components.as_slice() {
            let library = widget_library().iter().find(|x| x.0 == folder)?.1;
            library.iter().find_map(|builtin_file| {
                if builtin_file.path == file {
                    Some(VirtualFile {
                        canon_path: std::path::PathBuf::from(format!(
                            "builtin:/{}/{}",
                            folder.to_str().unwrap(),
                            builtin_file.path
                        )),
                        builtin_contents: Some(builtin_file.contents),
                    })
                } else {
                    None
                }
            })
        } else {
            None
        }
    }
}
