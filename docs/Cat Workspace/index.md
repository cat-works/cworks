# Cat Workspace

[カーネル](./カーネル.md)
[desktop](./desktop.md)

## 概要

- マルチウィンドウ（１アプリnウィンドウ）で作業できるように

## ウィンドウなどの仕様

- 閉じる, 最大化 or 最小化, スナップ (MUST)
- スナップのアニメーションあると嬉しいかも (OPTIONAL)
- ボタンのデザインあると良いかも
- タイトルバー
- ウィンドウ全体の UI あるといいね

## UI

- ページで使ってるものを流用
- 実際に描画するのは typescript 側

## ファイルシステム

IndexedDB とかを駆使してやる

- /
- usr [Dict]
  - mime [Dict]
    - application [Dict]
    - text
      - cafecode
      - minecode2
  - app
    - sys
      - .folder.json
        - name: String = “System Apps”
        - icon: String = “Haguruma”
      - explorer.app
      - terminal.app
      - setting.app
    - notepad.app
    - [necko.app](./Necko%20app)
    - [analysis.app](./Analysis%20app.md)
- mnt
  - [wiiu (ftpfs)]
- workspace

### .app について

Folder として保存されるアプリバンドル

```plain
Dict {
  "src": Dict {
  "main.mc": File {"..."}
  "***.mc"
  }
}
```

- Root
- src
  - main.mc
  - **.mc
- icon.png
- manifest.json
  - name: アプリ名
  - manifest_Version
  - version: バージョン
  - updater: 更新 URL
  - type: “MineCode2” | “JavaScript”
