<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-menu-states-light.png" imageWidth="760" imageHeight="260">
```slint
import { Palette } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 260px;
    background: Palette.background;

    MenuBar {
        Menu { title: "View";
            MenuItem { title: "Checked item"; checkable: true; checked: true; }
            MenuItem { title: "Plain item"; }
            MenuSeparator { }
            MenuItem { title: "Disabled item"; enabled: false; }
            Menu { title: "Nested";
                MenuItem { title: "Nested item"; }
            }
        }
        Menu { title: "Tools";
            MenuItem { title: "Command"; }
        }
        Menu { title: "Disabled"; enabled: false;
            MenuItem { title: "Unavailable"; }
        }
    }
}
```
</CodeSnippetMD>
