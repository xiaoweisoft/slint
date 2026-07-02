<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-time-picker.png" imageWidth="360" imageHeight="500">
```slint
import { Palette, TimePickerPopup } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 360px;
    height: 500px;
    background: Palette.background;

    init => {
        time-picker.show();
    }

    time-picker := TimePickerPopup {
        x: 24px;
        y: 24px;
        title: "Pick a time";
        time: { hour: 10, minute: 30, second: 0 };
        use-24-hour-format: true;
    }
}
```
</CodeSnippetMD>
