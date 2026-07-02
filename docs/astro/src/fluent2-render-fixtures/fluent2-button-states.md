<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-button-states.png" imageWidth="680" imageHeight="260">
```slint
import { Palette, Button, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 680px;
    height: 260px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 16px;

        HorizontalBox {
            spacing: 12px;
            Button { text: "Primary"; primary: true; }
            Button { text: "Neutral"; }
            Button { text: "Checked"; checkable: true; checked: true; }
        }

        HorizontalBox {
            spacing: 12px;
            Button { text: "Disabled primary"; primary: true; enabled: false; }
            Button { text: "Disabled neutral"; enabled: false; }
        }

        Rectangle {
            height: 86px;
            background: Palette.alternate-background;

            HorizontalBox {
                padding: 20px;
                spacing: 12px;
                Button { text: "Icon"; icon: @image-url("fluent2-button-icon.svg"); colorize-icon: true; }
                Button { icon: @image-url("fluent2-button-icon.svg"); colorize-icon: true; }
            }
        }
    }
}
```
</CodeSnippetMD>
