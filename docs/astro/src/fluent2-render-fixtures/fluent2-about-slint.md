<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-about-slint.png" imageWidth="420" imageHeight="180">
```slint
import { AboutSlint, Palette } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 420px;
    height: 180px;
    background: Palette.background;

    AboutSlint {
        x: 50px;
        y: 30px;
        width: 320px;
        height: 120px;
    }
}
```
</CodeSnippetMD>
