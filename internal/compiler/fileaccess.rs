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
            AboutSlint, Button, CheckBox, ComboBox, DatePickerPopup, GridBox, GroupBox,
            HorizontalBox, LineEdit, ListView, Palette, ProgressIndicator, ScrollView, Slider,
            SpinBox, Spinner, StandardButton, StandardListView, StandardTableView,
            StyleMetrics, Switch, TabWidget, TextEdit, TimePickerPopup, VerticalBox
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
                ScrollView {
                    viewport-width: 96px;
                    viewport-height: 64px;
                    Rectangle { width: 96px; height: 64px; background: Palette.background; }
                }
                HorizontalBox {
                    spacing: StyleMetrics.layout-spacing;
                    Button { text: "Left"; }
                    Button { text: "Right"; }
                }
                GridBox {
                    spacing: StyleMetrics.layout-spacing;
                    Button { row: 0; col: 0; text: "A"; }
                    Button { row: 0; col: 1; text: "B"; }
                }
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

    for expected in [
        "ScrollView {",
        "HorizontalBox {",
        "GridBox {",
        "Palette.background",
        "StyleMetrics.layout-spacing",
    ] {
        assert!(source.contains(expected), "fluent2 representative fixture should use {expected}");
    }

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
fn test_fluent2_render_fixture_covers_representative_controls() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("compiler crate should live under internal/compiler")
        .join("docs/astro/src/fluent2-render-fixtures");

    let controls_path = fixture_root.join("fluent2-controls.md");
    let states_path = fixture_root.join("fluent2-states.md");
    let controls = std::fs::read_to_string(&controls_path)
        .unwrap_or_else(|err| panic!("failed to read {controls_path:?}: {err}"));
    let states = std::fs::read_to_string(&states_path)
        .unwrap_or_else(|err| panic!("failed to read {states_path:?}: {err}"));

    for expected in [
        "imagePath=\"../assets/generated/fluent2-render-fixtures/fluent2-controls.png\"",
        "import { Palette,",
        "Button { text: \"Primary\"; primary: true; }",
        "StandardButton { kind: ok; }",
        "DatePickerPopup { title: \"Date\"; }",
        "TimePickerPopup { title: \"Time\"; }",
        "StandardListView {",
        "StandardTableView {",
        "AboutSlint { width: 280px; height: 80px; }",
    ] {
        assert!(
            controls.contains(expected),
            "fluent2 controls render fixture should include {expected}"
        );
    }

    for expected in [
        "imagePath=\"../assets/generated/fluent2-render-fixtures/fluent2-states.png\"",
        "Button { text: \"Disabled\"; enabled: false; }",
        "LineEdit { text: \"Read only\"; read-only: true; }",
        "TextEdit { text: \"Disabled multiline text\\nkeeps Fluent2 surface and foreground tokens\"; enabled: false; height: 80px; }",
        "ScrollView {",
        "Spinner { indeterminate: true; }",
    ] {
        assert!(
            states.contains(expected),
            "fluent2 states render fixture should include {expected}"
        );
    }
}

#[test]
fn test_fluent2_alias_render_fixtures_cover_light_and_dark_schemes() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("compiler crate should live under internal/compiler")
        .join("docs/astro/src/fluent2-render-fixtures");

    let light_path = fixture_root.join("fluent2-states-light.md");
    let dark_path = fixture_root.join("fluent2-states-dark.md");
    let light = std::fs::read_to_string(&light_path)
        .unwrap_or_else(|err| panic!("failed to read {light_path:?}: {err}"));
    let dark = std::fs::read_to_string(&dark_path)
        .unwrap_or_else(|err| panic!("failed to read {dark_path:?}: {err}"));

    for (source, scheme, image) in [
        (&light, "fluent2-light", "fluent2-states-light.png"),
        (&dark, "fluent2-dark", "fluent2-states-dark.png"),
    ] {
        assert!(
            source.contains(&format!(
                "imagePath=\"../assets/generated/fluent2-render-fixtures/{image}\""
            )),
            "{scheme} render fixture should write to its scheme-specific image"
        );
        assert!(source.contains("import { Palette,"), "{scheme} fixture should use std widgets");
        assert!(
            source.contains("Button { text: \"Primary\"; primary: true; }"),
            "{scheme} fixture should cover primary button rendering"
        );
        assert!(
            source.contains("ScrollView {"),
            "{scheme} fixture should cover direct ScrollView rendering"
        );
    }
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
fn test_fluent2_style_metrics_textedit_background_matches_input_surface() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/style-base.slint"))
        .expect("fluent2 should embed style-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("textedit-background: Fluent2Palette.control-background"),
        "fluent2 StyleMetrics textedit-background should match the Fluent2 text input resting surface"
    );
    assert!(
        !source.contains("textedit-background: Fluent2Palette.background"),
        "fluent2 StyleMetrics textedit-background should not report the page/window background"
    );

    assert!(
        source.contains(
            "textedit-background-disabled: Fluent2Palette.text-input-disabled-background"
        ),
        "fluent2 StyleMetrics textedit-background-disabled should match the Fluent2 disabled text input surface"
    );
    assert!(
        !source.contains("textedit-background-disabled: Fluent2Palette.control-disabled"),
        "fluent2 StyleMetrics textedit-background-disabled should not report the generic disabled control primitive"
    );

    assert!(
        source.contains("textedit-text-color: Fluent2Palette.text-input-foreground"),
        "fluent2 StyleMetrics textedit-text-color should match the Fluent2 text input foreground"
    );
    assert!(
        !source.contains("textedit-text-color: Fluent2Palette.foreground"),
        "fluent2 StyleMetrics textedit-text-color should not report the generic foreground primitive"
    );

    assert!(
        source.contains(
            "textedit-text-color-disabled: Fluent2Palette.text-input-disabled-foreground"
        ),
        "fluent2 StyleMetrics textedit-text-color-disabled should match the Fluent2 disabled text input foreground"
    );
    assert!(
        !source.contains("textedit-text-color-disabled: Fluent2Palette.text-disabled"),
        "fluent2 StyleMetrics textedit-text-color-disabled should not report the generic disabled text primitive"
    );
}

#[test]
fn test_fluent2_disabled_input_borders_use_disabled_stroke_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in
        ["text-input-disabled-border", "spinbox-disabled-border", "combobox-disabled-border"]
    {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for (control, component_marker, disabled_marker, expected_border) in [
        (
            "lineedit.slint",
            "export component LineEdit",
            "disabled when !root.enabled",
            "background.border-color: Fluent2Palette.text-input-disabled-border",
        ),
        (
            "textedit.slint",
            "export component TextEdit",
            "disabled when !root.enabled",
            "base.border-color: Fluent2Palette.text-input-disabled-border",
        ),
        (
            "spinbox.slint",
            "export component SpinBox",
            "disabled when !root.enabled",
            "background.border-color: Fluent2Palette.spinbox-disabled-border",
        ),
        (
            "combobox.slint",
            "export component ComboBox",
            "disabled when !root.enabled",
            "background.border-color: Fluent2Palette.combobox-disabled-border",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let component = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {control} should define {component_marker}"));
        let disabled_state = component
            .split(disabled_marker)
            .nth(1)
            .and_then(|after| after.split("}").next())
            .unwrap_or_else(|| panic!("fluent2 {control} should define disabled state"));

        assert!(
            disabled_state.contains(expected_border),
            "fluent2 {control} disabled state should use the semantic disabled border token"
        );
        assert!(
            !disabled_state.contains("border-color: Fluent2Palette.border"),
            "fluent2 {control} disabled state should not reuse the normal border token"
        );
        assert!(
            !disabled_state.contains("Fluent2Palette.control-strong-stroke-disabled"),
            "fluent2 {control} disabled state should not bind its border directly to the generic disabled strong stroke token"
        );
    }
}

#[test]
fn test_fluent2_disabled_border_tokens_use_disabled_stroke_primitive() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains(
            "property <brush> neutral-stroke-disabled: dark-color-scheme ? #FFFFFF29 : #00000038"
        ),
        "fluent2 styling should centralize disabled stroke color in a neutral-stroke-disabled primitive"
    );

    for expected in [
        "combobox-disabled-border: neutral-stroke-disabled",
        "switch-rail-disabled-border: neutral-stroke-disabled",
        "text-input-disabled-border: neutral-stroke-disabled",
        "spinbox-disabled-border: neutral-stroke-disabled",
        "control-strong-stroke-disabled: neutral-stroke-disabled",
        "checkbox-disabled-border: neutral-stroke-disabled",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 disabled border semantic token should use neutral-stroke-disabled: {expected}"
        );
    }

    for copied_literal in [
        "combobox-disabled-border: dark-color-scheme ? #FFFFFF29 : #00000038",
        "switch-rail-disabled-border: dark-color-scheme ? #FFFFFF29 : #00000038",
        "text-input-disabled-border: dark-color-scheme ? #FFFFFF29 : #00000038",
        "spinbox-disabled-border: dark-color-scheme ? #FFFFFF29 : #00000038",
        "control-strong-stroke-disabled: dark-color-scheme ? #FFFFFF29 : #00000038",
        "checkbox-disabled-border: dark-color-scheme ? #FFFFFF29 : #00000038",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 disabled border semantic token should not repeat disabled stroke literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_strong_border_tokens_use_strong_stroke_primitive() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains(
            "property <brush> neutral-stroke-strong: dark-color-scheme ? #FFFFFF99 : #00000099"
        ),
        "fluent2 styling should centralize strong neutral stroke color in neutral-stroke-strong"
    );

    for expected in [
        "switch-rail-border: neutral-stroke-strong",
        "control-strong-stroke: neutral-stroke-strong",
        "checkbox-border: neutral-stroke-strong",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 strong border semantic token should use neutral-stroke-strong: {expected}"
        );
    }

    for copied_literal in [
        "switch-rail-border: dark-color-scheme ? #FFFFFF99 : #00000099",
        "control-strong-stroke: dark-color-scheme ? #FFFFFF99 : #00000099",
        "checkbox-border: dark-color-scheme ? #FFFFFF99 : #00000099",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 strong border semantic token should not repeat strong stroke literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_circle_border_tokens_use_circle_stroke_primitive() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains(
            "property <brush> neutral-stroke-circle: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #FFFFFF12 100%) : @linear-gradient(180deg, #0000000F 0%, #00000029 100%)"
        ),
        "fluent2 styling should centralize circular neutral stroke gradients in neutral-stroke-circle"
    );

    for expected in [
        "switch-thumb-border: neutral-stroke-circle",
        "slider-thumb-border: neutral-stroke-circle",
        "circle-border: neutral-stroke-circle",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 circular border semantic token should use neutral-stroke-circle: {expected}"
        );
    }

    for copied_literal in [
        "switch-thumb-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #FFFFFF12 100%) : @linear-gradient(180deg, #0000000F 0%, #00000029 100%)",
        "slider-thumb-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #FFFFFF12 100%) : @linear-gradient(180deg, #0000000F 0%, #00000029 100%)",
        "circle-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #FFFFFF12 100%) : @linear-gradient(180deg, #0000000F 0%, #00000029 100%)",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 circular border semantic token should not repeat circular stroke gradients: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_pressed_accent_foregrounds_use_pressed_accent_primitive() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains(
            "property <brush> accent-foreground-pressed: dark-color-scheme ? #00000080 : #FFFFFFB3"
        ),
        "fluent2 styling should centralize pressed accent foreground color in accent-foreground-pressed"
    );

    for expected in [
        "button-primary-pressed-foreground: accent-foreground-pressed",
        "text-accent-foreground-secondary: accent-foreground-pressed",
        "checkbox-checkmark-pressed-foreground: accent-foreground-pressed",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 pressed accent foreground semantic token should use accent-foreground-pressed: {expected}"
        );
    }

    for copied_literal in [
        "button-primary-pressed-foreground: dark-color-scheme ? #00000080 : #FFFFFFB3",
        "text-accent-foreground-secondary: dark-color-scheme ? #00000080 : #FFFFFFB3",
        "checkbox-checkmark-pressed-foreground: dark-color-scheme ? #00000080 : #FFFFFFB3",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 pressed accent foreground semantic token should not repeat pressed accent foreground literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_surface_bridge_tokens_use_surface_primitives() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "property <brush> surface-layer-fill-alt: dark-color-scheme ? #3A3A3A73 : #FFFFFFB3;",
        "property <brush> surface-card-stroke: dark-color-scheme ? #0000001A : #0000000F;",
        "out property <brush> layer-on-mica-base-alt: surface-layer-fill-alt;",
        "out property <brush> card-stroke: surface-card-stroke;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 surface bridge token should use a surface primitive: {expected}"
        );
    }

    for copied_literal in [
        "out property <brush> layer-on-mica-base-alt: dark-color-scheme ? #3A3A3A73 : #FFFFFFB3",
        "out property <brush> card-stroke: dark-color-scheme ? #0000001A : #0000000F",
    ] {
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal),
            "fluent2 surface bridge token should not repeat surface primitive literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_gradient_border_tokens_use_border_primitives() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "property <brush> accent-stroke-control: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF14 90.67%, #00000024 100%) : @linear-gradient(180deg, #FFFFFF14 90.67%, #00000066 100%);",
        "property <brush> neutral-stroke-control: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #00000012 8.33%) : @linear-gradient(180deg, #0000000F 90.58%, #00000029 100%);",
        "property <brush> neutral-stroke-control-active: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF14 99.98%, #FFFFFF8A 100%, #FFFFFF8A 100%) : @linear-gradient(180deg, #0000000F 99.99%, #00000073 100%, #00000073 100%);",
        "out property <brush> accent-control-border: accent-stroke-control;",
        "out property <brush> button-primary-border: accent-stroke-control;",
        "out property <brush> control-border: neutral-stroke-control;",
        "out property <brush> text-control-border: neutral-stroke-control-active;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 gradient border token should use a named border primitive: {expected}"
        );
    }

    for copied_literal in [
        "out property <brush> accent-control-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF14 90.67%, #00000024 100%) : @linear-gradient(180deg, #FFFFFF14 90.67%, #00000066 100%)",
        "out property <brush> button-primary-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF14 90.67%, #00000024 100%) : @linear-gradient(180deg, #FFFFFF14 90.67%, #00000066 100%)",
        "out property <brush> control-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF17 0%, #00000012 8.33%) : @linear-gradient(180deg, #0000000F 90.58%, #00000029 100%)",
        "out property <brush> text-control-border: dark-color-scheme ? @linear-gradient(180deg, #FFFFFF14 99.98%, #FFFFFF8A 100%, #FFFFFF8A 100%) : @linear-gradient(180deg, #0000000F 99.99%, #00000073 100%, #00000073 100%)",
    ] {
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal),
            "fluent2 gradient border token should not repeat border primitive gradients: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_disabled_input_surfaces_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "text-input-disabled-background",
        "spinbox-disabled-background",
        "combobox-disabled-background",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for (control, component_marker, disabled_marker, expected_background) in [
        (
            "lineedit.slint",
            "export component LineEdit",
            "disabled when !root.enabled",
            "background.background: Fluent2Palette.text-input-disabled-background",
        ),
        (
            "textedit.slint",
            "export component TextEdit",
            "disabled when !root.enabled",
            "base.background: Fluent2Palette.text-input-disabled-background",
        ),
        (
            "spinbox.slint",
            "export component SpinBox",
            "disabled when !root.enabled",
            "background.background: Fluent2Palette.spinbox-disabled-background",
        ),
        (
            "combobox.slint",
            "export component ComboBox",
            "disabled when !root.enabled",
            "background.background: Fluent2Palette.combobox-disabled-background",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let component = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {control} should define {component_marker}"));
        let disabled_state = component
            .split(disabled_marker)
            .nth(1)
            .and_then(|after| after.split("}").next())
            .unwrap_or_else(|| panic!("fluent2 {control} should define disabled state"));

        assert!(
            disabled_state.contains(expected_background),
            "fluent2 {control} disabled state should use the semantic disabled background token"
        );
        assert!(
            !disabled_state.contains("Fluent2Palette.control-disabled"),
            "fluent2 {control} disabled state should not bind its surface directly to the generic control-disabled token"
        );
    }
}

#[test]
fn test_fluent2_combobox_border_uses_semantic_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("combobox-border"),
        "fluent2 styling should expose a combobox border token"
    );
    assert!(
        styling.contains("combobox-border-width"),
        "fluent2 styling should expose a combobox border width token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/combobox.slint"))
        .expect("fluent2 should embed combobox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let combobox =
        source.split("export component ComboBox").nth(1).expect("fluent2 should define ComboBox");

    assert!(
        combobox.contains("Fluent2Palette.combobox-border"),
        "fluent2 ComboBox should use the semantic combobox border token"
    );
    assert!(
        combobox.contains("border-color: Fluent2Palette.combobox-border"),
        "fluent2 ComboBox resting border should use the semantic combobox border token"
    );
    assert!(
        combobox.contains("border-width: Fluent2SizeSettings.combobox-border-width"),
        "fluent2 ComboBox resting border should use the semantic combobox border width token"
    );
    assert!(
        !combobox.contains("Fluent2Palette.border"),
        "fluent2 ComboBox should not use the generic border bridge token for combobox-specific state borders"
    );
    assert!(
        !combobox.contains("Fluent2Palette.control-border"),
        "fluent2 ComboBox should not bind state borders directly to the generic control-border token"
    );
    assert!(
        !combobox.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 ComboBox should not bind border width directly to the generic stroke-width token"
    );
}

#[test]
fn test_fluent2_combobox_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "combobox-background",
        "combobox-hover-background",
        "combobox-pressed-background",
        "combobox-foreground",
        "combobox-disabled-foreground",
        "combobox-pressed-foreground",
        "combobox-disabled-icon-foreground",
        "combobox-radius",
        "combobox-icon-size",
        "combobox-min-width",
        "combobox-height",
        "combobox-horizontal-padding",
        "combobox-content-spacing",
        "combobox-popup-padding",
        "combobox-popup-item-height",
        "combobox-motion-duration",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "combobox-background: control-background",
        "combobox-disabled-background: control-disabled",
        "combobox-disabled-border: control-strong-stroke-disabled",
        "combobox-hover-background: control-secondary",
        "combobox-foreground: control-foreground",
        "combobox-disabled-foreground: text-disabled",
        "combobox-pressed-foreground: text-secondary",
        "combobox-disabled-icon-foreground: text-disabled",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 combobox semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/combobox.slint"))
        .expect("fluent2 should embed combobox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let combobox =
        source.split("export component ComboBox").nth(1).expect("fluent2 should define ComboBox");

    for expected in [
        "background.background: Fluent2Palette.combobox-hover-background",
        "background.background: Fluent2Palette.combobox-pressed-background",
        "background: Fluent2Palette.combobox-background",
        "color: Fluent2Palette.combobox-foreground",
        "text.color: Fluent2Palette.combobox-disabled-foreground",
        "icon.colorize: Fluent2Palette.combobox-disabled-icon-foreground",
        "text.color: Fluent2Palette.combobox-pressed-foreground",
        "border-radius: Fluent2SizeSettings.combobox-radius",
        "width: Fluent2SizeSettings.combobox-icon-size",
        "min-width: max(Fluent2SizeSettings.combobox-min-width, layout.min-width)",
        "min-height: max(Fluent2SizeSettings.combobox-height, layout.min-height)",
        "padding-left: Fluent2SizeSettings.combobox-horizontal-padding",
        "padding-right: Fluent2SizeSettings.combobox-horizontal-padding",
        "spacing: Fluent2SizeSettings.combobox-content-spacing",
        "property <length> popup-padding: Fluent2SizeSettings.combobox-popup-padding",
        "height: root.visible-items * Fluent2SizeSettings.combobox-popup-item-height +  2 * root.popup-padding",
        "animate border-color { duration: Fluent2SizeSettings.combobox-motion-duration",
        "animate colorize { duration: Fluent2SizeSettings.combobox-motion-duration",
    ] {
        assert!(combobox.contains(expected), "fluent2 ComboBox should use {expected}");
    }

    for copied_literal in [
        "background.background: Fluent2Palette.control-secondary",
        "background.background: Fluent2Palette.control-alt-tertiary",
        "background: Fluent2Palette.control-background",
        "text.color: Fluent2Palette.text-disabled",
        "icon.colorize: Fluent2Palette.text-disabled",
        "text.color: Fluent2Palette.text-secondary",
        "color: Fluent2Palette.control-foreground",
        "border-radius: Fluent2SizeSettings.control-radius",
        "width: Fluent2SizeSettings.icon-size",
        "min-width: max(Fluent2SizeSettings.input-min-width, layout.min-height)",
        "min-height: max(Fluent2SizeSettings.control-height, layout.min-height)",
        "padding-left: Fluent2SizeSettings.control-horizontal-padding",
        "padding-right: Fluent2SizeSettings.control-horizontal-padding",
        "spacing: Fluent2SizeSettings.control-spacing",
        "property <length> popup-padding: Fluent2SizeSettings.overlay-padding",
        "height: root.visible-items * Fluent2SizeSettings.item-height +  2 * root.popup-padding",
        "duration: Fluent2SizeSettings.control-motion-duration",
    ] {
        assert!(
            !combobox.contains(copied_literal),
            "fluent2 ComboBox should not bind active colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_groupbox_title_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["groupbox-title-foreground", "groupbox-title-disabled-foreground"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "groupbox-title-foreground: control-foreground",
        "groupbox-title-disabled-foreground: text-disabled",
    ] {
        assert!(
            !styling.contains(copied_bridge),
            "fluent2 GroupBox title semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/groupbox.slint"))
        .expect("fluent2 should embed groupbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let groupbox =
        source.split("export component GroupBox").nth(1).expect("fluent2 should define GroupBox");

    assert!(
        groupbox.contains(
            "color: !root.enabled ? Fluent2Palette.groupbox-title-disabled-foreground : Fluent2Palette.groupbox-title-foreground"
        ),
        "fluent2 GroupBox title should use semantic title foreground tokens"
    );

    for copied_literal in [
        "Fluent2Palette.text-disabled : Fluent2Palette.control-foreground",
        "Fluent2Palette.foreground",
    ] {
        assert!(
            !groupbox.contains(copied_literal),
            "fluent2 GroupBox title should not bind directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_checkbox_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "checkbox-foreground",
        "checkbox-disabled-foreground",
        "checkbox-background",
        "checkbox-checked-background",
        "checkbox-checked-hover-background",
        "checkbox-checked-pressed-background",
        "checkbox-disabled-background",
        "checkbox-checked-disabled-background",
        "checkbox-hover-background",
        "checkbox-pressed-background",
        "checkbox-border",
        "checkbox-border-width",
        "checkbox-disabled-border",
        "checkbox-checkmark-foreground",
        "checkbox-checkmark-disabled-foreground",
        "checkbox-checkmark-pressed-foreground",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "checkbox-foreground: foreground",
        "checkbox-disabled-foreground: text-disabled",
        "checkbox-checked-background: accent-background",
        "checkbox-checked-hover-background: secondary-accent-background",
        "checkbox-checked-pressed-background: tertiary-accent-background",
        "checkbox-disabled-background: control-alt-disabled",
        "checkbox-checked-disabled-background: accent-disabled",
        "checkbox-border: control-strong-stroke",
        "checkbox-disabled-border: control-strong-stroke-disabled",
        "checkbox-checkmark-foreground: accent-foreground",
        "checkbox-checkmark-disabled-foreground: text-accent-foreground-disabled",
        "checkbox-checkmark-pressed-foreground: text-accent-foreground-secondary",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind checkbox semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    for expected in [
        "checkbox-background: control-alt-secondary",
        "checkbox-hover-background: control-alt-tertiary",
        "checkbox-pressed-background: control-alt-quaternary",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 checkbox alternate fills should use the corrected control-alt primitive path: {expected}"
        );
    }

    for copied_literal in [
        "checkbox-background: dark-color-scheme ? #0000001A : #00000005",
        "checkbox-hover-background: dark-color-scheme ? #FFFFFF0A : #0000000F",
        "checkbox-pressed-background: dark-color-scheme ? #FFFFFF12 : #00000017",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 checkbox alternate fills should not repeat copied raw light/dark alpha branches: {copied_literal}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let checkbox =
        source.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    for expected in [
        "private property <color> text-color: Fluent2Palette.checkbox-foreground",
        "border.border-color: Fluent2Palette.checkbox-disabled-border",
        "background.background: root.checked ? Fluent2Palette.checkbox-checked-disabled-background : Fluent2Palette.checkbox-disabled-background",
        "icon.colorize: Fluent2Palette.checkbox-checkmark-disabled-foreground",
        "root.text-color: Fluent2Palette.checkbox-disabled-foreground",
        "background.background: root.checked ? Fluent2Palette.checkbox-checked-pressed-background : Fluent2Palette.checkbox-pressed-background",
        "icon.colorize: Fluent2Palette.checkbox-checkmark-pressed-foreground",
        "background.background: root.checked ?  Fluent2Palette.checkbox-checked-hover-background : Fluent2Palette.checkbox-hover-background",
        "background.background: Fluent2Palette.checkbox-checked-background",
        "background: Fluent2Palette.checkbox-background",
        "border-color: Fluent2Palette.checkbox-border",
        "border-width: root.checked ? Fluent2SizeSettings.checkbox-hidden-border-width : Fluent2SizeSettings.checkbox-border-width",
        "colorize: Fluent2Palette.checkbox-checkmark-foreground",
    ] {
        assert!(checkbox.contains(expected), "fluent2 CheckBox should use {expected}");
    }

    for copied_literal in [
        "private property <color> text-color: Fluent2Palette.foreground",
        "border.border-color: Fluent2Palette.control-strong-stroke-disabled",
        "background.background: root.checked ? Fluent2Palette.accent-disabled : Fluent2Palette.control-alt-disabled",
        "icon.colorize: Fluent2Palette.text-accent-foreground-disabled",
        "root.text-color: Fluent2Palette.text-disabled",
        "background.background: root.checked ? Fluent2Palette.tertiary-accent-background : Fluent2Palette.control-alt-quaternary",
        "icon.colorize: Fluent2Palette.text-accent-foreground-secondary",
        "background.background: root.checked ?  Fluent2Palette.secondary-accent-background : Fluent2Palette.control-alt-tertiary",
        "background.background: Fluent2Palette.accent-background",
        "background: Fluent2Palette.control-alt-secondary",
        "border-color: Fluent2Palette.control-strong-stroke",
        "border-width: root.checked ? Fluent2SizeSettings.control-hidden-stroke-width : Fluent2SizeSettings.checkbox-border-width",
        "border-width: root.checked ? Fluent2SizeSettings.control-hidden-stroke-width : Fluent2SizeSettings.stroke-width",
        "colorize: Fluent2Palette.accent-foreground",
    ] {
        assert!(
            !checkbox.contains(copied_literal),
            "fluent2 CheckBox should not bind colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_checkbox_and_switch_label_spacing_use_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["checkbox-label-spacing", "switch-label-spacing"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let checkbox = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let checkbox_contents = checkbox.read();
    let checkbox = std::str::from_utf8(&checkbox_contents).unwrap();
    let checkbox =
        checkbox.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    let switch = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let switch_contents = switch.read();
    let switch = std::str::from_utf8(&switch_contents).unwrap();
    let switch =
        switch.split("export component Switch").nth(1).expect("fluent2 should define Switch");

    assert!(
        checkbox.contains("spacing: Fluent2SizeSettings.checkbox-label-spacing"),
        "fluent2 CheckBox label spacing should use its own token"
    );
    assert!(
        switch.contains("spacing: Fluent2SizeSettings.switch-label-spacing"),
        "fluent2 Switch label spacing should use its own token"
    );

    for (control_name, source) in [("CheckBox", checkbox), ("Switch", switch)] {
        assert!(
            !source.contains("spacing: Fluent2SizeSettings.control-label-spacing"),
            "fluent2 {control_name} should not use copied generic control label spacing"
        );
    }
}

#[test]
fn test_fluent2_checkbox_checkmark_size_uses_semantic_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("checkbox-checkmark-size"),
        "fluent2 styling should expose checkbox-checkmark-size"
    );

    let checkbox = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let checkbox_contents = checkbox.read();
    let checkbox = std::str::from_utf8(&checkbox_contents).unwrap();
    let checkbox =
        checkbox.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    assert!(
        checkbox.contains("width: Fluent2SizeSettings.checkbox-checkmark-size"),
        "fluent2 CheckBox checkmark width should use its own semantic size token"
    );
    assert!(
        !checkbox.contains("width: Fluent2SizeSettings.icon-size"),
        "fluent2 CheckBox checkmark should not use the generic icon-size token"
    );
}

#[test]
fn test_fluent2_checkbox_focus_radius_uses_semantic_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("checkbox-focus-radius"),
        "fluent2 styling should expose checkbox-focus-radius"
    );

    let checkbox = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let checkbox_contents = checkbox.read();
    let checkbox = std::str::from_utf8(&checkbox_contents).unwrap();
    let checkbox =
        checkbox.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    assert!(
        checkbox.contains("border-radius: Fluent2SizeSettings.checkbox-focus-radius"),
        "fluent2 CheckBox focus border should use its own semantic radius token"
    );
    assert!(
        !checkbox.contains("border-radius: Fluent2SizeSettings.button-radius"),
        "fluent2 CheckBox focus border should not borrow the button radius token"
    );
}

#[test]
fn test_fluent2_checkbox_motion_uses_checkbox_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("checkbox-motion-duration"),
        "fluent2 styling should expose checkbox-motion-duration"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/checkbox.slint"))
        .expect("fluent2 should embed checkbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let checkbox =
        source.split("export component CheckBox").nth(1).expect("fluent2 should define CheckBox");

    for expected in [
        "animate text-color { duration: Fluent2SizeSettings.checkbox-motion-duration",
        "animate background { duration: Fluent2SizeSettings.checkbox-motion-duration",
        "animate border-color { duration: Fluent2SizeSettings.checkbox-motion-duration",
        "animate colorize { duration: Fluent2SizeSettings.checkbox-motion-duration",
    ] {
        assert!(checkbox.contains(expected), "fluent2 CheckBox should use {expected}");
    }

    assert!(
        !checkbox.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 CheckBox should not use the generic control motion token"
    );
}

#[test]
fn test_fluent2_focused_text_inputs_keep_text_control_border_token() {
    for (control, component_marker, focused_marker, expected_border) in [
        (
            "lineedit.slint",
            "export component LineEdit",
            "focused when root.has-focus",
            "background.border-color: Fluent2Palette.text-control-border",
        ),
        (
            "textedit.slint",
            "export component TextEdit",
            "focused when root.has-focus",
            "base.border-color: Fluent2Palette.text-control-border",
        ),
        (
            "spinbox.slint",
            "export component SpinBox",
            "focused when root.has-focus",
            "background.border-color: Fluent2Palette.text-control-border",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let component = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {control} should define {component_marker}"));
        let focused_state = component
            .split(focused_marker)
            .nth(1)
            .and_then(|after| after.split("}").next())
            .unwrap_or_else(|| panic!("fluent2 {control} should define focused state"));

        assert!(
            focused_state.contains(expected_border),
            "fluent2 {control} focused state should keep the text-control border token"
        );
        assert!(
            !focused_state.contains("border-color: Fluent2Palette.border"),
            "fluent2 {control} focused state should not replace the text-control stroke with generic border"
        );
    }
}

#[test]
fn test_fluent2_text_inputs_use_semantic_color_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "text-input-background",
        "text-input-active-background",
        "text-input-foreground",
        "text-input-disabled-foreground",
        "text-input-placeholder-foreground",
        "text-input-focused-placeholder-foreground",
        "text-input-focus-indicator-background",
        "text-input-selection-background",
        "text-input-selection-foreground",
        "text-input-disabled-selection-foreground",
        "text-input-min-width",
        "text-input-height",
        "text-input-horizontal-padding",
        "text-input-vertical-padding",
        "text-input-border-width",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "text-input-background: control-background",
        "text-input-active-background: control-input-active",
        "text-input-disabled-background: control-disabled",
        "text-input-disabled-border: control-strong-stroke-disabled",
        "text-input-foreground: foreground",
        "text-input-disabled-foreground: text-disabled",
        "text-input-placeholder-foreground: text-secondary",
        "text-input-focused-placeholder-foreground: text-tertiary",
        "text-input-focus-indicator-background: accent-background",
        "text-input-selection-background: selection-background",
        "text-input-selection-foreground: accent-foreground",
        "text-input-disabled-selection-foreground: text-accent-foreground-disabled",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 text input semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    for (control, expected, copied_literals) in [
        (
            "lineedit.slint",
            vec![
                "background: Fluent2Palette.text-input-background",
                "background.background: Fluent2Palette.text-input-active-background",
                "focus-border.background: Fluent2Palette.text-input-focus-indicator-background",
                "base.text-color: Fluent2Palette.text-input-disabled-foreground",
                "base.placeholder-color: Fluent2Palette.text-input-disabled-foreground",
                "base.placeholder-color: Fluent2Palette.text-input-focused-placeholder-foreground",
                "selection-background-color: Fluent2Palette.text-input-selection-background",
                "selection-foreground-color: Fluent2Palette.text-input-selection-foreground",
                "text-color: Fluent2Palette.text-input-foreground",
                "placeholder-color: Fluent2Palette.text-input-placeholder-foreground",
                "base.selection-foreground-color: Fluent2Palette.text-input-disabled-selection-foreground",
                "min-width: max(Fluent2SizeSettings.text-input-min-width, layout.min-width)",
                "min-height: max(Fluent2SizeSettings.text-input-height, layout.min-height)",
                "padding-left: Fluent2SizeSettings.text-input-horizontal-padding",
                "padding-right: Fluent2SizeSettings.text-input-horizontal-padding",
                "border-width: Fluent2SizeSettings.text-input-border-width",
            ],
            vec![
                "background: Fluent2Palette.control-background",
                "background.background: Fluent2Palette.control-input-active",
                "focus-border.background: Fluent2Palette.accent-background",
                "base.text-color: Fluent2Palette.text-disabled",
                "base.placeholder-color: Fluent2Palette.text-disabled",
                "base.placeholder-color: Fluent2Palette.text-tertiary",
                "selection-background-color: Fluent2Palette.selection-background",
                "selection-foreground-color: Fluent2Palette.accent-foreground",
                "text-color: Fluent2Palette.foreground",
                "placeholder-color: Fluent2Palette.text-secondary",
                "base.selection-foreground-color: Fluent2Palette.text-accent-foreground-disabled",
                "min-width: max(Fluent2SizeSettings.input-min-width, layout.min-width)",
                "min-height: max(Fluent2SizeSettings.control-height, layout.min-height)",
                "padding-left: Fluent2SizeSettings.control-horizontal-padding",
                "padding-right: Fluent2SizeSettings.control-horizontal-padding",
                "border-width: Fluent2SizeSettings.stroke-width",
            ],
        ),
        (
            "textedit.slint",
            vec![
                "background: Fluent2Palette.text-input-background",
                "base.background: Fluent2Palette.text-input-active-background",
                "i-focus-border.background: Fluent2Palette.text-input-focus-indicator-background",
                "base.foreground: Fluent2Palette.text-input-disabled-foreground",
                "foreground: Fluent2Palette.text-input-foreground",
                "placeholder-color: Fluent2Palette.text-input-placeholder-foreground",
                "selection-background-color: Fluent2Palette.text-input-selection-background",
                "selection-foreground-color: Fluent2Palette.text-input-selection-foreground",
                "base.selection-foreground-color: Fluent2Palette.text-input-disabled-selection-foreground",
                "scroll-view-padding: Fluent2SizeSettings.text-input-horizontal-padding",
                "cursor-margin: Fluent2SizeSettings.text-input-horizontal-padding",
                "border-width: Fluent2SizeSettings.text-input-border-width",
            ],
            vec![
                "background: Fluent2Palette.control-background",
                "base.background: Fluent2Palette.control-input-active",
                "i-focus-border.background: Fluent2Palette.accent-background",
                "base.foreground: Fluent2Palette.text-disabled",
                "foreground: Fluent2Palette.foreground",
                "placeholder-color: Fluent2Palette.text-secondary",
                "selection-background-color: Fluent2Palette.selection-background",
                "selection-foreground-color: Fluent2Palette.selection-foreground",
                "base.selection-foreground-color: Fluent2Palette.text-accent-foreground-disabled",
                "scroll-view-padding: Fluent2SizeSettings.control-horizontal-padding",
                "cursor-margin: Fluent2SizeSettings.control-horizontal-padding",
                "border-width: Fluent2SizeSettings.stroke-width",
            ],
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        for expected in expected {
            assert!(source.contains(expected), "fluent2 {control} should use {expected}");
        }

        for copied_literal in copied_literals {
            assert!(
                !source.contains(copied_literal),
                "fluent2 {control} should not bind text input colors directly to copied generic token {copied_literal}"
            );
        }
    }
}

#[test]
fn test_fluent2_text_input_focus_indicator_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "input-focus-indicator-height",
        "text-input-focus-indicator-height",
        "spinbox-focus-indicator-height",
        "text-input-focus-indicator-horizontal-inset",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for (control, expected_height) in [
        ("lineedit.slint", "Fluent2SizeSettings.text-input-focus-indicator-height"),
        ("textedit.slint", "Fluent2SizeSettings.text-input-focus-indicator-height"),
        ("spinbox.slint", "Fluent2SizeSettings.spinbox-focus-indicator-height"),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains("Fluent2SizeSettings.text-input-focus-indicator-horizontal-inset"),
            "fluent2 {control} focus indicator should use the horizontal inset token"
        );
        assert!(
            source.contains(expected_height),
            "fluent2 {control} focus indicator height should use semantic component token {expected_height}"
        );
        assert!(
            !source.contains("Fluent2SizeSettings.input-focus-indicator-height"),
            "fluent2 {control} focus indicator should not bind directly to the broad input focus height token"
        );
        assert!(
            !source.contains("x: parent.border-radius")
                && !source.contains("width: parent.width - 2 * parent.border-radius"),
            "fluent2 {control} focus indicator should not derive visual inset from live border radius"
        );
    }
}

#[test]
fn test_fluent2_text_entry_radius_uses_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["text-input-radius", "spinbox-radius"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for (control, surface_marker, end_marker, expected, copied_literal) in [
        (
            "lineedit.slint",
            "export component LineEdit",
            "layout := HorizontalLayout",
            "border-radius: Fluent2SizeSettings.text-input-radius",
            "border-radius: Fluent2SizeSettings.button-radius",
        ),
        (
            "textedit.slint",
            "base := TextEditBase",
            "i-focus-border := Rectangle",
            "border-radius: Fluent2SizeSettings.text-input-radius",
            "border-radius: Fluent2SizeSettings.button-radius",
        ),
        (
            "spinbox.slint",
            "export component SpinBox",
            "layout := HorizontalLayout",
            "border-radius: Fluent2SizeSettings.spinbox-radius",
            "border-radius: Fluent2SizeSettings.control-radius",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let surface_block = source
            .split(surface_marker)
            .nth(1)
            .and_then(|after| after.split(end_marker).next())
            .unwrap_or_else(|| panic!("fluent2 {control} should define {surface_marker}"));

        assert!(surface_block.contains(expected), "fluent2 {control} should use {expected}");
        assert!(
            !surface_block.contains(copied_literal),
            "fluent2 {control} text-entry surface should not borrow copied radius token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_spinbox_input_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "spinbox-background",
        "spinbox-active-background",
        "spinbox-foreground",
        "spinbox-disabled-foreground",
        "spinbox-focus-indicator-background",
        "spinbox-selection-background",
        "spinbox-selection-foreground",
        "spinbox-disabled-selection-foreground",
        "spinbox-height",
        "spinbox-horizontal-padding",
        "spinbox-button-column-padding",
        "spinbox-vertical-padding",
        "spinbox-border-width",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "spinbox-background: control-background",
        "spinbox-active-background: control-input-active",
        "spinbox-disabled-background: control-disabled",
        "spinbox-disabled-border: control-strong-stroke-disabled",
        "spinbox-foreground: control-foreground",
        "spinbox-disabled-foreground: text-disabled",
        "spinbox-focus-indicator-background: accent-background",
        "spinbox-selection-background: selection-background",
        "spinbox-selection-foreground: accent-foreground",
        "spinbox-disabled-selection-foreground: text-accent-foreground-disabled",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 spinbox semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinbox.slint"))
        .expect("fluent2 should embed spinbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let spinbox =
        source.split("export component SpinBox").nth(1).expect("fluent2 should define SpinBox");

    for expected in [
        "background.background: Fluent2Palette.spinbox-active-background",
        "focus-border.background: Fluent2Palette.spinbox-focus-indicator-background",
        "background: Fluent2Palette.spinbox-background",
        "base.color: Fluent2Palette.spinbox-disabled-foreground",
        "color: Fluent2Palette.spinbox-foreground",
        "selection-background-color: Fluent2Palette.spinbox-selection-background",
        "selection-foreground-color: Fluent2Palette.spinbox-selection-foreground",
        "base.selection-foreground-color: Fluent2Palette.spinbox-disabled-selection-foreground",
        "min-height: max(Fluent2SizeSettings.spinbox-height, layout.min-height)",
        "padding-left: Fluent2SizeSettings.spinbox-horizontal-padding",
        "padding-right: Fluent2SizeSettings.spinbox-button-column-padding",
        "padding-top: Fluent2SizeSettings.spinbox-vertical-padding",
        "padding-bottom: Fluent2SizeSettings.spinbox-vertical-padding",
        "border-width: Fluent2SizeSettings.spinbox-border-width",
    ] {
        assert!(spinbox.contains(expected), "fluent2 SpinBox should use {expected}");
    }

    for copied_literal in [
        "background.background: Fluent2Palette.control-input-active",
        "focus-border.background: Fluent2Palette.accent-background",
        "background: Fluent2Palette.control-background",
        "base.color: Fluent2Palette.text-disabled",
        "color: Fluent2Palette.control-foreground",
        "selection-background-color: Fluent2Palette.selection-background",
        "selection-foreground-color: Fluent2Palette.accent-foreground",
        "base.selection-foreground-color: Fluent2Palette.text-accent-foreground-disabled",
        "min-height: max(Fluent2SizeSettings.control-height, layout.min-height)",
        "padding-left: Fluent2SizeSettings.control-horizontal-padding",
        "padding-right: Fluent2SizeSettings.control-tight-horizontal-padding",
        "padding-top: Fluent2SizeSettings.control-vertical-padding",
        "padding-bottom: Fluent2SizeSettings.control-vertical-padding",
        "border-width: Fluent2SizeSettings.stroke-width",
    ] {
        assert!(
            !spinbox.contains(copied_literal),
            "fluent2 SpinBox should not bind input colors directly to copied generic token {copied_literal}"
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
fn test_fluent2_progress_indicator_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "progress-track-background",
        "progress-rail-radius",
        "progress-track-radius",
        "progress-determinate-track-offset",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/progressindicator.slint"))
        .expect("fluent2 should embed progressindicator.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "background: Fluent2Palette.progress-track-background",
        "border-radius: Fluent2SizeSettings.progress-rail-radius",
        "border-radius: Fluent2SizeSettings.progress-track-radius",
        "x: !root.indeterminate ? Fluent2SizeSettings.progress-determinate-track-offset",
    ] {
        assert!(source.contains(expected), "fluent2 progress indicator should use {expected}");
    }

    assert!(
        !source.contains("background: Fluent2Palette.border"),
        "fluent2 progress indicator rail should use a semantic progress track token, not the generic border token"
    );

    assert!(
        !source.contains("border-radius: Fluent2SizeSettings.progress-height"),
        "fluent2 progress indicator track radius should not be derived from progress height"
    );
    assert!(
        !source.contains("x: !root.indeterminate ? 0px"),
        "fluent2 progress indicator determinate track offset should not be hardcoded"
    );
}

#[test]
fn test_fluent2_progress_indicators_use_semantic_active_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["progress-indicator-active-background", "spinner-active-stroke"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "progress-indicator-active-background: accent-background",
        "spinner-active-stroke: accent-background",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind progress semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let progress = load_file(&std::path::PathBuf::from("builtin:/fluent2/progressindicator.slint"))
        .expect("fluent2 should embed progressindicator.slint");
    let progress_contents = progress.read();
    let progress = std::str::from_utf8(&progress_contents).unwrap();

    assert!(
        progress.contains("background: Fluent2Palette.progress-indicator-active-background"),
        "fluent2 ProgressIndicator active track should use the semantic active background token"
    );
    assert!(
        !progress.contains("background: Fluent2Palette.accent-background"),
        "fluent2 ProgressIndicator active track should not bind directly to the generic accent background"
    );

    let spinner = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinner.slint"))
        .expect("fluent2 should embed spinner.slint");
    let spinner_contents = spinner.read();
    let spinner = std::str::from_utf8(&spinner_contents).unwrap();

    assert!(
        spinner.contains("stroke: Fluent2Palette.spinner-active-stroke"),
        "fluent2 Spinner active arc should use the semantic active stroke token"
    );
    assert!(
        !spinner.contains("stroke: Fluent2Palette.accent-background"),
        "fluent2 Spinner active arc should not bind directly to the generic accent background"
    );
}

#[test]
fn test_fluent2_spinner_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("spinner-radius"),
        "fluent2 styling should expose a spinner radius token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinner-base.slint"))
        .expect("fluent2 should embed spinner-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("private property <float> radius: Fluent2SizeSettings.spinner-radius"),
        "fluent2 spinner arc radius should use the radius token"
    );
    assert!(
        !source.contains(
            "private property <float> radius: min(self.viewbox-width, self.viewbox-height) / 2"
        ),
        "fluent2 spinner arc radius should not be derived from live viewbox geometry"
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
fn test_fluent2_shadow_bridge_uses_rest_elevation_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling
            .lines()
            .any(|line| line.trim() == "out property <brush> shadow: elevation-shadow-rest-color;"),
        "fluent2 legacy shadow bridge should use the semantic rest elevation color token"
    );
    assert!(
        !styling.lines().any(|line| line.trim() == "out property <brush> shadow: shadow-ambient;"),
        "fluent2 legacy shadow bridge should not bypass semantic elevation tokens"
    );
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
fn test_fluent2_state_layer_motion_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("state-layer-motion-duration"),
        "fluent2 styling should expose a semantic state layer motion duration token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let state_layer = source
        .split("export component StateLayer")
        .nth(1)
        .and_then(|after| after.split("export struct IconButtonStyle").next())
        .expect("fluent2 should define StateLayer before IconButtonStyle");

    assert!(
        state_layer.contains(
            "animate background { duration: Fluent2SizeSettings.state-layer-motion-duration; }"
        ),
        "fluent2 StateLayer should animate background with the semantic state layer motion token"
    );
    assert!(
        !state_layer.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 StateLayer should not borrow the generic control motion token"
    );
}

#[test]
fn test_fluent2_picker_state_brushes_use_state_layer_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "property <brush> state-layer-brush: dark-color-scheme ? #FFFFFF : #000000",
        "property <brush> state-layer-on-accent-brush: dark-color-scheme ? #000000 : #FFFFFF",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should define {expected}");
    }

    for expected in [
        "date-picker-day-state-brush: state-layer-brush",
        "date-picker-day-selected-state-brush: state-layer-on-accent-brush",
        "date-picker-day-today-state-brush: state-layer-brush",
        "date-picker-icon-state-brush: state-layer-brush",
        "date-picker-selection-button-state-brush: state-layer-brush",
        "time-picker-selector-state-brush: state-layer-brush",
        "time-picker-selector-selected-state-brush: state-layer-on-accent-brush",
        "time-picker-input-state-brush: state-layer-brush",
        "time-picker-input-selected-state-brush: state-layer-on-accent-brush",
        "time-picker-period-item-state-brush: state-layer-brush",
        "time-picker-period-item-selected-state-brush: state-layer-on-accent-brush",
        "lineedit-icon-state-brush: state-layer-brush",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected_line),
            "fluent2 picker and icon state brushes should use shared Fluent2 state-layer tokens: {expected}"
        );
    }

    for copied_literal in [
        "date-picker-day-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "date-picker-day-selected-state-brush: dark-color-scheme ? #000000 : #FFFFFF",
        "date-picker-day-today-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "date-picker-icon-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "date-picker-selection-button-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "time-picker-selector-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "time-picker-selector-selected-state-brush: dark-color-scheme ? #000000 : #FFFFFF",
        "time-picker-input-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "time-picker-input-selected-state-brush: dark-color-scheme ? #000000 : #FFFFFF",
        "time-picker-period-item-state-brush: dark-color-scheme ? #FFFFFF : #000000",
        "time-picker-period-item-selected-state-brush: dark-color-scheme ? #000000 : #FFFFFF",
        "lineedit-icon-state-brush: dark-color-scheme ? #FFFFFF : #000000",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 semantic state brush should not repeat copied light/dark literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_public_state_bridges_use_state_layer_primitives() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "out property <brush> state: state-layer-brush;",
        "out property <brush> state-secondary: state-layer-on-accent-brush;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 public state bridge should use state-layer primitive: {expected}"
        );
    }

    for copied_literal in [
        "out property <brush> state: dark-color-scheme ? #FFFFFF : #000000",
        "out property <brush> state-secondary: dark-color-scheme ? #000000 : #FFFFFF",
    ] {
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal),
            "fluent2 public state bridge should not repeat state-layer primitive literals: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_transparent_visuals_use_palette_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("control-fill-transparent"),
        "fluent2 styling should expose a transparent control fill token"
    );

    for control in [
        "button.slint",
        "components.slint",
        "menus.slint",
        "switch.slint",
        "tabwidget.slint",
        "tableview.slint",
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        let expected_transparent_token = match control {
            "button.slint" => "Fluent2Palette.button-icon-transparent-foreground",
            "components.slint" => "Fluent2Palette.list-item-background",
            "menus.slint" => "Fluent2Palette.menu-popup-window-background",
            "switch.slint" => "Fluent2Palette.switch-rail-disabled-background",
            "tabwidget.slint" => "Fluent2Palette.tab-background",
            "tableview.slint" => "Fluent2Palette.table-row-alternate-background",
            _ => "Fluent2Palette.control-fill-transparent",
        };

        assert!(
            source.contains(expected_transparent_token),
            "fluent2 {control} should use the transparent palette token through {expected_transparent_token}"
        );

        for copied_literal in [
            "background: transparent",
            "border-color: transparent",
            "colorize: root.colorize-icon ? root.text-color : transparent",
        ] {
            assert!(
                !source.contains(copied_literal),
                "fluent2 {control} should not hardcode transparent visual treatment with {copied_literal}"
            );
        }
    }

    assert!(
        styling.contains(
            "out property <brush> button-icon-transparent-foreground: control-fill-transparent"
        ),
        "fluent2 Button transparent icon fallback should stay backed by the transparent control fill token"
    );
    assert!(
        styling.contains(
            "out property <brush> menu-popup-window-background: control-fill-transparent"
        ),
        "fluent2 PopupMenuImpl window background should stay backed by the transparent control fill token"
    );
    assert!(
        styling
            .contains("property <brush> neutral-background-transparent: control-fill-transparent"),
        "fluent2 transparent neutral backgrounds should stay backed by the transparent control fill token"
    );

    for expected in [
        "out property <brush> list-item-background: neutral-background-transparent",
        "out property <brush> table-row-alternate-background: neutral-background-transparent",
    ] {
        assert!(
            styling.contains(expected),
            "fluent2 semantic transparent visual token should use the Fluent2 transparent neutral background token: {expected}"
        );
    }

    for copied_literal in [
        "out property <brush> list-item-background: transparent",
        "out property <brush> table-row-alternate-background: transparent",
    ] {
        assert!(
            !styling.contains(copied_literal),
            "fluent2 semantic transparent visual token should not bypass control-fill-transparent with {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_transparent_strokes_use_transparent_stroke_primitive() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "property <brush> neutral-stroke-transparent: control-fill-transparent;",
        "out property <brush> button-primary-disabled-border: neutral-stroke-transparent;",
        "out property <brush> tab-background: neutral-stroke-transparent;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 transparent stroke/background token should use the transparent stroke primitive: {expected}"
        );
    }

    for copied_literal in [
        "button-primary-disabled-border: neutral-background-1.transparentize(100%)",
        "tab-background: neutral-background-1.transparentize(100%)",
    ] {
        assert!(
            !styling.contains(copied_literal),
            "fluent2 transparent stroke/background token should not derive transparency from a live background: {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_control_alt_quaternary_token_uses_fluent2_name() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "property <brush> control-alt-fill-secondary: dark-color-scheme ? #0000001A : #00000005;",
        "property <brush> control-alt-fill-tertiary: dark-color-scheme ? #FFFFFF0A : #0000000F;",
        "property <brush> control-alt-fill-quaternary: dark-color-scheme ? #FFFFFF12 : #00000017;",
        "out property <brush> control-alt-secondary: control-alt-fill-secondary;",
        "out property <brush> control-alt-tertiary: control-alt-fill-tertiary;",
        "out property <brush> control-alt-quaternary: control-alt-fill-quaternary;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 public control-alt bridge tokens should route through named Fluent2 fill primitives: {expected}"
        );
    }

    for copied_bridge in [
        "out property <brush> control-alt-secondary: dark-color-scheme ? #0000001A : #00000005;",
        "out property <brush> control-alt-tertiary: dark-color-scheme ? #FFFFFF0A : #0000000F;",
        "out property <brush> control-alt-quaternary: dark-color-scheme ? #FFFFFF12 : #00000017;",
    ] {
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge),
            "fluent2 public control-alt bridge tokens should not own raw copied light/dark alpha branches: {copied_bridge}"
        );
    }

    assert!(
        styling.contains("control-alt-quaternary"),
        "fluent2 styling should expose the Fluent2 control-alt-quaternary token name"
    );
    assert!(
        !styling.contains("control-alt-quartiary"),
        "fluent2 styling should not keep the copied misspelled control-alt-quartiary token name"
    );

    for (control, expected) in [("switch.slint", "Fluent2Palette.switch-rail-pressed-background")] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains(expected),
            "fluent2 {control} should use the corrected control-alt-quaternary token through {expected}"
        );
        assert!(
            !source.contains("control-alt-quartiary"),
            "fluent2 {control} should not use the copied misspelled token name"
        );
    }

    for expected in [
        "property <brush> switch-rail-pressed-fill: control-alt-quaternary;",
        "out property <brush> switch-rail-pressed-background: switch-rail-pressed-fill;",
    ] {
        assert!(
            styling.contains(expected),
            "fluent2 semantic pressed tokens should use the expected Fluent2 pressed treatment through {expected}"
        );
    }

    assert!(
        !styling.contains(
            "out property <brush> switch-rail-pressed-background: dark-color-scheme ? #FFFFFF12 : #00000017"
        ),
        "fluent2 switch pressed rail token should not repeat the copied raw alpha branch"
    );
}

#[test]
fn test_fluent2_disabled_transparent_fill_uses_palette_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("out property <brush> control-fill-transparent: transparent"),
        "fluent2 should expose a single transparent control fill token"
    );
    assert!(
        styling.contains("out property <brush> control-alt-disabled: control-fill-transparent"),
        "fluent2 disabled alternate fill should reuse the transparent fill token"
    );
    assert!(
        !styling.contains("out property <brush> control-alt-disabled: transparent"),
        "fluent2 disabled alternate fill should not hardcode transparent separately"
    );
}

#[test]
fn test_fluent2_scrollbar_opacity_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "scrollbar-button-hidden-opacity",
        "scrollbar-button-visible-opacity",
        "scrollbar-motion-duration",
    ] {
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
        "duration: Fluent2SizeSettings.scrollbar-motion-duration",
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

    assert!(
        !source.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 scrollview should not borrow the generic control motion token for scrollbar motion"
    );
}

#[test]
fn test_fluent2_scrollbar_thumb_radius_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "scrollbar-track-radius",
        "scrollbar-track-border-width",
        "scrollbar-thumb-radius",
        "scrollbar-zero-range-thumb-size",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/scrollview.slint"))
        .expect("fluent2 should embed scrollview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("border-radius: Fluent2SizeSettings.scrollbar-track-radius"),
        "fluent2 scrollbar track should use the track radius token"
    );
    assert!(
        source.contains("border-width: Fluent2SizeSettings.scrollbar-track-border-width"),
        "fluent2 scrollbar track should use the track border width token"
    );
    assert!(
        source.contains("border-radius: Fluent2SizeSettings.scrollbar-thumb-radius"),
        "fluent2 scrollbar thumb should use the thumb radius token"
    );
    assert!(
        source
            .contains("root.maximum <= 0phx ? Fluent2SizeSettings.scrollbar-zero-range-thumb-size"),
        "fluent2 scrollbar thumb should use the zero-range thumb size token"
    );
    assert!(
        !source.contains("border-radius: Fluent2SizeSettings.overlay-radius"),
        "fluent2 scrollbar track should not borrow the generic overlay radius token"
    );
    assert!(
        !source.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 scrollbar track should not bind border width directly to the generic stroke-width token"
    );
    assert!(
        !source.contains("border-radius: (root.horizontal ? self.height : self.width) / 2"),
        "fluent2 scrollbar thumb radius should not be derived from live thumb geometry"
    );
    assert!(
        !source.contains("root.maximum <= 0phx ? 0phx"),
        "fluent2 scrollbar thumb zero-range size should not be hardcoded"
    );
}

#[test]
fn test_fluent2_scrollbar_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["scrollbar-button-foreground", "scrollbar-thumb-background"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/scrollview.slint"))
        .expect("fluent2 should embed scrollview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    let button = source
        .split("component ScrollViewButton")
        .nth(1)
        .and_then(|after| after.split("component ScrollBar").next())
        .expect("fluent2 should define ScrollViewButton");
    assert!(
        button.contains("colorize: Fluent2Palette.scrollbar-button-foreground"),
        "fluent2 scrollbar buttons should use the semantic button foreground token"
    );
    assert!(
        !button.contains("Fluent2Palette.border"),
        "fluent2 scrollbar buttons should not use the generic border token for icon color"
    );

    let scrollbar =
        source.split("component ScrollBar").nth(1).expect("fluent2 should define ScrollBar");
    assert!(
        scrollbar.contains("background: Fluent2Palette.scrollbar-thumb-background"),
        "fluent2 scrollbar thumb should use the semantic thumb background token"
    );
    assert!(
        !scrollbar.contains("background: Fluent2Palette.border"),
        "fluent2 scrollbar thumb should not use the generic border token for fill"
    );
}

#[test]
fn test_fluent2_control_icons_use_semantic_foreground_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "combobox-icon-foreground",
        "combobox-icon-pressed-foreground",
        "spinbox-button-icon-foreground",
        "spinbox-button-icon-pressed-foreground",
        "table-sort-icon-foreground",
        "scrollbar-button-hover-foreground",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "combobox-icon-foreground: text-secondary",
        "combobox-icon-pressed-foreground: text-tertiary",
        "spinbox-button-icon-foreground: text-secondary",
        "spinbox-button-icon-pressed-foreground: text-tertiary",
        "table-sort-icon-foreground: text-secondary",
        "scrollbar-button-hover-foreground: text-secondary",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind control icon semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    for (file, component_marker, expected, copied_literal) in [
        (
            "combobox.slint",
            "export component ComboBox",
            "icon.colorize: Fluent2Palette.combobox-icon-pressed-foreground",
            "icon.colorize: Fluent2Palette.text-tertiary",
        ),
        (
            "combobox.slint",
            "export component ComboBox",
            "colorize: Fluent2Palette.combobox-icon-foreground",
            "colorize: Fluent2Palette.text-secondary",
        ),
        (
            "spinbox.slint",
            "component SpinBoxButton",
            "icon.colorize: Fluent2Palette.spinbox-button-icon-pressed-foreground",
            "icon.colorize: Fluent2Palette.text-tertiary",
        ),
        (
            "spinbox.slint",
            "component SpinBoxButton",
            "colorize: Fluent2Palette.spinbox-button-icon-foreground",
            "colorize: Fluent2Palette.text-secondary",
        ),
        (
            "tableview.slint",
            "component TableViewColumn",
            "colorize: Fluent2Palette.table-header-sort-icon-foreground",
            "colorize: Fluent2Palette.text-secondary",
        ),
        (
            "scrollview.slint",
            "component ScrollViewButton",
            "icon.colorize: Fluent2Palette.scrollbar-button-hover-foreground",
            "icon.colorize: Fluent2Palette.text-secondary",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let block = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {file} should define {component_marker}"));

        assert!(block.contains(expected), "fluent2 {file} should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 {file} should not use copied generic text tokens for control icon foregrounds"
        );
    }

    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();
    assert!(
        !styling.contains(
            "out property <brush> table-header-sort-icon-foreground: table-sort-icon-foreground"
        ),
        "fluent2 table header sort icon token should bind directly instead of through the table sort icon bridge"
    );
}

#[test]
fn test_fluent2_scrollbar_buttons_use_focus_touch_area() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("scrollbar-button-pressed-icon-inset"),
        "fluent2 styling should expose a scrollbar-specific pressed icon inset token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/scrollview.slint"))
        .expect("fluent2 should embed scrollview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("FocusTouchArea"),
        "fluent2 scrollview should import the Fluent2 interaction helper"
    );

    let block = source
        .split("component ScrollViewButton")
        .nth(1)
        .and_then(|after| after.split("component ScrollBar").next())
        .expect("fluent2 should define ScrollViewButton");

    for expected in [
        "out property <bool> pressed: touch-area.pressed",
        "out property <bool> has-hover: touch-area.has-hover",
        "callback clicked <=> touch-area.clicked",
        "touch-area := FocusTouchArea",
        "enabled: true",
        "pressed when root.pressed",
        "icon.width: root.width - Fluent2SizeSettings.scrollbar-button-pressed-icon-inset",
        "hover when root.has-hover",
    ] {
        assert!(block.contains(expected), "fluent2 ScrollViewButton should use {expected}");
    }

    assert!(
        !block.contains(
            "icon.width: root.width - Fluent2SizeSettings.control-tight-horizontal-padding"
        ),
        "fluent2 ScrollViewButton pressed icon sizing should not use the generic control padding token"
    );

    assert!(
        !source.contains("component ScrollViewButton inherits TouchArea"),
        "fluent2 ScrollViewButton should not inherit directly from TouchArea"
    );
    assert!(
        source.contains("touch-area := TouchArea"),
        "fluent2 ScrollBar should keep the thumb/track drag area as a behavior-specific TouchArea"
    );
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
fn test_fluent2_selection_indicator_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("selection-indicator-offset"),
        "fluent2 should expose a token for selection indicator alignment"
    );

    for (file, component_marker) in [
        ("components.slint", "export component ListItem"),
        ("tableview.slint", "component TableViewRow"),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let block = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {file} should define {component_marker}"));

        for expected in [
            "x: Fluent2SizeSettings.selection-indicator-offset",
            "width: Fluent2SizeSettings.selection-indicator-width",
            "height: Fluent2SizeSettings.selection-indicator-hidden-height",
            "border-radius: Fluent2SizeSettings.selection-indicator-radius",
        ] {
            assert!(block.contains(expected), "fluent2 {file} should use {expected}");
        }

        assert!(
            !block.contains("x: 0px"),
            "fluent2 {file} should not hardcode selection indicator x alignment"
        );
    }
}

#[test]
fn test_fluent2_list_and_table_row_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "list-item-background",
        "list-item-foreground",
        "list-item-hover-foreground",
        "list-item-hover-background",
        "list-item-pressed-background",
        "list-item-selected-background",
        "list-item-selected-hover-background",
        "list-item-selected-pressed-background",
        "list-item-selection-indicator-background",
        "table-row-background",
        "table-row-alternate-background",
        "table-row-foreground",
        "table-row-alternate-foreground",
        "table-row-hover-background",
        "table-row-pressed-background",
        "table-row-selected-background",
        "table-row-selected-hover-background",
        "table-row-selected-pressed-background",
        "table-row-selection-indicator-background",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "list-item-background: control-fill-transparent",
        "list-item-foreground: control-foreground",
        "list-item-hover-foreground: text-secondary",
        "list-item-hover-background: subtle-secondary",
        "list-item-pressed-background: subtle-tertiary",
        "list-item-selected-background: subtle-secondary",
        "list-item-selected-hover-background: subtle-tertiary",
        "list-item-selected-pressed-background: subtle-secondary",
        "list-item-selection-indicator-background: accent-background",
        "table-row-background: control-background",
        "table-row-alternate-background: control-fill-transparent",
        "table-row-foreground: control-foreground",
        "table-row-alternate-foreground: text-secondary",
        "table-row-hover-background: subtle-secondary",
        "table-row-pressed-background: subtle-tertiary",
        "table-row-selected-background: subtle-secondary",
        "table-row-selected-hover-background: subtle-tertiary",
        "table-row-selected-pressed-background: subtle-secondary",
        "table-row-selection-indicator-background: accent-background",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind list/table row semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let components = load_file(&std::path::PathBuf::from("builtin:/fluent2/components.slint"))
        .expect("fluent2 should embed components.slint");
    let components_contents = components.read();
    let components = std::str::from_utf8(&components_contents).unwrap();
    let list_item = components
        .split("export component ListItem")
        .nth(1)
        .expect("fluent2 should define ListItem");

    for expected in [
        "i-background.background: is-selected ? Fluent2Palette.list-item-selected-pressed-background : Fluent2Palette.list-item-pressed-background",
        "i-text.color: Fluent2Palette.list-item-hover-foreground",
        "i-background.background: is-selected ? Fluent2Palette.list-item-selected-hover-background : Fluent2Palette.list-item-hover-background",
        "i-background.background: Fluent2Palette.list-item-selected-background",
        "min-height: max(Fluent2SizeSettings.list-item-height, i-layout.min-height)",
        "border-radius: Fluent2SizeSettings.list-item-radius",
        "spacing: Fluent2SizeSettings.list-item-content-spacing",
        "background: Fluent2Palette.list-item-background",
        "color: Fluent2Palette.list-item-foreground",
        "background: Fluent2Palette.list-item-selection-indicator-background",
    ] {
        assert!(list_item.contains(expected), "fluent2 ListItem should use {expected}");
    }

    for copied_literal in [
        "i-background.background: is-selected ? Fluent2Palette.subtle-secondary : Fluent2Palette.subtle-tertiary",
        "i-text.color: Fluent2Palette.text-secondary",
        "i-background.background: is-selected ? Fluent2Palette.subtle-tertiary : Fluent2Palette.subtle-secondary",
        "i-background.background: Fluent2Palette.subtle-secondary",
        "min-height: max(Fluent2SizeSettings.item-height, i-layout.min-height)",
        "border-radius: Fluent2SizeSettings.control-radius",
        "spacing: Fluent2SizeSettings.overlay-padding",
        "background: Fluent2Palette.control-fill-transparent",
        "color: Fluent2Palette.control-foreground",
        "background: Fluent2Palette.accent-background",
    ] {
        assert!(
            !list_item.contains(copied_literal),
            "fluent2 ListItem should not bind row colors directly to copied generic token {copied_literal}"
        );
    }

    let table = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let table_contents = table.read();
    let table = std::str::from_utf8(&table_contents).unwrap();
    let table_row =
        table.split("component TableViewRow").nth(1).expect("fluent2 should define TableViewRow");

    for expected in [
        "background: root.even ? Fluent2Palette.table-row-background : Fluent2Palette.table-row-alternate-background",
        "root.background: selected ? Fluent2Palette.table-row-selected-pressed-background : Fluent2Palette.table-row-pressed-background",
        "root.background: selected ? Fluent2Palette.table-row-selected-hover-background : Fluent2Palette.table-row-hover-background",
        "root.background: Fluent2Palette.table-row-selected-background",
        "background: Fluent2Palette.table-row-selection-indicator-background",
    ] {
        assert!(table_row.contains(expected), "fluent2 TableViewRow should use {expected}");
    }

    let standard_table = table
        .split("export component StandardTableView")
        .nth(1)
        .expect("fluent2 should define StandardTableView");
    for expected in [
        "color: mod(idx, 2) == 0 ? Fluent2Palette.table-row-foreground : Fluent2Palette.table-row-alternate-foreground",
    ] {
        assert!(
            standard_table.contains(expected),
            "fluent2 StandardTableView should use {expected}"
        );
    }

    for copied_literal in [
        "background: root.even ? Fluent2Palette.control-background : Fluent2Palette.control-fill-transparent",
        "root.background: selected ? Fluent2Palette.subtle-secondary : Fluent2Palette.subtle-tertiary",
        "root.background: selected ? Fluent2Palette.subtle-tertiary : Fluent2Palette.subtle-secondary",
        "root.background: Fluent2Palette.subtle-secondary",
        "background: Fluent2Palette.accent-background",
        "color: mod(idx, 2) == 0 ? Fluent2Palette.control-foreground : Fluent2Palette.text-secondary",
    ] {
        assert!(
            !table.contains(copied_literal),
            "fluent2 table rows should not bind row colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_list_item_motion_uses_list_duration_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("list-item-motion-duration"),
        "fluent2 styling should expose a semantic list item motion duration token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/components.slint"))
        .expect("fluent2 should embed components.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let list_item = source
        .split("export component ListItem")
        .nth(1)
        .and_then(|after| after.split("@children").next())
        .expect("fluent2 components should define ListItem before @children");

    for expected in [
        "animate background { duration: Fluent2SizeSettings.list-item-motion-duration; }",
        "animate color { duration: Fluent2SizeSettings.list-item-motion-duration; }",
        "animate height { duration: Fluent2SizeSettings.list-item-motion-duration; easing: ease-out; }",
    ] {
        assert!(list_item.contains(expected), "fluent2 ListItem should use {expected}");
    }

    assert!(
        !list_item.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 ListItem should not borrow the generic control motion token"
    );
}

#[test]
fn test_fluent2_table_header_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "table-header-sort-icon-spacing",
        "table-header-sort-icon-size",
        "table-header-separator-thickness",
        "table-header-resize-separator-thickness",
        "table-row-height",
        "table-row-radius",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for token in ["table-header-horizontal-padding", "table-cell-horizontal-padding"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for token in [
        "table-header-background",
        "table-header-hover-background",
        "table-header-pressed-background",
        "table-header-foreground",
        "table-header-sort-icon-foreground",
        "table-header-resize-hover-background",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "table-header-pressed-background: control-secondary",
        "table-header-foreground: text-secondary",
        "table-header-sort-icon-foreground: table-sort-icon-foreground",
        "table-header-resize-hover-background: control-secondary",
    ] {
        assert!(
            !styling.contains(copied_alias),
            "fluent2 styling should bind table header semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let column = source
        .split("component TableViewColumn")
        .nth(1)
        .and_then(|after| after.split("component TableViewCell").next())
        .expect("fluent2 should define TableViewColumn before TableViewCell");

    for expected in [
        "background: Fluent2Palette.table-header-background",
        "background: Fluent2Palette.table-header-pressed-background",
        "background: Fluent2Palette.table-header-hover-background",
        "colorize: Fluent2Palette.table-header-sort-icon-foreground",
        "width: Fluent2SizeSettings.table-header-sort-icon-size",
        "background: Fluent2Palette.table-header-resize-hover-background",
        "height: Fluent2SizeSettings.table-header-separator-thickness",
        "x: parent.width - Fluent2SizeSettings.table-header-resize-separator-thickness",
        "width: Fluent2SizeSettings.table-header-resize-separator-thickness",
        "spacing: Fluent2SizeSettings.table-header-sort-icon-spacing",
        "width: parent.width - Fluent2SizeSettings.table-header-horizontal-padding",
        "padding-left: Fluent2SizeSettings.table-header-horizontal-padding",
        "padding-right: Fluent2SizeSettings.table-header-horizontal-padding",
    ] {
        assert!(column.contains(expected), "fluent2 TableViewColumn should use {expected}");
    }

    let cell = source
        .split("component TableViewCell")
        .nth(1)
        .and_then(|after| after.split("component TableViewRow").next())
        .expect("fluent2 should define TableViewCell before TableViewRow");
    for expected in [
        "padding-left: Fluent2SizeSettings.table-cell-horizontal-padding",
        "padding-right: Fluent2SizeSettings.table-cell-horizontal-padding",
    ] {
        assert!(cell.contains(expected), "fluent2 TableViewCell should use {expected}");
    }

    let header_delegate = source
        .split("for column[index] in root.columns : TableViewColumn")
        .nth(1)
        .and_then(|after| after.split("scroll-view := ListView").next())
        .expect("fluent2 StandardTableView should define header columns before the row ListView");

    assert!(
        header_delegate.contains("color: Fluent2Palette.table-header-foreground"),
        "fluent2 StandardTableView header text should use the semantic table header foreground token"
    );

    for copied_literal in [
        "background: Fluent2Palette.control-secondary",
        "background: Fluent2Palette.sub-title-tertiary",
        "colorize: Fluent2Palette.table-sort-icon-foreground",
        "color: Fluent2Palette.text-secondary",
        "width: Fluent2SizeSettings.icon-size",
        "height: Fluent2SizeSettings.separator-thickness",
        "width: Fluent2SizeSettings.separator-thickness",
        "spacing: Fluent2SizeSettings.control-tight-horizontal-padding",
        "width: parent.width - Fluent2SizeSettings.control-horizontal-padding",
        "padding-left: Fluent2SizeSettings.control-horizontal-padding",
        "padding-right: Fluent2SizeSettings.control-horizontal-padding",
    ] {
        assert!(
            !source.contains(copied_literal),
            "fluent2 table headers should not bind directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_table_motion_uses_table_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("table-motion-duration"),
        "fluent2 styling should expose table-motion-duration"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    let column = source
        .split("component TableViewColumn")
        .nth(1)
        .and_then(|after| after.split("component TableViewCell").next())
        .expect("fluent2 should define TableViewColumn before TableViewCell");

    for expected in [
        "animate colorize { duration: Fluent2SizeSettings.table-motion-duration",
        "animate background { duration: Fluent2SizeSettings.table-motion-duration",
    ] {
        assert!(column.contains(expected), "fluent2 TableViewColumn should use {expected}");
    }

    assert!(
        !column.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 TableViewColumn should not use the generic control motion token"
    );

    let row = source
        .split("component TableViewRow")
        .nth(1)
        .and_then(|after| after.split("export component StandardTableView").next())
        .expect("fluent2 should define TableViewRow before StandardTableView");

    assert!(
        row.contains("animate height { duration: Fluent2SizeSettings.table-motion-duration"),
        "fluent2 TableViewRow selection indicator should use table-motion-duration"
    );
    assert!(
        !row.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 TableViewRow should not use the generic control motion token"
    );
}

#[test]
fn test_fluent2_menu_separator_insets_use_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "menu-separator-horizontal-inset",
        "menu-separator-vertical-padding",
        "menu-separator-thickness",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/menu.slint"))
        .expect("fluent2 should embed menu.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let block = source
        .split("export component MenuItem")
        .nth(1)
        .expect("fluent2 menu.slint should define MenuItem");

    for expected in [
        "padding: entry.is-separator ? Fluent2SizeSettings.menu-separator-vertical-padding : Fluent2SizeSettings.menu-item-outer-padding",
        "padding-left: entry.is-separator ? Fluent2SizeSettings.menu-separator-horizontal-inset : Fluent2SizeSettings.menu-item-outer-padding",
        "padding-right: entry.is-separator ? Fluent2SizeSettings.menu-separator-horizontal-inset : Fluent2SizeSettings.menu-item-outer-padding",
        "separator-height: Fluent2SizeSettings.menu-separator-thickness",
    ] {
        assert!(block.contains(expected), "fluent2 MenuItem should use {expected}");
    }

    for copied_literal in [
        "padding-left: entry.is-separator ? 0px",
        "padding-right: entry.is-separator ? 0px",
        "padding: entry.is-separator ? Fluent2SizeSettings.overlay-padding",
        "separator-height: Fluent2SizeSettings.separator-thickness",
    ] {
        assert!(
            !block.contains(copied_literal),
            "fluent2 MenuItem should not hardcode separator inset with {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_menu_border_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "menu-frame-border",
        "menu-flyout-border",
        "menu-separator-color",
        "menu-frame-radius",
        "menu-flyout-radius",
        "menu-frame-border-width",
        "menu-flyout-border-width",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for copied_alias in ["menu-flyout-border: control-background-stroke-flyout"] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind menu border semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/menu.slint"))
        .expect("fluent2 should embed menu.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    let frame_block = source
        .split("export component MenuFrame")
        .nth(1)
        .and_then(|block| block.split("export component MenuItem").next())
        .expect("fluent2 menu.slint should define MenuFrame before MenuItem");
    let item_block = source
        .split("export component MenuItem")
        .nth(1)
        .expect("fluent2 menu.slint should define MenuItem");

    assert!(
        frame_block.contains("border-color: Fluent2Palette.menu-frame-border"),
        "fluent2 MenuFrame should use the semantic menu frame border token"
    );
    assert!(
        frame_block.contains("border-radius: Fluent2SizeSettings.menu-frame-radius"),
        "fluent2 MenuFrame should use the semantic menu frame radius token"
    );
    assert!(
        frame_block.contains("border-width: Fluent2SizeSettings.menu-frame-border-width"),
        "fluent2 MenuFrame should use the semantic menu frame border width token"
    );
    assert!(
        item_block.contains("separator-color: Fluent2Palette.menu-separator-color"),
        "fluent2 MenuItem separators should use the semantic separator color token"
    );

    assert!(
        !frame_block.contains("Fluent2Palette.border"),
        "fluent2 MenuFrame should not use the generic border token"
    );
    assert!(
        !frame_block.contains("Fluent2SizeSettings.overlay-radius"),
        "fluent2 MenuFrame should not borrow the generic overlay radius token"
    );
    assert!(
        !frame_block.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 MenuFrame should not bind border width directly to the generic stroke-width token"
    );
    assert!(
        !item_block.contains("separator-color: Fluent2Palette.border"),
        "fluent2 MenuItem separators should not use the generic border token"
    );

    let components = load_file(&std::path::PathBuf::from("builtin:/fluent2/components.slint"))
        .expect("fluent2 should embed components.slint");
    let components_contents = components.read();
    let components = std::str::from_utf8(&components_contents).unwrap();
    let menu_border = components
        .split("export component MenuBorder")
        .nth(1)
        .and_then(|block| block.split("export component ListItem").next())
        .expect("fluent2 components.slint should define MenuBorder before ListItem");

    assert!(
        menu_border.contains("border-color: Fluent2Palette.menu-flyout-border"),
        "fluent2 MenuBorder should use the semantic menu flyout border token"
    );
    assert!(
        menu_border.contains("border-radius: Fluent2SizeSettings.menu-flyout-radius"),
        "fluent2 MenuBorder should use the semantic menu flyout radius token"
    );
    assert!(
        menu_border.contains("border-width: Fluent2SizeSettings.menu-flyout-border-width"),
        "fluent2 MenuBorder should use the semantic menu flyout border width token"
    );
    assert!(
        !menu_border.contains("Fluent2Palette.control-background-stroke-flyout"),
        "fluent2 MenuBorder should not bind directly to the generic flyout stroke token"
    );
    assert!(
        !menu_border.contains("Fluent2SizeSettings.overlay-radius"),
        "fluent2 MenuBorder should not borrow the generic overlay radius token"
    );
    assert!(
        !menu_border.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 MenuBorder should not bind border width directly to the generic stroke-width token"
    );
}

#[test]
fn test_fluent2_menu_item_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "menu-bar-item-foreground",
        "menu-bar-item-pressed-foreground",
        "menu-bar-item-hover-background",
        "menu-bar-item-pressed-background",
        "menu-item-foreground",
        "menu-item-current-foreground",
        "menu-item-current-background",
        "menu-bar-item-horizontal-padding",
        "menu-bar-item-top-padding",
        "menu-bar-item-radius",
        "menu-bar-spacing",
        "menu-item-horizontal-padding",
        "menu-item-spacing",
        "menu-item-radius",
        "menu-item-icon-size",
        "menu-item-height",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "menu-bar-item-foreground: foreground",
        "menu-bar-item-pressed-foreground: text-secondary",
        "menu-bar-item-hover-background: subtle-secondary",
        "menu-bar-item-pressed-background: control-alt-tertiary",
        "menu-item-foreground: foreground",
        "menu-item-current-foreground: foreground",
        "menu-item-current-background: subtle-secondary",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 menu semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    assert!(
        styling.contains(
            "menu-bar-item-pressed-background: state-layer-brush.with_alpha(Fluent2SizeSettings.state-layer-active-opacity)"
        ),
        "fluent2 menu bar pressed background should use the shared state-layer active opacity token"
    );
    assert!(
        !styling.contains(
            "menu-bar-item-pressed-background: dark-color-scheme ? #FFFFFF0A : #0000000F"
        ),
        "fluent2 menu bar pressed background should not hardcode copied light/dark alpha colors"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/menu.slint"))
        .expect("fluent2 should embed menu.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    let menu_bar_item = source
        .split("export component MenuBarItem")
        .nth(1)
        .and_then(|after| after.split("export component MenuBar").next())
        .expect("fluent2 menu.slint should define MenuBarItem before MenuBar");

    for expected in [
        "default-foreground: Fluent2Palette.menu-bar-item-foreground",
        "hover-foreground: Fluent2Palette.menu-bar-item-foreground",
        "pressed-foreground: Fluent2Palette.menu-bar-item-pressed-foreground",
        "hover-background: Fluent2Palette.menu-bar-item-hover-background",
        "pressed-background: Fluent2Palette.menu-bar-item-pressed-background",
        "horizontal-padding: Fluent2SizeSettings.menu-bar-item-horizontal-padding",
        "top-padding: Fluent2SizeSettings.menu-bar-item-top-padding",
        "border-radius: Fluent2SizeSettings.menu-bar-item-radius",
    ] {
        assert!(menu_bar_item.contains(expected), "fluent2 MenuBarItem should use {expected}");
    }

    let menu_bar = source
        .split("export component MenuBar inherits MenuBarBase")
        .nth(1)
        .and_then(|after| after.split("export component MenuFrame").next())
        .expect("fluent2 menu.slint should define MenuBar before MenuFrame");
    assert!(
        menu_bar.contains("spacing: Fluent2SizeSettings.menu-bar-spacing"),
        "fluent2 MenuBar should use menu-bar-spacing"
    );

    for copied_literal in [
        "default-foreground: Fluent2Palette.foreground",
        "hover-foreground: Fluent2Palette.foreground",
        "pressed-foreground: Fluent2Palette.text-secondary",
        "hover-background: Fluent2Palette.subtle-secondary",
        "pressed-background: Fluent2Palette.control-alt-tertiary",
        "horizontal-padding: Fluent2SizeSettings.control-horizontal-padding",
        "top-padding: Fluent2SizeSettings.control-vertical-padding",
        "spacing: Fluent2SizeSettings.control-spacing",
        "border-radius: Fluent2SizeSettings.control-radius",
    ] {
        assert!(
            !source.contains(copied_literal),
            "fluent2 menu controls should not bind directly to copied generic token {copied_literal}"
        );
    }

    let menu_item = source
        .split("export component MenuItem")
        .nth(1)
        .expect("fluent2 menu.slint should define MenuItem");

    for expected in [
        "default-foreground: Fluent2Palette.menu-item-foreground",
        "current-foreground: Fluent2Palette.menu-item-current-foreground",
        "current-background: Fluent2Palette.menu-item-current-background",
        "horizontal-padding: Fluent2SizeSettings.menu-item-horizontal-padding",
        "spacing: Fluent2SizeSettings.menu-item-spacing",
        "border-radius: Fluent2SizeSettings.menu-item-radius",
        "icon-size: Fluent2SizeSettings.menu-item-icon-size",
        "min-height: entry.is-separator ? Fluent2SizeSettings.menu-separator-height : max(Fluent2SizeSettings.menu-item-height, base.min-height)",
    ] {
        assert!(menu_item.contains(expected), "fluent2 MenuItem should use {expected}");
    }

    for copied_literal in [
        "default-foreground: Fluent2Palette.foreground",
        "current-foreground: Fluent2Palette.foreground",
        "current-background: Fluent2Palette.subtle-secondary",
        "border-radius: Fluent2SizeSettings.control-radius",
        "icon-size: Fluent2SizeSettings.icon-size",
        "Fluent2SizeSettings.compact-item-height",
    ] {
        assert!(
            !menu_item.contains(copied_literal),
            "fluent2 MenuItem should not bind colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_picker_borders_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "date-picker-border",
        "date-picker-day-today-border-width",
        "date-picker-separator-thickness",
        "time-picker-period-border",
        "time-picker-period-border-width",
        "time-picker-period-separator-thickness",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for (file, expected) in [
        ("datepicker.slint", "border-brush: Fluent2Palette.date-picker-border"),
        ("time-picker.slint", "border-brush: Fluent2Palette.time-picker-period-border"),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(source.contains(expected), "fluent2 {file} should use {expected}");
        assert!(
            !source.contains("border-brush: Fluent2Palette.border"),
            "fluent2 {file} should not use the generic border token for picker borders"
        );
    }

    for (file, expected, copied_literal) in [
        (
            "datepicker-base.slint",
            "height: Fluent2SizeSettings.date-picker-separator-thickness",
            "height: Fluent2SizeSettings.separator-thickness",
        ),
        (
            "time-picker-base.slint",
            "height: Fluent2SizeSettings.time-picker-period-separator-thickness",
            "height: Fluent2SizeSettings.separator-thickness",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(source.contains(expected), "fluent2 {file} should use {expected}");
        assert!(
            !source.contains(copied_literal),
            "fluent2 {file} should not borrow the generic separator thickness token"
        );
    }

    let datepicker_base =
        load_file(&std::path::PathBuf::from("builtin:/fluent2/datepicker-base.slint"))
            .expect("fluent2 should embed datepicker-base.slint");
    let datepicker_base_contents = datepicker_base.read();
    let datepicker_base = std::str::from_utf8(&datepicker_base_contents).unwrap();
    assert!(
        datepicker_base.contains(
            "background-layer.border-width: Fluent2SizeSettings.date-picker-day-today-border-width"
        ),
        "fluent2 DatePicker today delegate should use the semantic today border width token"
    );
    assert!(
        !datepicker_base
            .contains("background-layer.border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 DatePicker today delegate should not bind border width directly to generic stroke-width"
    );

    let time_picker = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker.slint"))
        .expect("fluent2 should embed time-picker.slint");
    let time_picker_contents = time_picker.read();
    let time_picker = std::str::from_utf8(&time_picker_contents).unwrap();
    assert!(
        time_picker.contains("border-width: Fluent2SizeSettings.time-picker-period-border-width"),
        "fluent2 TimePicker period selector should use the semantic period border width token"
    );
    assert!(
        !time_picker.contains("border-width: Fluent2SizeSettings.stroke-width,"),
        "fluent2 TimePicker period selector should not bind border width directly to generic stroke-width"
    );
}

#[test]
fn test_fluent2_date_picker_day_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "date-picker-day-foreground",
        "date-picker-day-state-brush",
        "date-picker-day-selected-background",
        "date-picker-day-selected-foreground",
        "date-picker-day-selected-state-brush",
        "date-picker-day-today-border",
        "date-picker-day-today-foreground",
        "date-picker-day-today-state-brush",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "date-picker-day-foreground: foreground",
        "date-picker-day-state-brush: state",
        "date-picker-day-selected-background: accent-background",
        "date-picker-day-selected-foreground: accent-foreground",
        "date-picker-day-selected-state-brush: state-secondary",
        "date-picker-day-today-border: accent-background",
        "date-picker-day-today-foreground: accent-background",
        "date-picker-day-today-state-brush: state",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 date picker day semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/datepicker.slint"))
        .expect("fluent2 should embed datepicker.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let block = source
        .split("delegate-style:")
        .nth(1)
        .and_then(|after| after.split("icon-button-style:").next())
        .expect("fluent2 datepicker.slint should define calendar delegate style");

    for expected in [
        "foreground: Fluent2Palette.date-picker-day-foreground",
        "state-brush: Fluent2Palette.date-picker-day-state-brush",
        "background-selected: Fluent2Palette.date-picker-day-selected-background",
        "foreground-selected: Fluent2Palette.date-picker-day-selected-foreground",
        "state-brush-selected: Fluent2Palette.date-picker-day-selected-state-brush",
        "border-color-today: Fluent2Palette.date-picker-day-today-border",
        "foreground-today: Fluent2Palette.date-picker-day-today-foreground",
        "state-brush-today: Fluent2Palette.date-picker-day-today-state-brush",
    ] {
        assert!(block.contains(expected), "fluent2 DatePicker day delegate should use {expected}");
    }

    for copied_literal in [
        "foreground: Fluent2Palette.foreground",
        "state-brush: Fluent2Palette.state",
        "background-selected: Fluent2Palette.accent-background",
        "foreground-selected: Fluent2Palette.accent-foreground",
        "state-brush-selected: Fluent2Palette.state-secondary",
        "border-color-today: Fluent2Palette.accent-background",
        "foreground-today: Fluent2Palette.accent-background",
        "state-brush-today: Fluent2Palette.state",
    ] {
        assert!(
            !block.contains(copied_literal),
            "fluent2 DatePicker day delegate should not bind picker colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_date_picker_popup_controls_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "date-picker-icon-foreground",
        "date-picker-icon-state-brush",
        "date-picker-current-day-foreground",
        "date-picker-title-foreground",
        "date-picker-selection-button-foreground",
        "date-picker-selection-button-state-brush",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "date-picker-icon-foreground: foreground",
        "date-picker-icon-state-brush: state",
        "date-picker-current-day-foreground: foreground",
        "date-picker-title-foreground: foreground",
        "date-picker-selection-button-foreground: foreground",
        "date-picker-selection-button-state-brush: state",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind date picker popup semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/datepicker.slint"))
        .expect("fluent2 should embed datepicker.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for (style_marker, next_marker, expected, copied_literal) in [
        (
            "icon-button-style:",
            "current-day-style:",
            "foreground: Fluent2Palette.date-picker-icon-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "icon-button-style:",
            "current-day-style:",
            "state-brush: Fluent2Palette.date-picker-icon-state-brush",
            "state-brush: Fluent2Palette.state",
        ),
        (
            "current-day-style:",
            "title-style:",
            "foreground: Fluent2Palette.date-picker-current-day-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "title-style:",
            "previous-icon:",
            "foreground: Fluent2Palette.date-picker-title-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "selection-button-style:",
            "font-family: Fluent2Palette.font-family",
            "foreground: Fluent2Palette.date-picker-selection-button-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "selection-button-style:",
            "font-family: Fluent2Palette.font-family",
            "state-brush: Fluent2Palette.date-picker-selection-button-state-brush",
            "state-brush: Fluent2Palette.state",
        ),
    ] {
        let block = source
            .split(style_marker)
            .nth(1)
            .and_then(|after| after.split(next_marker).next())
            .unwrap_or_else(|| panic!("fluent2 datepicker.slint should define {style_marker}"));

        assert!(block.contains(expected), "fluent2 date picker style should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 date picker style should not bind popup controls directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_surface_backgrounds_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "menu-frame-background",
        "menu-flyout-background",
        "scrollbar-track-hover-background",
        "table-header-background",
        "time-clock-background",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for copied_alias in [
        "menu-flyout-background: alternate-background",
        "scrollbar-track-hover-background: alternate-background",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind surface semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    for (file, component_marker, expected, copied_literal) in [
        (
            "menu.slint",
            "export component MenuFrame",
            "background: Fluent2Palette.menu-frame-background",
            "background: Fluent2Palette.background",
        ),
        (
            "components.slint",
            "export component MenuBorder",
            "background: Fluent2Palette.menu-flyout-background",
            "background: Fluent2Palette.alternate-background",
        ),
        (
            "scrollview.slint",
            "component ScrollBar",
            "root.background: Fluent2Palette.scrollbar-track-hover-background",
            "root.background: Fluent2Palette.alternate-background",
        ),
        (
            "tableview.slint",
            "component TableViewColumn",
            "background: Fluent2Palette.table-header-background",
            "background: Fluent2Palette.background",
        ),
        (
            "time-picker.slint",
            "clock-style:",
            "background: Fluent2Palette.time-clock-background",
            "background: Fluent2Palette.background",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let block = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {file} should define {component_marker}"));

        assert!(block.contains(expected), "fluent2 {file} should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 {file} should not use the generic public background bridge for control surface backgrounds"
        );
    }
}

#[test]
fn test_fluent2_separator_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in ["table-header-separator", "tab-separator"] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for (file, component_marker, expected, copied_literal) in [
        (
            "tableview.slint",
            "component TableViewColumn",
            "background: Fluent2Palette.table-header-separator",
            "background: Fluent2Palette.divider",
        ),
        (
            "tabwidget.slint",
            "component TabImpl",
            "background: Fluent2Palette.tab-separator",
            "background: Fluent2Palette.divider",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let block = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {file} should define {component_marker}"));

        assert!(block.contains(expected), "fluent2 {file} should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 {file} should not use the generic divider token for control-specific separators"
        );
    }
}

#[test]
fn test_fluent2_tab_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "tab-background",
        "tab-hover-background",
        "tab-pressed-background",
        "tab-selected-background",
        "tab-border",
        "tab-foreground",
        "tab-selected-foreground",
        "tab-horizontal-padding",
        "tab-min-width",
        "tab-height",
        "tab-radius",
        "tab-separator-thickness",
        "tab-border-width",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "tab-background: control-fill-transparent",
        "tab-hover-background: layer-on-mica-base-alt-secondary",
        "tab-pressed-background: layer-on-mica-base-alt",
        "tab-selected-background: layer-on-mica-base-alt",
        "tab-border: card-stroke",
        "tab-foreground: text-secondary",
        "tab-selected-foreground: control-foreground",
    ] {
        assert!(
            !styling.contains(copied_bridge),
            "fluent2 tab semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tabwidget.slint"))
        .expect("fluent2 should embed tabwidget.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let tab_impl = source
        .split("export component TabImpl")
        .nth(1)
        .and_then(|after| after.split("component FluentTabBarBase").next())
        .expect("fluent2 tabwidget should define TabImpl before FluentTabBarBase");

    assert!(
        tab_impl.contains(
            "color: root.is-current ? Fluent2Palette.tab-selected-foreground : Fluent2Palette.tab-foreground"
        ),
        "fluent2 TabImpl label should use semantic tab foreground tokens"
    );

    assert!(
        tab_impl.contains(
            "background: i-touch-area.pressed ? Fluent2Palette.tab-pressed-background : root.is-current ? Fluent2Palette.tab-selected-background : i-touch-area.has-hover ? Fluent2Palette.tab-hover-background : Fluent2Palette.tab-background"
        ),
        "fluent2 TabImpl background should use semantic tab background tokens"
    );

    assert!(
        tab_impl.contains("border-color: Fluent2Palette.tab-border"),
        "fluent2 TabImpl border should use the semantic tab border token"
    );
    assert!(
        tab_impl.contains("y: Fluent2SizeSettings.tab-border-width"),
        "fluent2 TabImpl selected surface offset should use the semantic tab border width token"
    );
    assert!(
        tab_impl.contains("border-width: root.is-current ? Fluent2SizeSettings.tab-border-width : Fluent2SizeSettings.tab-hidden-border-width"),
        "fluent2 TabImpl selected border should use the semantic tab border width token"
    );
    assert!(
        tab_impl.contains("border-radius: Fluent2SizeSettings.tab-radius"),
        "fluent2 TabImpl should use a semantic tab radius token"
    );

    assert!(
        tab_impl.contains("padding-left: Fluent2SizeSettings.tab-horizontal-padding"),
        "fluent2 TabImpl should use a tab-specific left padding token"
    );
    assert!(
        tab_impl.contains("padding-right: Fluent2SizeSettings.tab-horizontal-padding"),
        "fluent2 TabImpl should use a tab-specific right padding token"
    );
    assert!(
        tab_impl.contains("min-width: max(Fluent2SizeSettings.tab-min-width, i-text.min-width)"),
        "fluent2 TabImpl should use a tab-specific min-width token"
    );
    assert!(
        tab_impl.contains("min-height: max(Fluent2SizeSettings.tab-height, i-text.min-height)"),
        "fluent2 TabImpl should use a tab-specific height token"
    );
    assert!(
        tab_impl.contains("width: Fluent2SizeSettings.tab-separator-thickness"),
        "fluent2 TabImpl right separator should use a tab-specific separator thickness token"
    );
    assert!(
        tab_impl.contains("height: Fluent2SizeSettings.tab-separator-thickness"),
        "fluent2 TabImpl bottom separator should use a tab-specific separator thickness token"
    );

    for copied_literal in [
        "Fluent2Palette.control-foreground",
        "Fluent2Palette.text-secondary",
        "Fluent2Palette.layer-on-mica-base-alt",
        "Fluent2Palette.layer-on-mica-base-alt-secondary",
        "Fluent2Palette.control-fill-transparent",
        "Fluent2Palette.card-stroke",
        "padding-left: Fluent2SizeSettings.control-horizontal-padding",
        "padding-right: Fluent2SizeSettings.control-horizontal-padding",
        "Fluent2SizeSettings.compact-item-height",
        "Fluent2SizeSettings.overlay-radius",
        "Fluent2SizeSettings.separator-thickness",
        "y: Fluent2SizeSettings.stroke-width",
        "border-width: root.is-current ? Fluent2SizeSettings.stroke-width : Fluent2SizeSettings.control-hidden-stroke-width",
    ] {
        assert!(
            !tab_impl.contains(copied_literal),
            "fluent2 TabImpl should not bind tab visuals directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_hidden_strokes_use_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "control-hidden-stroke-width",
        "checkbox-hidden-border-width",
        "switch-hidden-thumb-border-width",
        "switch-hidden-rail-border-width",
        "tab-hidden-border-width",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for (control, expected_tokens) in [
        ("checkbox.slint", &["Fluent2SizeSettings.checkbox-hidden-border-width"][..]),
        (
            "switch.slint",
            &[
                "Fluent2SizeSettings.switch-hidden-thumb-border-width",
                "Fluent2SizeSettings.switch-hidden-rail-border-width",
            ][..],
        ),
        ("tabwidget.slint", &["Fluent2SizeSettings.tab-hidden-border-width"][..]),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        for token in expected_tokens {
            assert!(source.contains(token), "fluent2 {control} should use {token}");
        }
        for copied_literal in [
            "Fluent2SizeSettings.control-hidden-stroke-width",
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
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "spinbox-button-hover-background",
        "spinbox-button-pressed-background",
        "spinbox-button-disabled-icon-foreground",
        "spinbox-button-radius",
        "spinbox-button-icon-size",
        "spinbox-button-width",
        "spinbox-button-spacing",
        "spinbox-button-motion-duration",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "spinbox-button-hover-background: subtle-secondary",
        "spinbox-button-pressed-background: subtle-tertiary",
        "spinbox-button-disabled-icon-foreground: text-disabled",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind spinbox button semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinbox.slint"))
        .expect("fluent2 should embed spinbox.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let button = source
        .split("component SpinBoxButton")
        .nth(1)
        .and_then(|after| after.split("export component SpinBox").next())
        .expect("fluent2 spinbox should define SpinBoxButton before SpinBox");

    assert!(
        source.contains("import { FocusBorder } from \"components.slint\""),
        "fluent2 SpinBoxButton should import the shared Fluent2 focus border"
    );

    for expected in [
        "in property <bool> enabled",
        "enabled: root.enabled",
        "out property <bool> has-focus: touch-area.has-focus",
        "FocusTouchArea",
        "if root.has-focus && root.enabled : FocusBorder",
        "border-radius: Fluent2SizeSettings.spinbox-button-radius",
        "min-width: Fluent2SizeSettings.spinbox-button-width",
        "width: Fluent2SizeSettings.spinbox-button-icon-size",
        "disabled when !root.enabled",
        "hover when touch-area.has-hover",
        "background.background: Fluent2Palette.spinbox-button-hover-background",
        "background.background: Fluent2Palette.spinbox-button-pressed-background",
        "icon.colorize: Fluent2Palette.spinbox-button-disabled-icon-foreground",
        "animate background { duration: Fluent2SizeSettings.spinbox-button-motion-duration",
        "animate colorize { duration: Fluent2SizeSettings.spinbox-button-motion-duration",
    ] {
        assert!(button.contains(expected), "fluent2 SpinBoxButton should use {expected}");
    }

    assert!(
        !button.contains("animate background, border-color"),
        "fluent2 SpinBoxButton should not animate border-color on a background-only layer"
    );
    assert!(
        !button.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 SpinBoxButton should not use the generic control motion token"
    );

    for copied_literal in [
        "Fluent2Palette.subtle-secondary",
        "Fluent2Palette.subtle-tertiary",
        "Fluent2Palette.text-disabled",
        "border-radius: Fluent2SizeSettings.control-radius",
        "width: Fluent2SizeSettings.icon-size",
        "min-width: Fluent2SizeSettings.compact-control-height",
    ] {
        assert!(
            !button.contains(copied_literal),
            "fluent2 SpinBoxButton should not bind button states directly to copied generic token {copied_literal}"
        );
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
    assert!(
        source.contains("spacing: Fluent2SizeSettings.spinbox-button-spacing"),
        "fluent2 SpinBox should use a semantic button spacing token"
    );
    assert!(
        !source.contains("spacing: Fluent2SizeSettings.overlay-padding"),
        "fluent2 SpinBox should not borrow generic overlay padding for button spacing"
    );
}

#[test]
fn test_fluent2_spinbox_base_uses_focus_touch_area_for_scroll() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/spinbox-base.slint"))
        .expect("fluent2 should embed spinbox-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "import { FocusTouchArea } from \"internal-components.slint\";",
        "touch-area := FocusTouchArea",
        "enabled: root.enabled && !root.read-only",
        "scroll-event(event) =>",
        "root.increment();",
        "root.decrement();",
    ] {
        assert!(source.contains(expected), "fluent2 SpinBoxBase should use {expected}");
    }

    assert!(
        !source.lines().any(|line| line.trim_start().starts_with("TouchArea")),
        "fluent2 SpinBoxBase should use FocusTouchArea instead of a bare TouchArea"
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

    for copied_alias in [
        "switch-thumb-disabled: text-secondary",
        "switch-thumb-checked-disabled: text-accent-foreground-disabled",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind switch disabled thumb semantic tokens directly, not through copied generic alias {copied_alias}"
        );
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
            "rail.background: root.checked ? Fluent2Palette.switch-rail-checked-disabled-background : Fluent2Palette.switch-rail-disabled-background"
        ),
        "fluent2 Switch disabled rail should use explicit checked and unchecked disabled rail tokens"
    );
}

#[test]
fn test_fluent2_switch_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "switch-foreground",
        "switch-disabled-foreground",
        "switch-rail-background",
        "switch-rail-hover-background",
        "switch-rail-pressed-background",
        "switch-rail-checked-background",
        "switch-rail-checked-hover-background",
        "switch-rail-checked-pressed-background",
        "switch-rail-disabled-background",
        "switch-rail-checked-disabled-background",
        "switch-rail-border",
        "switch-rail-border-width",
        "switch-rail-disabled-border",
        "switch-thumb-background",
        "switch-thumb-hover-background",
        "switch-thumb-pressed-background",
        "switch-thumb-checked-background",
        "switch-thumb-checked-hover-background",
        "switch-thumb-checked-pressed-background",
        "switch-thumb-border",
        "switch-thumb-border-width",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "switch-foreground: foreground",
        "switch-disabled-foreground: text-disabled",
        "switch-rail-checked-background: accent-background",
        "switch-rail-checked-hover-background: secondary-accent-background",
        "switch-rail-checked-pressed-background: tertiary-accent-background",
        "switch-rail-checked-disabled-background: accent-disabled",
        "switch-rail-border: control-strong-stroke",
        "switch-rail-disabled-border: control-strong-stroke-disabled",
        "switch-thumb-background: text-secondary",
        "switch-thumb-hover-background: text-secondary",
        "switch-thumb-pressed-background: text-secondary",
        "switch-thumb-checked-background: accent-foreground",
        "switch-thumb-checked-hover-background: accent-foreground",
        "switch-thumb-checked-pressed-background: accent-foreground",
        "switch-thumb-border: circle-border",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 switch semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    for expected in [
        "property <brush> switch-rail-rest-fill: control-alt-secondary;",
        "property <brush> switch-rail-hover-fill: control-alt-tertiary;",
        "property <brush> switch-rail-pressed-fill: control-alt-quaternary;",
        "switch-rail-background: switch-rail-rest-fill",
        "switch-rail-hover-background: switch-rail-hover-fill",
        "switch-rail-pressed-background: switch-rail-pressed-fill",
    ] {
        let expected_line = format!("out property <brush> {expected};");
        assert!(
            styling.lines().any(|line| line.trim() == expected || line.trim() == expected_line),
            "fluent2 switch rail fills should route through switch-specific Fluent2 primitives: {expected}"
        );
    }

    for copied_bridge in [
        "switch-rail-background: control-alt-secondary",
        "switch-rail-hover-background: control-alt-tertiary",
        "switch-rail-pressed-background: control-alt-quaternary",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 switch rail semantic tokens should not alias through copied generic control-alt bridge {copied_bridge}"
        );
    }

    for copied_literal in [
        "switch-rail-background: dark-color-scheme ? #0000001A : #00000005",
        "switch-rail-hover-background: dark-color-scheme ? #FFFFFF0A : #0000000F",
        "switch-rail-pressed-background: dark-color-scheme ? #FFFFFF12 : #00000017",
    ] {
        let copied_literal_line = format!("out property <brush> {copied_literal};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_literal_line),
            "fluent2 switch rail alternate fills should not repeat copied raw light/dark alpha branches: {copied_literal}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "private property <color> text-color: Fluent2Palette.switch-foreground",
        "rail.background: root.checked ? Fluent2Palette.switch-rail-checked-disabled-background : Fluent2Palette.switch-rail-disabled-background",
        "rail.border-color: Fluent2Palette.switch-rail-disabled-border",
        "root.text-color: Fluent2Palette.switch-disabled-foreground",
        "rail.background: root.checked ? Fluent2Palette.switch-rail-checked-pressed-background : Fluent2Palette.switch-rail-pressed-background",
        "thumb.background: root.checked ? Fluent2Palette.switch-thumb-checked-pressed-background : Fluent2Palette.switch-thumb-pressed-background",
        "rail.background:  root.checked ? Fluent2Palette.switch-rail-checked-hover-background : Fluent2Palette.switch-rail-hover-background",
        "thumb.background: root.checked ? Fluent2Palette.switch-thumb-checked-hover-background : Fluent2Palette.switch-thumb-hover-background",
        "rail.background: Fluent2Palette.switch-rail-checked-background",
        "thumb.border-color: Fluent2Palette.switch-thumb-border",
        "thumb.border-width: Fluent2SizeSettings.switch-thumb-border-width",
        "thumb.background: Fluent2Palette.switch-thumb-checked-background",
        "border-width: root.checked ? Fluent2SizeSettings.switch-hidden-rail-border-width : Fluent2SizeSettings.switch-rail-border-width",
        "border-color: Fluent2Palette.switch-rail-border",
        "background: Fluent2Palette.switch-rail-background",
        "background: Fluent2Palette.switch-thumb-background",
    ] {
        assert!(source.contains(expected), "fluent2 Switch should use {expected}");
    }

    for copied_literal in [
        "thumb.border-width: root.checked ? Fluent2SizeSettings.stroke-width : Fluent2SizeSettings.control-hidden-stroke-width",
        "thumb.border-width: root.checked ? Fluent2SizeSettings.switch-thumb-border-width : Fluent2SizeSettings.control-hidden-stroke-width",
        "thumb.border-width: Fluent2SizeSettings.stroke-width",
        "border-width: root.checked ? Fluent2SizeSettings.control-hidden-stroke-width : Fluent2SizeSettings.switch-rail-border-width",
        "border-width: root.checked ? Fluent2SizeSettings.control-hidden-stroke-width : Fluent2SizeSettings.stroke-width",
    ] {
        assert!(
            !source.contains(copied_literal),
            "fluent2 Switch should not bind stroke widths directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_switch_radius_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in ["switch-rail-radius", "switch-thumb-radius"] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "border-radius: Fluent2SizeSettings.switch-rail-radius",
        "border-radius: Fluent2SizeSettings.switch-thumb-radius",
    ] {
        assert!(source.contains(expected), "fluent2 Switch should use {expected}");
    }

    assert!(
        !source.contains("border-radius: self.height / 2"),
        "fluent2 Switch radius geometry should not be derived from live element height"
    );
}

#[test]
fn test_fluent2_switch_motion_uses_switch_duration_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("switch-motion-duration"),
        "fluent2 styling should expose a semantic switch motion duration token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let rail_block = source
        .split("rail := Rectangle")
        .nth(1)
        .and_then(|after| after.split("thumb := Rectangle").next())
        .expect("fluent2 Switch should define a rail rectangle before thumb");
    let thumb_block = source
        .split("thumb := Rectangle")
        .nth(1)
        .and_then(|after| after.split("// focus border").next())
        .expect("fluent2 Switch should define a thumb rectangle before focus border");

    assert!(
        thumb_block.contains("animate background, border-color, border-width, x, y, width, height"),
        "fluent2 Switch thumb should animate movement and size changes, not only color/width"
    );
    assert!(
        rail_block.contains("duration: Fluent2SizeSettings.switch-motion-duration"),
        "fluent2 Switch rail motion should use the semantic switch motion token"
    );
    assert!(
        thumb_block.contains("duration: Fluent2SizeSettings.switch-motion-duration"),
        "fluent2 Switch thumb motion should use the semantic switch motion token"
    );
    assert!(
        !rail_block.contains("duration: Fluent2SizeSettings.control-motion-duration")
            && !thumb_block.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 Switch rail/thumb motion should not borrow the generic control motion token"
    );
}

#[test]
fn test_fluent2_switch_stroke_motion_is_on_stroke_elements() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/switch.slint"))
        .expect("fluent2 should embed switch.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let rail_block = source
        .split("rail := Rectangle")
        .nth(1)
        .and_then(|after| after.split("thumb := Rectangle").next())
        .expect("fluent2 Switch should define a rail rectangle before thumb");
    let thumb_block = source
        .split("thumb := Rectangle")
        .nth(1)
        .and_then(|after| after.split("// focus border").next())
        .expect("fluent2 Switch should define a thumb rectangle before focus border");

    assert!(
        rail_block
            .contains("animate background, border-color { duration: Fluent2SizeSettings.switch-motion-duration"),
        "fluent2 Switch rail should animate border-color on the element that owns the rail stroke"
    );
    assert!(
        thumb_block.contains(
            "animate background, border-color, border-width, x, y, width, height { duration: Fluent2SizeSettings.switch-motion-duration"
        ),
        "fluent2 Switch thumb should animate border-color and border-width on the element that owns the thumb stroke"
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

    let about = load_file(&std::path::PathBuf::from("builtin:/fluent2/about-slint.slint"))
        .expect("fluent2 should embed about-slint.slint");
    let about_contents = about.read();
    let about = std::str::from_utf8(&about_contents).unwrap();

    assert!(
        about.contains("about-layout-spacing"),
        "fluent2 styling should expose an AboutSlint-specific spacing token"
    );
    assert!(
        about.contains("spacing: Fluent2SizeSettings.about-layout-spacing"),
        "fluent2 AboutSlint should use its own layout spacing token"
    );
    assert!(
        !about.contains("spacing: Fluent2SizeSettings.control-spacing"),
        "fluent2 AboutSlint should not use generic control spacing"
    );

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
fn test_fluent2_lineedit_icon_state_layer_radius_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("lineedit-icon-state-layer-radius"),
        "fluent2 styling should expose a line edit icon state layer radius token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit-base.slint"))
        .expect("fluent2 should embed lineedit-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for component in ["LineEditClearIcon", "LineEditPasswordIcon"] {
        let block = source
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|after| after.split("export component ").next())
            .unwrap_or_else(|| panic!("fluent2 lineedit-base should define {component}"));

        assert!(
            block.contains("border-radius: Fluent2SizeSettings.lineedit-icon-state-layer-radius"),
            "fluent2 {component} state layer should use the line edit icon radius token"
        );
        assert!(
            !block.contains("border-radius: max(root.width, root.height) / 2"),
            "fluent2 {component} state layer radius should not be derived from live geometry"
        );
    }
}

#[test]
fn test_fluent2_lineedit_icon_state_brush_uses_semantic_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("lineedit-icon-state-brush"),
        "fluent2 styling should expose a semantic line edit icon state brush token"
    );
    assert!(
        !styling
            .lines()
            .any(|line| line.trim() == "out property <brush> lineedit-icon-state-brush: state;"),
        "fluent2 line edit icon state brush should not alias through the copied generic state token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit-base.slint"))
        .expect("fluent2 should embed lineedit-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for component in ["LineEditClearIcon", "LineEditPasswordIcon"] {
        let block = source
            .split(&format!("export component {component}"))
            .nth(1)
            .and_then(|after| after.split("export component ").next())
            .unwrap_or_else(|| panic!("fluent2 lineedit-base should define {component}"));

        assert!(
            block.contains("state-brush: Fluent2Palette.lineedit-icon-state-brush"),
            "fluent2 {component} should default to the semantic line edit icon state brush"
        );
        assert!(
            !block.contains("state-brush: Fluent2Palette.state"),
            "fluent2 {component} should not bind directly to the generic state brush"
        );
    }
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
    let input_style = source
        .split("export struct TimePickerInputStyle")
        .nth(1)
        .and_then(|after| after.split("export component TimePickerInput").next())
        .expect("fluent2 time picker should define TimePickerInputStyle");
    for expected in ["state-brush: brush", "state-brush-selected: brush"] {
        assert!(
            input_style.contains(expected),
            "fluent2 TimePickerInputStyle should use {expected}"
        );
    }

    for component in ["TimeSelector", "TimePickerInput", "PeriodSelectorItem"] {
        let block = source
            .split(&format!("component {component}"))
            .nth(1)
            .and_then(|after| after.split("component ").next())
            .unwrap_or_else(|| panic!("fluent2 time picker should define {component}"));

        let mut expected = vec!["FocusTouchArea", "StateLayer"];
        if component == "TimePickerInput" {
            expected.extend([
                "enabled: root.read-only",
                "pressed: touch-area.pressed",
                "has-hover: touch-area.has-hover",
                "has-focus: touch-area.has-focus",
                "state-brush: root.style.state-brush",
                "state-layer.state-brush: root.style.state-brush-selected",
            ]);
        } else {
            expected.extend([
                "in property <bool> enabled",
                "enabled: root.enabled",
                "pressed: touch-area.pressed",
                "has-hover: touch-area.has-hover",
                "has-focus: touch-area.has-focus",
                "state-brush: root.style.state-brush",
            ]);
        }

        for expected in expected {
            assert!(block.contains(expected), "fluent2 {component} should use {expected}");
        }

        assert!(
            !block.lines().any(|line| line.trim_start().starts_with("TouchArea")),
            "fluent2 {component} should use FocusTouchArea instead of a bare TouchArea"
        );
    }

    let popup = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker.slint"))
        .expect("fluent2 should embed time-picker.slint");
    let popup_contents = popup.read();
    let popup = std::str::from_utf8(&popup_contents).unwrap();

    for expected in [
        "state-brush: Fluent2Palette.time-picker-selector-state-brush",
        "state-brush-selected: Fluent2Palette.time-picker-selector-selected-state-brush",
        "state-brush: Fluent2Palette.time-picker-input-state-brush",
        "state-brush-selected: Fluent2Palette.time-picker-input-selected-state-brush",
        "state-brush: Fluent2Palette.time-picker-period-item-state-brush",
        "state-brush-selected: Fluent2Palette.time-picker-period-item-selected-state-brush",
    ] {
        assert!(popup.contains(expected), "fluent2 TimePickerPopup should wire {expected}");
    }
}

#[test]
fn test_fluent2_time_picker_selector_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "time-picker-selector-foreground",
        "time-picker-selector-selected-foreground",
        "time-picker-selector-state-brush",
        "time-picker-selector-selected-state-brush",
        "time-picker-input-background",
        "time-picker-input-selected-background",
        "time-picker-input-foreground",
        "time-picker-input-selected-foreground",
        "time-picker-input-state-brush",
        "time-picker-input-selected-state-brush",
        "time-picker-period-item-foreground",
        "time-picker-period-item-selected-background",
        "time-picker-period-item-selected-foreground",
        "time-picker-period-item-state-brush",
        "time-picker-period-item-selected-state-brush",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "time-picker-selector-foreground: foreground",
        "time-picker-selector-selected-foreground: accent-foreground",
        "time-picker-selector-state-brush: state",
        "time-picker-selector-selected-state-brush: state-secondary",
        "time-picker-input-background: control-background",
        "time-picker-input-selected-background: accent-background",
        "time-picker-input-foreground: foreground",
        "time-picker-input-selected-foreground: accent-foreground",
        "time-picker-input-state-brush: state",
        "time-picker-input-selected-state-brush: state-secondary",
        "time-picker-period-item-foreground: foreground",
        "time-picker-period-item-selected-background: accent-background",
        "time-picker-period-item-selected-foreground: accent-foreground",
        "time-picker-period-item-state-brush: state",
        "time-picker-period-item-selected-state-brush: state-secondary",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind time picker selector semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker.slint"))
        .expect("fluent2 should embed time-picker.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for (style_marker, next_marker, expected, copied_literal) in [
        (
            "time-selector-style:",
            "font-size: Fluent2FontSettings.body-strong.font-size",
            "foreground: Fluent2Palette.time-picker-selector-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "time-selector-style:",
            "font-size: Fluent2FontSettings.body-strong.font-size",
            "foreground-selected: Fluent2Palette.time-picker-selector-selected-foreground",
            "foreground-selected: Fluent2Palette.accent-foreground",
        ),
        (
            "time-selector-style:",
            "font-size: Fluent2FontSettings.body-strong.font-size",
            "state-brush: Fluent2Palette.time-picker-selector-state-brush",
            "state-brush: Fluent2Palette.state",
        ),
        (
            "time-selector-style:",
            "font-size: Fluent2FontSettings.body-strong.font-size",
            "state-brush-selected: Fluent2Palette.time-picker-selector-selected-state-brush",
            "state-brush-selected: Fluent2Palette.state-secondary",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "background: Fluent2Palette.time-picker-input-background",
            "background: Fluent2Palette.control-background",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "background-selected: Fluent2Palette.time-picker-input-selected-background",
            "background-selected: Fluent2Palette.accent-background",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "foreground: Fluent2Palette.time-picker-input-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "foreground-selected: Fluent2Palette.time-picker-input-selected-foreground",
            "foreground-selected: Fluent2Palette.accent-foreground",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "state-brush: Fluent2Palette.time-picker-input-state-brush",
            "state-brush: Fluent2Palette.state",
        ),
        (
            "input-style:",
            "border-radius: Fluent2SizeSettings.overlay-radius",
            "state-brush-selected: Fluent2Palette.time-picker-input-selected-state-brush",
            "state-brush-selected: Fluent2Palette.state-secondary",
        ),
        (
            "item-style:",
            "title-style:",
            "foreground: Fluent2Palette.time-picker-period-item-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "item-style:",
            "title-style:",
            "background-selected: Fluent2Palette.time-picker-period-item-selected-background",
            "background-selected: Fluent2Palette.accent-background",
        ),
        (
            "item-style:",
            "title-style:",
            "foreground-selected: Fluent2Palette.time-picker-period-item-selected-foreground",
            "foreground-selected: Fluent2Palette.accent-foreground",
        ),
        (
            "item-style:",
            "title-style:",
            "state-brush: Fluent2Palette.time-picker-period-item-state-brush",
            "state-brush: Fluent2Palette.state",
        ),
        (
            "item-style:",
            "title-style:",
            "state-brush-selected: Fluent2Palette.time-picker-period-item-selected-state-brush",
            "state-brush-selected: Fluent2Palette.state-secondary",
        ),
    ] {
        let block = source
            .split(style_marker)
            .nth(1)
            .and_then(|after| after.split(next_marker).next())
            .unwrap_or_else(|| panic!("fluent2 time-picker.slint should define {style_marker}"));

        assert!(block.contains(expected), "fluent2 time picker style should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 time picker style should not bind selector colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_time_picker_popup_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "time-picker-popup-foreground",
        "time-picker-clock-foreground",
        "time-picker-title-foreground",
        "time-input-radius",
        "time-period-selector-radius",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_alias in [
        "time-picker-popup-foreground: foreground",
        "time-picker-clock-foreground: accent-background",
        "time-picker-title-foreground: foreground",
    ] {
        let copied_alias_line = format!("out property <brush> {copied_alias};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_alias_line),
            "fluent2 styling should bind time picker popup semantic tokens directly, not through copied generic alias {copied_alias}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker.slint"))
        .expect("fluent2 should embed time-picker.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for (style_marker, next_marker, expected, copied_literal) in [
        (
            "style: {",
            "vertical-spacing: Fluent2SizeSettings.picker-vertical-spacing",
            "foreground: Fluent2Palette.time-picker-popup-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "clock-style:",
            "time-selector-style:",
            "foreground: Fluent2Palette.time-picker-clock-foreground",
            "foreground: Fluent2Palette.accent-background",
        ),
        (
            "title-style:",
            "};",
            "foreground: Fluent2Palette.time-picker-title-foreground",
            "foreground: Fluent2Palette.foreground",
        ),
        (
            "input-style:",
            "period-selector-style:",
            "border-radius: Fluent2SizeSettings.time-input-radius",
            "border-radius: Fluent2SizeSettings.overlay-radius",
        ),
        (
            "period-selector-style:",
            "item-style:",
            "border-radius: Fluent2SizeSettings.time-period-selector-radius",
            "border-radius: Fluent2SizeSettings.overlay-radius",
        ),
    ] {
        let block = source
            .split(style_marker)
            .nth(1)
            .and_then(|after| after.split(next_marker).next())
            .unwrap_or_else(|| panic!("fluent2 time-picker.slint should define {style_marker}"));

        assert!(block.contains(expected), "fluent2 time picker style should use {expected}");
        assert!(
            !block.contains(copied_literal),
            "fluent2 time picker style should not bind popup colors directly to copied generic token {copied_literal}"
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
fn test_fluent2_combobox_base_uses_focus_touch_area() {
    let helper = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let helper_contents = helper.read();
    let helper = std::str::from_utf8(&helper_contents).unwrap();

    let focus_touch_area = helper
        .split("export component FocusTouchArea")
        .nth(1)
        .and_then(|after| after.split("export component IconButton").next())
        .expect("fluent2 should define FocusTouchArea");

    assert!(
        focus_touch_area.contains("callback scroll-event <=> touch-area.scroll-event"),
        "fluent2 FocusTouchArea should forward scroll events for composite controls"
    );

    let base = load_file(&std::path::PathBuf::from("builtin:/fluent2/combobox-base.slint"))
        .expect("fluent2 should own combobox-base.slint");
    let base_contents = base.read();
    let base = std::str::from_utf8(&base_contents).unwrap();

    for expected in [
        "import { FocusTouchArea } from \"internal-components.slint\";",
        "i-touch-area := FocusTouchArea",
        "enabled: root.enabled",
        "scroll-event(event) =>",
        "root.focus();",
        "root.show-popup();",
    ] {
        assert!(base.contains(expected), "fluent2 ComboBoxBase should use {expected}");
    }

    assert!(
        !base.contains("i-touch-area := TouchArea"),
        "fluent2 ComboBoxBase should use the shared focus/touch helper instead of a bare TouchArea"
    );
}

#[test]
fn test_fluent2_menu_items_use_focus_touch_area() {
    let helper = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let helper_contents = helper.read();
    let helper = std::str::from_utf8(&helper_contents).unwrap();

    let focus_touch_area = helper
        .split("export component FocusTouchArea")
        .nth(1)
        .and_then(|after| after.split("export component IconButton").next())
        .expect("fluent2 should define FocusTouchArea");

    assert!(
        focus_touch_area.contains("@children"),
        "fluent2 FocusTouchArea should preserve child-content layout for composite controls"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/menu-base.slint"))
        .expect("fluent2 should embed menu-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("import { FocusTouchArea } from \"internal-components.slint\";"),
        "fluent2 menu-base should import the shared interaction helper"
    );

    let menu_bar_item = source
        .split("export component MenuBarItemBase")
        .nth(1)
        .and_then(|after| after.split("export component MenuBarBase").next())
        .expect("fluent2 should define MenuBarItemBase");
    let menu_item = source
        .split("export component MenuItemBase")
        .nth(1)
        .expect("fluent2 should define MenuItemBase");

    for (name, block) in [("MenuBarItemBase", menu_bar_item), ("MenuItemBase", menu_item)] {
        for expected in
            ["touch-area := FocusTouchArea", "pointer-event(event) =>", "changed has-hover =>"]
        {
            assert!(block.contains(expected), "fluent2 {name} should use {expected}");
        }

        assert!(
            !block.contains("touch-area := TouchArea"),
            "fluent2 {name} should use FocusTouchArea instead of a bare TouchArea"
        );
    }

    for expected in ["pressed when touch-area.pressed", "has-hover when touch-area.has-hover"] {
        assert!(menu_bar_item.contains(expected), "fluent2 MenuBarItemBase should use {expected}");
    }

    assert!(
        menu_item.contains("is-current when root.is-current"),
        "fluent2 MenuItemBase should keep current-item menu selection semantics"
    );
}

#[test]
fn test_fluent2_picker_radius_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "date-delegate-radius",
        "time-selector-radius",
        "time-clock-radius",
        "time-clock-coordinate-radius",
        "time-clock-center-dot-radius",
        "time-clock-current-selector-radius",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }
    for stale_token in ["time-clock-inner-dot-size", "time-clock-inner-dot-radius"] {
        assert!(
            !styling.contains(stale_token),
            "fluent2 styling should not keep stale {stale_token} after removing negative-index selector chrome"
        );
    }

    let datepicker = load_file(&std::path::PathBuf::from("builtin:/fluent2/datepicker-base.slint"))
        .expect("fluent2 should embed datepicker-base.slint");
    let datepicker_contents = datepicker.read();
    let datepicker = std::str::from_utf8(&datepicker_contents).unwrap();

    assert!(
        datepicker.contains("border-radius: Fluent2SizeSettings.date-delegate-radius"),
        "fluent2 date picker delegates should use date-delegate-radius"
    );
    assert!(
        !datepicker.contains("border-radius: self.height / 2"),
        "fluent2 date picker delegate radius should not be derived from live element height"
    );

    let timepicker =
        load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker-base.slint"))
            .expect("fluent2 should embed time-picker-base.slint");
    let timepicker_contents = timepicker.read();
    let timepicker = std::str::from_utf8(&timepicker_contents).unwrap();

    for expected in [
        "border-radius: Fluent2SizeSettings.time-selector-radius",
        "border-radius: Fluent2SizeSettings.time-clock-radius",
        "property <length> radius: Fluent2SizeSettings.time-clock-coordinate-radius",
        "border-radius: Fluent2SizeSettings.time-clock-center-dot-radius",
        "border-radius: Fluent2SizeSettings.time-clock-current-selector-radius",
    ] {
        assert!(timepicker.contains(expected), "fluent2 time picker should use {expected}");
    }

    for copied_geometry in [
        "border-radius: max(root.width, root.height) / 2",
        "property <length> radius: max(root.width, root.height) / 2",
        "border-radius: max(self.width, self.height) / 2",
        "border-radius: self.width / 2",
        "border-radius: root.picker-ditameter / 2",
    ] {
        assert!(
            !timepicker.contains(copied_geometry),
            "fluent2 time picker radius geometry should not use copied expression {copied_geometry}"
        );
    }
}

#[test]
fn test_fluent2_time_picker_clock_selection_uses_bounded_index() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/time-picker-base.slint"))
        .expect("fluent2 should embed time-picker-base.slint");
    let contents = source.read();
    let source = std::str::from_utf8(&contents).unwrap();

    assert!(
        source.contains("if root.current-item >= 0 && root.current-item < root.model.length: Path"),
        "fluent2 time picker clock hand should render only for a bounded selected index"
    );
    assert!(
        !source
            .contains("if root.current-item >= 0 || root.current-item < root.model.length: Path"),
        "fluent2 time picker clock hand should not use an always-true lower/upper bound union"
    );
    assert!(
        source.contains(
            "if root.current-item >= 0 && root.current-item < root.model.length: Rectangle"
        ),
        "fluent2 time picker current selector should render only for a bounded selected index"
    );
    assert!(
        !source.contains("if root.current-item < root.model.length: Rectangle"),
        "fluent2 time picker current selector should not render for negative current indexes"
    );
    assert!(
        !source.contains("if root.current-item < 0: Rectangle"),
        "fluent2 time picker current selector should not keep unreachable negative-index selector chrome"
    );
}

#[test]
fn test_fluent2_picker_icon_state_layer_radius_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("picker-icon-state-layer-radius"),
        "fluent2 styling should expose a picker icon state layer radius token"
    );
    assert!(
        styling.contains("picker-selection-state-layer-radius"),
        "fluent2 styling should expose a picker selection state layer radius token"
    );
    assert!(
        styling.contains("picker-selection-motion-duration"),
        "fluent2 styling should expose a picker selection motion duration token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let block = source
        .split("export component IconButton")
        .nth(1)
        .and_then(|after| after.split("export struct SelectionButtonStyle").next())
        .expect("fluent2 internal components should define IconButton");

    assert!(
        block.contains("border-radius: Fluent2SizeSettings.picker-icon-state-layer-radius"),
        "fluent2 IconButton state layer should use the picker icon radius token"
    );
    assert!(
        !block.contains("border-radius: max(self.width, self.height) / 2"),
        "fluent2 IconButton state layer radius should not be derived from live geometry"
    );

    let block = source
        .split("export component SelectionButton")
        .nth(1)
        .and_then(|after| after.split("export struct ColoredTextStyle").next())
        .expect("fluent2 internal components should define SelectionButton");

    assert!(
        block.contains("border-radius: Fluent2SizeSettings.picker-selection-state-layer-radius"),
        "fluent2 SelectionButton state layer should use the picker selection radius token"
    );
    assert!(
        block.contains("animate transform-rotation { duration: Fluent2SizeSettings.picker-selection-motion-duration; }"),
        "fluent2 SelectionButton icon rotation should use the picker selection motion token"
    );
    assert!(
        !block.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 SelectionButton should not borrow the generic control motion token"
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
fn test_fluent2_table_rows_use_focus_border() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("import { FocusBorder } from \"components.slint\""),
        "fluent2 table rows should import the shared Fluent2 focus border"
    );

    let row_block = source
        .split("component TableViewRow")
        .nth(1)
        .and_then(|after| after.split("export component StandardTableView").next())
        .expect("fluent2 should define TableViewRow");

    for expected in [
        "in property <bool> has-focus",
        "min-height: max(Fluent2SizeSettings.table-row-height, layout.min-height)",
        "border-radius: Fluent2SizeSettings.table-row-radius",
        "if root.has-focus : FocusBorder",
    ] {
        assert!(row_block.contains(expected), "fluent2 TableViewRow should use {expected}");
    }

    for copied_literal in [
        "min-height: max(Fluent2SizeSettings.item-height, layout.min-height)",
        "border-radius: Fluent2SizeSettings.control-radius",
    ] {
        assert!(
            !row_block.contains(copied_literal),
            "fluent2 TableViewRow should not borrow copied generic geometry token {copied_literal}"
        );
    }

    let table_block = source
        .split("export component StandardTableView")
        .nth(1)
        .expect("fluent2 should define StandardTableView");
    assert!(
        table_block.contains("has-focus: root.has-focus && idx == root.current-row"),
        "fluent2 StandardTableView should pass keyboard focus to the current row"
    );
}

#[test]
fn test_fluent2_table_headers_use_focus_touch_area() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tableview.slint"))
        .expect("fluent2 should embed tableview.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    let block = source
        .split("component TableViewColumn")
        .nth(1)
        .and_then(|after| after.split("component TableViewCell").next())
        .expect("fluent2 should define TableViewColumn");

    for expected in [
        "callback clicked <=> touch-area.clicked",
        "touch-area := FocusTouchArea",
        "enabled: true",
        "pressed when touch-area.pressed",
        "hover when touch-area.has-hover",
        "width: parent.width - Fluent2SizeSettings.table-header-horizontal-padding",
    ] {
        assert!(block.contains(expected), "fluent2 TableViewColumn should use {expected}");
    }

    assert!(
        !block.contains("width: parent.width - Fluent2SizeSettings.control-horizontal-padding"),
        "fluent2 TableViewColumn focus receiver should not use the generic control padding token"
    );

    assert!(
        !block.lines().any(|line| line.trim_start().starts_with("touch-area := TouchArea")),
        "fluent2 TableViewColumn should use FocusTouchArea instead of a bare sort TouchArea"
    );
    assert!(
        block.contains("movable-touch-area := TouchArea"),
        "fluent2 TableViewColumn should keep the resize grip as a drag-specific TouchArea"
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
fn test_fluent2_button_border_uses_semantic_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("button-border"),
        "fluent2 styling should expose a button border token"
    );
    assert!(
        styling.contains("button-border-width"),
        "fluent2 styling should expose a button border width token"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/button.slint"))
        .expect("fluent2 should embed button.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let button =
        source.split("export component Button").nth(1).expect("fluent2 should define Button");

    assert!(
        button.contains("Fluent2Palette.button-border"),
        "fluent2 Button should use the semantic button border token"
    );
    assert!(
        button.contains("border-color: root.primary ? Fluent2Palette.button-primary-border : Fluent2Palette.button-border"),
        "fluent2 Button resting border should use semantic button border tokens"
    );
    assert!(
        button.contains("border-width: Fluent2SizeSettings.button-border-width"),
        "fluent2 Button resting border should use the semantic button border width token"
    );
    assert!(
        !button.contains("Fluent2Palette.border"),
        "fluent2 Button should not use the generic border bridge token for button-specific state borders"
    );
    assert!(
        !button.contains("Fluent2Palette.control-border"),
        "fluent2 Button should not bind state borders directly to the generic control-border token"
    );
    assert!(
        !button.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 Button should not bind border width directly to the generic stroke-width token"
    );
}

#[test]
fn test_fluent2_button_motion_uses_button_token() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    assert!(
        styling.contains("button-motion-duration"),
        "fluent2 styling should expose button-motion-duration"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/button.slint"))
        .expect("fluent2 should embed button.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let button =
        source.split("export component Button").nth(1).expect("fluent2 should define Button");

    for expected in [
        "animate background { duration: Fluent2SizeSettings.button-motion-duration",
        "animate border-color { duration: Fluent2SizeSettings.button-motion-duration",
        "animate color { duration: Fluent2SizeSettings.button-motion-duration",
    ] {
        assert!(button.contains(expected), "fluent2 Button should use {expected}");
    }

    assert!(
        !button.contains("duration: Fluent2SizeSettings.control-motion-duration"),
        "fluent2 Button should not use the generic control motion token"
    );
}

#[test]
fn test_fluent2_state_border_motion_is_on_stroke_elements() {
    for (control, component_marker, border_marker, duration_token) in [
        (
            "button.slint",
            "export component Button",
            "i-border := Rectangle",
            "button-motion-duration",
        ),
        (
            "checkbox.slint",
            "export component CheckBox",
            "border := Rectangle",
            "checkbox-motion-duration",
        ),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let component = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {control} should define {component_marker}"));
        let border_block = component
            .split(border_marker)
            .nth(1)
            .and_then(|after| after.split("}").next())
            .unwrap_or_else(|| panic!("fluent2 {control} should define {border_marker}"));

        assert!(
            border_block.contains(&format!(
                "animate border-color {{ duration: Fluent2SizeSettings.{duration_token}"
            )),
            "fluent2 {control} should animate border-color on the element that owns the stroke"
        );
    }

    for (control, component_marker) in [
        ("button.slint", "export component Button"),
        ("checkbox.slint", "export component CheckBox"),
        ("spinbox.slint", "component SpinBoxButton"),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{control}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {control}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();
        let component = source
            .split(component_marker)
            .nth(1)
            .unwrap_or_else(|| panic!("fluent2 {control} should define {component_marker}"));

        assert!(
            !component.contains("animate background, border-color"),
            "fluent2 {control} should not animate border-color on a parent background layer"
        );
    }
}

#[test]
fn test_fluent2_button_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for token in [
        "button-background",
        "button-hover-background",
        "button-pressed-background",
        "button-disabled-background",
        "button-foreground",
        "button-hover-foreground",
        "button-pressed-foreground",
        "button-disabled-foreground",
        "button-primary-background",
        "button-primary-hover-background",
        "button-primary-pressed-background",
        "button-primary-disabled-background",
        "button-primary-foreground",
        "button-primary-hover-foreground",
        "button-primary-pressed-foreground",
        "button-primary-disabled-foreground",
        "button-primary-border",
        "button-primary-disabled-border",
        "button-icon-transparent-foreground",
        "button-min-width",
        "button-height",
        "button-container-radius",
        "button-horizontal-padding",
        "button-vertical-padding",
        "button-content-spacing",
    ] {
        assert!(styling.contains(token), "fluent2 styling should expose {token}");
    }

    for copied_bridge in [
        "button-background: control-background",
        "button-hover-background: control-secondary",
        "button-pressed-background: control-tertiary",
        "button-disabled-background: control-disabled",
        "button-foreground: control-foreground",
        "button-hover-foreground: control-foreground",
        "button-pressed-foreground: text-secondary",
        "button-disabled-foreground: text-disabled",
        "button-primary-background: accent-background",
        "button-primary-hover-background: secondary-accent-background",
        "button-primary-pressed-background: tertiary-accent-background",
        "button-primary-disabled-background: accent-disabled",
        "button-primary-foreground: accent-foreground",
        "button-primary-hover-foreground: accent-foreground",
        "button-primary-pressed-foreground: text-accent-foreground-secondary",
        "button-primary-disabled-foreground: text-accent-foreground-disabled",
        "button-primary-border: accent-control-border",
        "button-primary-disabled-border: control-fill-transparent",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 button semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/button.slint"))
        .expect("fluent2 should embed button.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();
    let button =
        source.split("export component Button").nth(1).expect("fluent2 should define Button");

    for expected in [
        "private property <brush> text-color: primary || checked ? Fluent2Palette.button-primary-foreground : Fluent2Palette.button-foreground",
        "i-background.background: root.primary || root.checked ? Fluent2Palette.button-primary-disabled-background : Fluent2Palette.button-disabled-background",
        "i-border.border-color: root.primary || root.checked ? Fluent2Palette.button-primary-disabled-border : Fluent2Palette.button-border",
        "root.text-color: root.primary || root.checked ? Fluent2Palette.button-primary-disabled-foreground : Fluent2Palette.button-disabled-foreground",
        "i-background.background: root.primary || root.checked ? Fluent2Palette.button-primary-pressed-background : Fluent2Palette.button-pressed-background",
        "root.text-color: root.primary || root.checked ? Fluent2Palette.button-primary-pressed-foreground : Fluent2Palette.button-pressed-foreground",
        "i-background.background: root.primary || root.checked ? Fluent2Palette.button-primary-hover-background : Fluent2Palette.button-hover-background",
        "root.text-color: root.primary || root.checked ? Fluent2Palette.button-primary-hover-foreground : Fluent2Palette.button-hover-foreground",
        "i-background.background: Fluent2Palette.button-primary-background",
        "i-border.border-color: Fluent2Palette.button-primary-border",
        "root.text-color: Fluent2Palette.button-primary-foreground",
        "background: root.primary ? Fluent2Palette.button-primary-background : Fluent2Palette.button-background",
        "border-color: root.primary ? Fluent2Palette.button-primary-border : Fluent2Palette.button-border",
        "colorize: root.colorize-icon ? root.text-color : Fluent2Palette.button-icon-transparent-foreground",
        "min-width: max(Fluent2SizeSettings.button-min-width, i-layout.min-width)",
        "min-height: max(Fluent2SizeSettings.button-height, i-layout.min-height)",
        "border-radius: Fluent2SizeSettings.button-container-radius",
        "padding-left: Fluent2SizeSettings.button-horizontal-padding",
        "padding-right: Fluent2SizeSettings.button-horizontal-padding",
        "padding-top: Fluent2SizeSettings.button-vertical-padding",
        "padding-bottom: Fluent2SizeSettings.button-vertical-padding",
        "spacing: Fluent2SizeSettings.button-content-spacing",
    ] {
        assert!(button.contains(expected), "fluent2 Button should use {expected}");
    }

    for copied_literal in [
        "Fluent2Palette.accent-foreground",
        "Fluent2Palette.control-foreground",
        "Fluent2Palette.accent-disabled",
        "Fluent2Palette.control-disabled",
        "Fluent2Palette.text-accent-foreground-disabled",
        "Fluent2Palette.text-disabled",
        "Fluent2Palette.tertiary-accent-background",
        "Fluent2Palette.control-tertiary",
        "Fluent2Palette.text-accent-foreground-secondary",
        "Fluent2Palette.text-secondary",
        "Fluent2Palette.secondary-accent-background",
        "Fluent2Palette.control-secondary",
        "Fluent2Palette.accent-background",
        "Fluent2Palette.accent-control-border",
        "Fluent2Palette.control-background",
        "Fluent2Palette.control-fill-transparent",
        "min-width: max(Fluent2SizeSettings.control-min-width, i-layout.min-width)",
        "min-height: max(Fluent2SizeSettings.control-height, i-layout.min-height)",
        "border-radius: Fluent2SizeSettings.button-radius",
        "padding-left: Fluent2SizeSettings.control-horizontal-padding",
        "padding-right: Fluent2SizeSettings.control-horizontal-padding",
        "padding-top: Fluent2SizeSettings.control-vertical-padding",
        "padding-bottom: Fluent2SizeSettings.control-vertical-padding",
        "spacing: Fluent2SizeSettings.overlay-padding",
    ] {
        assert!(
            !button.contains(copied_literal),
            "fluent2 Button should not bind active colors directly to copied generic token {copied_literal}"
        );
    }
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
fn test_fluent2_tabbar_base_uses_focus_touch_area() {
    let helper = load_file(&std::path::PathBuf::from("builtin:/fluent2/internal-components.slint"))
        .expect("fluent2 should embed internal-components.slint");
    let helper_contents = helper.read();
    let helper = std::str::from_utf8(&helper_contents).unwrap();

    let focus_touch_area = helper
        .split("export component FocusTouchArea")
        .nth(1)
        .and_then(|after| after.split("export component IconButton").next())
        .expect("fluent2 should define FocusTouchArea");

    assert!(
        focus_touch_area.contains("callback scroll-event <=> touch-area.scroll-event"),
        "fluent2 FocusTouchArea should forward scroll events for tab bars"
    );

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/tabwidget-base.slint"))
        .expect("fluent2 should own tabwidget-base.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "import { FocusTouchArea } from \"internal-components.slint\";",
        "export component TabBarBase",
        "touch-area := FocusTouchArea",
        "enabled: true",
        "scroll-event(event) =>",
        "@children",
        "Fluent2SizeSettings.tab-scroll-delta",
    ] {
        assert!(source.contains(expected), "fluent2 TabBarBase should use {expected}");
    }

    assert!(
        !source.contains("export component TabBarBase inherits TouchArea"),
        "fluent2 TabBarBase should not inherit directly from TouchArea"
    );
}

#[test]
fn test_fluent2_combo_and_spin_scroll_delta_use_component_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "out property <length> combobox-scroll-delta: control-scroll-delta;",
        "out property <length> spinbox-scroll-delta: control-scroll-delta;",
        "out property <length> tab-scroll-delta: control-scroll-delta;",
    ] {
        assert!(
            styling.lines().any(|line| line.trim() == expected),
            "fluent2 scroll thresholds should expose component-specific aliases: {expected}"
        );
    }

    for (file, expected_token) in [
        ("combobox-base.slint", "Fluent2SizeSettings.combobox-scroll-delta"),
        ("spinbox-base.slint", "Fluent2SizeSettings.spinbox-scroll-delta"),
    ] {
        let source = load_file(&std::path::PathBuf::from(format!("builtin:/fluent2/{file}")))
            .unwrap_or_else(|| panic!("fluent2 should embed {file}"));
        let source_contents = source.read();
        let source = std::str::from_utf8(&source_contents).unwrap();

        assert!(
            source.contains(expected_token),
            "fluent2 {file} should use component-specific scroll threshold token {expected_token}"
        );
        assert!(
            !source.contains("Fluent2SizeSettings.control-scroll-delta"),
            "fluent2 {file} should not bind directly to the broad control scroll threshold token"
        );
    }
}

#[test]
fn test_fluent2_owns_lineedit_base_geometry() {
    let base = load_file(&std::path::PathBuf::from("builtin:/fluent2/lineedit-base.slint"))
        .expect("fluent2 should own lineedit-base.slint");
    assert!(base.is_builtin(), "fluent2 lineedit-base.slint should be embedded");

    let base_contents = base.read();
    let base = std::str::from_utf8(&base_contents).unwrap();
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();
    assert!(
        styling.lines().any(|line| line.trim()
            == "out property <length> lineedit-min-focusable-width: input-min-focusable-width;"),
        "fluent2 styling should expose a line edit semantic focusable-width token"
    );

    assert!(
        base.contains("Fluent2SizeSettings.lineedit-min-focusable-width"),
        "fluent2 line edit base should use the semantic minimum focusable width token"
    );
    assert!(
        !base.contains("Fluent2SizeSettings.input-min-focusable-width"),
        "fluent2 line edit base should not bind directly to the broad input focusable-width token"
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
fn test_fluent2_slider_pressed_state_tracks_pointer_state() {
    let base = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider-base.slint"))
        .expect("fluent2 should own slider-base.slint");
    let base_contents = base.read();
    let base = std::str::from_utf8(&base_contents).unwrap();

    assert!(
        base.contains("out property <bool> pressed <=> touch-area.pressed"),
        "fluent2 slider pressed state should track actual pointer press state"
    );
    assert!(
        !base.contains("out property <bool> pressed <=> touch-area.enabled"),
        "fluent2 slider pressed state should not be true merely because the slider is enabled"
    );
}

#[test]
fn test_fluent2_slider_uses_focus_border() {
    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    assert!(
        source.contains("import { FocusBorder } from \"components.slint\""),
        "fluent2 Slider should import the shared Fluent2 focus border"
    );
    assert!(
        source.contains("if root.has-focus && root.enabled : FocusBorder"),
        "fluent2 Slider should render the shared focus border when keyboard-focused"
    );
    assert!(
        source.contains("border-radius: Fluent2SizeSettings.slider-thumb-radius"),
        "fluent2 Slider focus border should follow the thumb radius token"
    );
}

#[test]
fn test_fluent2_slider_track_geometry_uses_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "slider-track-radius",
        "slider-track-active-radius",
        "slider-thumb-inner-size",
        "slider-thumb-inner-radius",
        "slider-cross-axis-min-size",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "border-radius: Fluent2SizeSettings.slider-track-radius",
        "border-radius: Fluent2SizeSettings.slider-track-active-radius",
        "width: Fluent2SizeSettings.slider-thumb-inner-size",
        "border-radius: Fluent2SizeSettings.slider-thumb-inner-radius",
        "min-width: base.vertical ? Fluent2SizeSettings.slider-thumb-size : Fluent2SizeSettings.slider-cross-axis-min-size",
        "min-height: base.vertical ? Fluent2SizeSettings.slider-cross-axis-min-size : Fluent2SizeSettings.slider-thumb-size",
    ] {
        assert!(source.contains(expected), "fluent2 Slider should use {expected}");
    }

    assert!(
        !source.contains("border-radius: rail.border-radius"),
        "fluent2 Slider active track radius should not depend on rail border radius"
    );
    assert!(
        !source.contains("border-radius: self.width / 2"),
        "fluent2 Slider thumb inner radius should not be derived from live element width"
    );
    assert!(
        !source.contains("width: Fluent2SizeSettings.icon-size"),
        "fluent2 Slider inner thumb width should not use the generic icon-size token"
    );
    assert!(
        !source.contains("min-width: base.vertical ? Fluent2SizeSettings.slider-thumb-size : 0px"),
        "fluent2 Slider horizontal cross-axis minimum should use a semantic token"
    );
    assert!(
        !source.contains("min-height: base.vertical ? 0px : Fluent2SizeSettings.slider-thumb-size"),
        "fluent2 Slider vertical cross-axis minimum should use a semantic token"
    );
}

#[test]
fn test_fluent2_slider_rail_and_thumb_stroke_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in ["slider-rail-background", "slider-thumb-stroke", "slider-thumb-border-width"] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for copied_bridge in ["slider-rail-background: border", "slider-thumb-stroke: border"] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 slider semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "thumb.border-color: Fluent2Palette.slider-thumb-stroke",
        "border-width: Fluent2SizeSettings.slider-thumb-border-width",
        "background: Fluent2Palette.slider-rail-background",
    ] {
        assert!(source.contains(expected), "fluent2 Slider should use {expected}");
    }

    assert!(
        !source.contains("Fluent2Palette.border"),
        "fluent2 Slider should not use the generic border token for slider-specific rail or thumb visuals"
    );
    assert!(
        !source.contains("border-width: Fluent2SizeSettings.stroke-width"),
        "fluent2 Slider should not bind thumb border width directly to the generic stroke-width token"
    );
}

#[test]
fn test_fluent2_slider_active_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in [
        "slider-track-active-background",
        "slider-thumb-active-background",
        "slider-thumb-hover-background",
        "slider-thumb-pressed-background",
        "slider-motion-duration",
    ] {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for copied_bridge in [
        "slider-track-active-background: accent-background",
        "slider-thumb-active-background: accent-background",
        "slider-thumb-hover-background: secondary-accent-background",
        "slider-thumb-pressed-background: tertiary-accent-background",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 slider semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "thumb-inner.background: Fluent2Palette.slider-thumb-pressed-background",
        "thumb-inner.background: Fluent2Palette.slider-thumb-hover-background",
        "background: Fluent2Palette.slider-track-active-background",
        "background: Fluent2Palette.slider-thumb-active-background",
        "animate background, width { duration: Fluent2SizeSettings.slider-motion-duration",
    ] {
        assert!(source.contains(expected), "fluent2 Slider should use {expected}");
    }

    for copied_literal in [
        "track.background: Fluent2Palette.accent-background",
        "thumb-inner.background: Fluent2Palette.tertiary-accent-background",
        "thumb-inner.background: Fluent2Palette.secondary-accent-background",
        "background: Fluent2Palette.accent-background",
        "duration: Fluent2SizeSettings.control-motion-duration",
    ] {
        assert!(
            !source.contains(copied_literal),
            "fluent2 Slider should not bind active slider colors directly to copied generic token {copied_literal}"
        );
    }
}

#[test]
fn test_fluent2_slider_disabled_and_thumb_rest_colors_use_semantic_tokens() {
    let styling = load_file(&std::path::PathBuf::from("builtin:/fluent2/styling.slint"))
        .expect("fluent2 should embed styling.slint");
    let styling_contents = styling.read();
    let styling = std::str::from_utf8(&styling_contents).unwrap();

    for expected in ["slider-disabled-background", "slider-thumb-background", "slider-thumb-border"]
    {
        assert!(styling.contains(expected), "fluent2 styling should expose {expected}");
    }

    for copied_bridge in [
        "slider-disabled-background: accent-disabled",
        "slider-thumb-background: control-solid",
        "slider-thumb-border: circle-border",
    ] {
        let copied_bridge_line = format!("out property <brush> {copied_bridge};");
        assert!(
            !styling.lines().any(|line| line.trim() == copied_bridge_line),
            "fluent2 slider semantic tokens should not alias through copied bridge {copied_bridge}"
        );
    }

    let source = load_file(&std::path::PathBuf::from("builtin:/fluent2/slider.slint"))
        .expect("fluent2 should embed slider.slint");
    let source_contents = source.read();
    let source = std::str::from_utf8(&source_contents).unwrap();

    for expected in [
        "track.background: Fluent2Palette.slider-disabled-background",
        "rail.background: Fluent2Palette.slider-disabled-background",
        "thumb-inner.background: Fluent2Palette.slider-disabled-background",
        "background: Fluent2Palette.slider-thumb-background",
        "border-color: Fluent2Palette.slider-thumb-border",
    ] {
        assert!(source.contains(expected), "fluent2 Slider should use {expected}");
    }

    for copied_literal in [
        "track.background: Fluent2Palette.accent-disabled",
        "rail.background: Fluent2Palette.accent-disabled",
        "thumb-inner.background: Fluent2Palette.accent-disabled",
        "background: Fluent2Palette.control-solid",
        "border-color: Fluent2Palette.circle-border",
    ] {
        assert!(
            !source.contains(copied_literal),
            "fluent2 Slider should not bind disabled/resting thumb colors directly to copied generic token {copied_literal}"
        );
    }
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
