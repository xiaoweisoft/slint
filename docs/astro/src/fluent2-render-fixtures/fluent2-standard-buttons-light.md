<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-standard-buttons-light.png" imageWidth="760" imageHeight="220">
```slint
import { Palette, StandardButton, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 220px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 12px;

        HorizontalBox {
            spacing: 10px;
            StandardButton { kind: ok; primary: true; }
            StandardButton { kind: cancel; }
            StandardButton { kind: apply; }
            StandardButton { kind: close; }
        }

        HorizontalBox {
            spacing: 10px;
            StandardButton { kind: reset; }
            StandardButton { kind: help; }
            StandardButton { kind: yes; primary: true; }
            StandardButton { kind: no; }
        }

        HorizontalBox {
            spacing: 10px;
            StandardButton { kind: abort; }
            StandardButton { kind: retry; }
            StandardButton { kind: ignore; enabled: false; }
        }
    }
}
```
</CodeSnippetMD>
