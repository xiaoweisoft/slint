<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-controls-dark.png" imageWidth="760" imageHeight="640">
```slint
import { Palette,
    AboutSlint, Button, CheckBox, ComboBox, DatePickerPopup, GroupBox,
    LineEdit, ProgressIndicator,
    Slider, SpinBox, Spinner, StandardButton, StandardListView, StandardTableView,
    Switch, TabWidget, TextEdit, TimePickerPopup, VerticalBox, HorizontalBox
} from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 640px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 16px;

        HorizontalBox {
            spacing: 12px;
            Button { text: "Primary"; primary: true; }
            Button { text: "Neutral"; }
            CheckBox { text: "Selected"; checked: true; }
            Switch { text: "Enabled"; checked: true; }
        }

        HorizontalBox {
            spacing: 12px;
            LineEdit { text: "Line edit"; }
            SpinBox { value: 42; minimum: 0; maximum: 100; }
            ComboBox { model: ["Alpha", "Beta", "Gamma"]; current-index: 1; }
        }

        TextEdit { text: "Multiline text edit\nwith Fluent2 styling"; height: 88px; }

        HorizontalBox {
            spacing: 24px;
            Slider { value: 42; minimum: 0; maximum: 100; }
            ProgressIndicator { progress: 0.62; }
            Spinner { indeterminate: true; }
        }

        GroupBox {
            title: "Grouped controls";
            HorizontalBox {
                spacing: 8px;
                Button { text: "Apply"; }
                StandardButton { kind: ok; }
            }
        }

        HorizontalBox {
            spacing: 12px;
            DatePickerPopup { title: "Date"; }
            TimePickerPopup { title: "Time"; }
            AboutSlint { width: 280px; height: 80px; }
        }

        TabWidget {
            Tab { title: "List";
                StandardListView {
                    model: [ { text: "Navigation" }, { text: "Documents" }, { text: "Settings" } ];
                    current-item: 1;
                }
            }
            Tab { title: "Table";
                StandardTableView {
                    columns: [
                        { title: "Name", min-width: 110px, horizontal-stretch: 1, sort-order: SortOrder.unsorted },
                        { title: "State", min-width: 110px, horizontal-stretch: 1, sort-order: SortOrder.unsorted },
                    ];
                    rows: [
                        [ { text: "Alpha" }, { text: "Ready" } ],
                        [ { text: "Beta" }, { text: "Paused" } ],
                    ];
                }
            }
        }
    }
}
```
</CodeSnippetMD>
