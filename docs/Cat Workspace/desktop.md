# desktop

- desktop.ts によって管理される
- 一つのハンドルに付き 1 ウィンドウとする
- GUI 関連の操作をする

## プロトコル

```plain
--> {"event": "SetName", "value": "TCP Necko"}
<-- {"status": "Success"}
--> {"event": "SetOptions", "options": "+StandardApp, -CloseButton"}
NOTE: options: [+|-][
                 StandardApp |
                 [Close, Max, Minimalize, App]Button |
                 TitleBar | ActionBar | Resizable |
               ]
```
