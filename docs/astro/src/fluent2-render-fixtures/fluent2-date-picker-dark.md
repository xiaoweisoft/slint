<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-date-picker-dark.png" imageWidth="360" imageHeight="500">
```slint
import { Palette, DatePickerPopup } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 360px;
    height: 500px;
    background: Palette.background;

    init => {
        date-picker.show();
    }

    date-picker := DatePickerPopup {
        x: 24px;
        y: 24px;
        title: "Pick a date";
        date: { year: 2026, month: 7, day: 2 };
    }
}
```
</CodeSnippetMD>
