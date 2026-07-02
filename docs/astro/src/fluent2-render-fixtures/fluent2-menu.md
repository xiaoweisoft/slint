<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-menu.png" imageWidth="760" imageHeight="220">
```slint
import { Palette } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 220px;
    background: Palette.background;

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
```
</CodeSnippetMD>
