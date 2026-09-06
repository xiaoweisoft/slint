// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use slint::ComponentHandle;

slint::slint! {
    #[style="fluent2-light"]
    import { Button, CheckBox, Switch } from "std-widgets.slint";

    export component Fluent2PointerHarness inherits Window {
        width: 360px;
        height: 240px;
        in property <bool> controls-enabled: true;
        in property <length> button-width: 180px;
        out property <int> clicks;
        out property <bool> checked <=> check.checked;
        out property <bool> switched <=> toggle.checked;
        VerticalLayout {
            button := Button {
                width: root.button-width;
                height: 48px;
                text: "Native button";
                enabled: root.controls-enabled;
                clicked => { root.clicks += 1; }
            }
            check := CheckBox { text: "Native checkbox"; enabled: root.controls-enabled; }
            toggle := Switch { text: "Native switch"; enabled: root.controls-enabled; }
        }
    }
}

#[test]
fn native_controls_keep_their_rendered_pointer_bounds() {
    use i_slint_backend_testing::ElementHandle;
    use slint::platform::{PointerEventButton, WindowEvent};
    i_slint_backend_testing::init_no_event_loop();
    let instance = Fluent2PointerHarness::new().unwrap();
    for (label, enabled) in [
        ("Native button", true),
        ("Native checkbox", true),
        ("Native switch", true),
        ("Native button", false),
        ("Native checkbox", false),
        ("Native switch", false),
    ] {
        instance.set_controls_enabled(enabled);
        let control = ElementHandle::find_by_accessible_label(&instance, label).next().unwrap();
        let origin = control.absolute_position();
        let size = control.size();
        assert!(size.width > 0.0 && size.height > 0.0);
        let position =
            slint::LogicalPosition::new(origin.x + size.width / 2.0, origin.y + size.height / 2.0);
        instance.window().dispatch_event(WindowEvent::PointerMoved { position });
        instance.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        instance.window().dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        match label {
            "Native button" => assert_eq!(instance.get_clicks(), 1),
            "Native checkbox" => assert!(instance.get_checked()),
            "Native switch" => assert!(instance.get_switched()),
            _ => unreachable!(),
        }
    }
    // Explicit consumer geometry and later resizing override the fill default.
    instance.set_controls_enabled(true);
    instance.set_button_width(240.0);
    let button =
        ElementHandle::find_by_accessible_label(&instance, "Native button").next().unwrap();
    assert_eq!(button.size().width, 240.0);
    assert_eq!(button.size().height, 48.0);
    let origin = button.absolute_position();
    let position = slint::LogicalPosition::new(origin.x + 220.0, origin.y + 24.0);
    instance.window().dispatch_event(WindowEvent::PointerMoved { position });
    instance
        .window()
        .dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
    instance.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    assert_eq!(instance.get_clicks(), 2);
}

slint::slint! {
    #[style="fluent2-light"]
    import { ComboBox } from "std-widgets.slint";

    export component Fluent2ComboBoxHarness inherits Window {
        width: 200px;
        height: 200px;

        VerticalLayout {
            alignment: center;

            box := ComboBox {
                model: ["Aaa", "Bbb", "Ccc"];
            }
        }

        out property <string> current-value <=> box.current-value;
    }
}

#[test]
fn combobox_icon_is_vertically_centered() {
    i_slint_backend_testing::init_no_event_loop();
    let instance = Fluent2ComboBoxHarness::new().unwrap();
    let combo = i_slint_backend_testing::ElementHandle::find_by_element_id(
        &instance,
        "Fluent2ComboBoxHarness::box",
    )
    .next()
    .unwrap();
    let icon =
        i_slint_backend_testing::ElementHandle::find_by_element_id(&instance, "ComboBox::icon")
            .next()
            .unwrap();

    let combo_center = combo.absolute_position().y + combo.size().height / 2.0;
    let icon_center = icon.absolute_position().y + icon.size().height / 2.0;
    assert_eq!(icon_center, combo_center);
}

#[test]
fn combobox_click_opens_popup() {
    i_slint_backend_testing::init_no_event_loop();
    let instance = Fluent2ComboBoxHarness::new().unwrap();
    let combo = i_slint_backend_testing::ElementHandle::find_by_element_id(
        &instance,
        "Fluent2ComboBoxHarness::box",
    )
    .next()
    .unwrap();

    assert_eq!(combo.accessible_expanded(), Some(false));
    let position = slint::LogicalPosition::new(100.0, 100.0);
    instance.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position,
        button: slint::platform::PointerEventButton::Left,
    });
    instance.window().dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position,
        button: slint::platform::PointerEventButton::Left,
    });

    assert_eq!(combo.accessible_expanded(), Some(true));
    assert_eq!(
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(&instance, "Aaa").count(),
        1
    );

    let second_item =
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(&instance, "Bbb")
            .next()
            .unwrap();
    let item_position = second_item.absolute_position();
    let item_size = second_item.size();
    let combo_position = combo.absolute_position();
    let position = slint::LogicalPosition::new(
        combo_position.x + item_position.x + item_size.width / 2.0,
        combo_position.y - 4.0 + item_position.y + item_size.height / 2.0,
    );
    instance.window().dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position,
        button: slint::platform::PointerEventButton::Left,
    });
    instance.window().dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position,
        button: slint::platform::PointerEventButton::Left,
    });

    assert_eq!(instance.get_current_value(), "Bbb");
    assert_eq!(combo.accessible_expanded(), Some(false));
}
