<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-pickers.png" imageWidth="760" imageHeight="520">
```slint
import { Palette, DatePickerPopup, TimePickerPopup, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 520px;
    background: Palette.background;
    property <bool> popups-open: { date-picker.show(); time-picker.show(); true }

    HorizontalBox {
        padding: 24px;
        spacing: 24px;

        date-picker := DatePickerPopup {
            title: "Pick a date";
            date: { year: 2026, month: 7, day: 2 };
        }

        time-picker := TimePickerPopup {
            title: "Pick a time";
            time: { hour: 10, minute: 30, second: 0 };
            use-24-hour-format: true;
        }
    }
}
```
</CodeSnippetMD>
