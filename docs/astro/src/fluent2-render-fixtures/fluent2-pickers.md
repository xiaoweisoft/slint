<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-pickers.png" imageWidth="760" imageHeight="520">
```slint
import { Palette, DatePickerPopup, TimePickerPopup, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 520px;
    background: Palette.background;

    init => {
        date-picker.show();
        time-picker.show();
    }

    HorizontalBox {
        padding: 24px;
        spacing: 24px;

        date-picker := DatePickerPopup {
            x: 24px;
            y: 24px;
            title: "Pick a date";
            date: { year: 2026, month: 7, day: 2 };
        }

        time-picker := TimePickerPopup {
            x: 384px;
            y: 24px;
            title: "Pick a time";
            time: { hour: 10, minute: 30, second: 0 };
            use-24-hour-format: true;
        }
    }
}
```
</CodeSnippetMD>
