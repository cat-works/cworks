# Necko APP

- コードを本体側でオンオフ出来るように
- コードのフィルタリング
- コードのフォルダー化
- コード同士の依存関係
- コードフォルダーまるごと適応
- Memory Viewer

## [ProductServer AppData] syoch NeckoCode

- / [Dict]
- manifest.json [Dict]
  - type: String = “cafecode” | “minecode2”
- params.json [Dict]
  - [key]
    - display_name: String
    - type: String = `/[u|s][8|16|32]|String|Select/`
    - default: String
    - choices?: String[]
