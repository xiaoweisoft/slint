<CodeSnippetMD imagePath="../assets/generated/fluent2-render-fixtures/fluent2-list-table.png" imageWidth="760" imageHeight="360">
```slint
import { Palette, StandardListView, StandardTableView, VerticalBox, HorizontalBox } from "std-widgets.slint";

export component ScreenShotThis inherits Window {
    width: 760px;
    height: 360px;
    background: Palette.background;

    HorizontalBox {
        padding: 24px;
        spacing: 20px;

        VerticalBox {
            spacing: 8px;
            StandardListView {
                width: 220px;
                height: 280px;
                model: [
                    { text: "Inbox" },
                    { text: "Documents" },
                    { text: "Archive" },
                    { text: "Settings" },
                ];
                current-item: 1;
            }
        }

        StandardTableView {
            width: 468px;
            height: 280px;
            columns: [
                { title: "Name", min-width: 140px, horizontal-stretch: 1, sort-order: SortOrder.ascending },
                { title: "State", min-width: 120px, horizontal-stretch: 1, sort-order: SortOrder.unsorted },
                { title: "Latency", min-width: 100px, horizontal-stretch: 0.5, sort-order: SortOrder.unsorted },
            ];
            rows: [
                [ { text: "Alpha" }, { text: "Ready" }, { text: "12 ms" } ],
                [ { text: "Beta" }, { text: "Paused" }, { text: "24 ms" } ],
                [ { text: "Gamma" }, { text: "Disabled" }, { text: "48 ms" } ],
                [ { text: "Delta" }, { text: "Ready" }, { text: "15 ms" } ],
            ];
        }
    }
}
```
</CodeSnippetMD>
