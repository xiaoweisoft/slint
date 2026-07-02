<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-states-light.png" imageWidth="760" imageHeight="640">
```slint
import { Palette,
    Button, CheckBox, ComboBox, GroupBox, LineEdit, ProgressIndicator, ScrollView,
    Slider, SpinBox, Spinner, Switch, TextEdit, VerticalBox, HorizontalBox
} from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 640px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 16px;

        GroupBox {
            title: "Enabled controls";
            VerticalBox {
                spacing: 10px;
                HorizontalBox {
                    spacing: 12px;
                    Button { text: "Primary"; primary: true; }
                    Button { text: "Neutral"; }
                    CheckBox { text: "Checked"; checked: true; }
                    Switch { text: "On"; checked: true; }
                }
                HorizontalBox {
                    spacing: 12px;
                    LineEdit { text: "Editable text"; }
                    SpinBox { value: 24; minimum: 0; maximum: 100; }
                    ComboBox { model: ["One", "Two", "Three"]; current-index: 1; }
                }
            }
        }

        GroupBox {
            title: "Disabled and read-only";
            VerticalBox {
                spacing: 10px;
                HorizontalBox {
                    spacing: 12px;
                    Button { text: "Disabled"; enabled: false; }
                    CheckBox { text: "Disabled"; checked: true; enabled: false; }
                    Switch { text: "Disabled"; checked: true; enabled: false; }
                    Slider { value: 55; enabled: false; }
                }
                HorizontalBox {
                    spacing: 12px;
                    LineEdit { text: "Read only"; read-only: true; }
                    LineEdit { text: "Disabled"; enabled: false; }
                    SpinBox { value: 42; enabled: false; }
                }
            }
        }

        TextEdit { text: "Disabled multiline text\nkeeps Fluent2 surface and foreground tokens"; enabled: false; height: 80px; }

        HorizontalBox {
            spacing: 16px;
            ProgressIndicator { progress: 0.72; }
            Spinner { indeterminate: true; }
        }

        ScrollView {
            height: 120px;
            viewport-width: 920px;
            viewport-height: 220px;
            Rectangle {
                width: 920px;
                height: 220px;
                background: Palette.alternate-background;
                VerticalBox {
                    padding: 12px;
                    spacing: 8px;
                    Text { text: "ScrollView viewport"; color: Palette.foreground; }
                    for row in 8 : Button { text: "Scrollable row " + row; }
                }
            }
        }
    }
}
```
</CodeSnippetMD>
