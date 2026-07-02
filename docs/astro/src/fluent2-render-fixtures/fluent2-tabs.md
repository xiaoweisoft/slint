<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-tabs.png" imageWidth="760" imageHeight="320">
```slint
import { Palette, Button, HorizontalBox, TabBarHorizontalImpl, TabBarVerticalImpl, TabImpl, VerticalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 320px;
    background: Palette.background;

    HorizontalBox {
        padding: 24px;
        spacing: 24px;

        TabBarVerticalImpl {
            width: 168px;
            current: 0;
            current-focused: 0;
            num-tabs: 3;
            TabImpl { title: "Overview"; current: 0; current-focused: 0; tab-index: 0; num-tabs: 3; }
            TabImpl { title: "Details"; current: 0; current-focused: 0; tab-index: 1; num-tabs: 3; }
            TabImpl { title: "Advanced"; current: 0; current-focused: 0; tab-index: 2; num-tabs: 3; enabled: false; }
        }

        VerticalBox {
            spacing: 18px;
            alignment: start;

            TabBarHorizontalImpl {
                current: 0;
                current-focused: 0;
                num-tabs: 3;
                TabImpl { title: "History"; current: 0; current-focused: 0; tab-index: 0; num-tabs: 3; }
                TabImpl { title: "Metrics"; current: 0; current-focused: 0; tab-index: 1; num-tabs: 3; }
                TabImpl { title: "Disabled"; current: 0; current-focused: 0; tab-index: 2; num-tabs: 3; enabled: false; }
            }

            Rectangle {
                min-width: 440px;
                min-height: 180px;
                background: Palette.alternate-background;

                VerticalBox {
                    padding: 16px;
                    spacing: 12px;
                    Text { text: "Tab surface"; color: Palette.foreground; }
                    Button { text: "Contained action"; }
                }
            }
        }
    }
}
```
</CodeSnippetMD>
