<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-group-scroll-dark.png" imageWidth="720" imageHeight="420">
```slint
import { Palette, Button, GroupBox, ScrollView, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 720px;
    height: 420px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 18px;

        HorizontalBox {
            spacing: 24px;

            GroupBox {
                title: "Enabled group";
                VerticalBox {
                    spacing: 10px;
                    Button { text: "Primary action"; primary: true; }
                    Button { text: "Secondary action"; }
                }
            }

            GroupBox {
                title: "Disabled group";
                enabled: false;
                VerticalBox {
                    spacing: 10px;
                    Button { text: "Disabled primary"; primary: true; enabled: false; }
                    Button { text: "Disabled neutral"; enabled: false; }
                }
            }
        }

        ScrollView {
            height: 220px;
            viewport-width: 960px;
            viewport-height: 420px;

            Rectangle {
                width: 960px;
                height: 420px;
                background: Palette.alternate-background;

                VerticalBox {
                    padding: 16px;
                    spacing: 10px;

                    for row in 14 : HorizontalBox {
                        spacing: 10px;
                        Button { text: "Scrollable action " + row; }
                        Rectangle { width: 560px; height: 32px; background: Palette.background; }
                    }
                }
            }
        }
    }
}
```
</CodeSnippetMD>
