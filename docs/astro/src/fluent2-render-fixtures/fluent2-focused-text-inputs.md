<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-focused-text-inputs.png" imageWidth="760" imageHeight="420">
```slint
import { Palette, ComboBox, LineEdit, SpinBox, TextEdit, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 420px;
    background: Palette.background;

    init => { focused-line.focus(); }

    VerticalBox {
        padding: 24px;
        spacing: 16px;

        HorizontalBox {
            spacing: 12px;
            focused-line := LineEdit {
                width: 220px;
                text: "Focused line";
            }
            LineEdit { placeholder-text: "Placeholder"; }
            LineEdit { text: "Password"; input-type: InputType.password; }
        }

        TextEdit {
            width: 672px;
            height: 120px;
            text: "Focused multiline text";
        }

        HorizontalBox {
            spacing: 12px;
            LineEdit { text: "Read only"; read-only: true; }
            LineEdit { text: "Disabled"; enabled: false; }
        }

        HorizontalBox {
            spacing: 12px;
            focused-spin := SpinBox { value: 24; minimum: 0; maximum: 100; }
            SpinBox { value: 64; minimum: 0; maximum: 100; enabled: false; }
            focused-combo := ComboBox { model: ["Alpha", "Beta", "Gamma"]; current-index: 1; }
            ComboBox { model: ["Alpha", "Beta", "Gamma"]; current-index: 2; enabled: false; }
        }
    }
}
```
</CodeSnippetMD>
