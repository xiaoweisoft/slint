<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-value-choice-controls-light.png" imageWidth="680" imageHeight="360">
```slint
import { Palette, CheckBox, ComboBox, Slider, SpinBox, Switch, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 680px;
    height: 360px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 18px;

        HorizontalBox {
            spacing: 28px;
            VerticalBox {
                spacing: 12px;
                CheckBox { text: "Checked"; checked: true; }
                CheckBox { text: "Unchecked"; }
                CheckBox { text: "Disabled"; checked: true; enabled: false; }
            }
            VerticalBox {
                spacing: 12px;
                Switch { text: "On"; checked: true; }
                Switch { text: "Off"; }
                Switch { text: "Disabled"; checked: true; enabled: false; }
            }
        }

        Rectangle {
            height: 128px;
            background: Palette.alternate-background;

            VerticalBox {
                padding: 18px;
                spacing: 14px;
                Slider { value: 18; minimum: 0; maximum: 100; }
                Slider { value: 82; minimum: 0; maximum: 100; }
                Slider { value: 50; minimum: 0; maximum: 100; enabled: false; }
            }
        }

        HorizontalBox {
            spacing: 16px;
            SpinBox { value: 24; minimum: 0; maximum: 100; }
            SpinBox { value: 64; minimum: 0; maximum: 100; enabled: false; }
            ComboBox { model: ["Alpha", "Beta", "Gamma"]; current-index: 1; }
            ComboBox { model: ["Alpha", "Beta", "Gamma"]; current-index: 2; enabled: false; }
        }
    }
}
```
</CodeSnippetMD>
