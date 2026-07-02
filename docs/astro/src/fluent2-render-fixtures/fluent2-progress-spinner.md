<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-progress-spinner.png" imageWidth="560" imageHeight="260">
```slint
import { Palette, ProgressIndicator, Spinner, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 560px;
    height: 260px;
    background: Palette.background;

    VerticalBox {
        padding: 24px;
        spacing: 18px;

        ProgressIndicator { progress: 0.38; }
        ProgressIndicator { progress: 0.74; }
        ProgressIndicator { indeterminate: true; }

        Rectangle {
            height: 96px;
            background: Palette.alternate-background;

            HorizontalBox {
                padding: 24px;
                spacing: 32px;

                Spinner { progress: 0.42; }
                Spinner { indeterminate: true; }
            }
        }
    }
}
```
</CodeSnippetMD>
