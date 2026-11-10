# YuiClock

デスクトップに置いておけるWindows向けのデジタル時計です。常に最前面に表示でき、文字色、背景色とそれぞれの透明度、フォント、日付や秒の表示、曜日の表記（日／日曜日／Sunday／Sun）、角の丸さを変えられます。

## 使い方

- 時計はドラッグで動かせます。位置と大きさは次に起動したときも保たれます
- 時計の右クリックか、タスクトレイのアイコンからメニューを開けます。「最前面に表示」「設定…」「終了…」があります
- タスクトレイのアイコンを左クリックすると、時計を前面に戻します。画面の外に出ていたら中央に戻します

## OBSに表示する

設定の「OBSにドラッグして追加」を、OBSの「ソース」欄かプレビューにドラッグしてください。確認で「はい」を選ぶと、YuiClockという名前のブラウザソースが追加されます。色や表示の設定は時計と同じで、設定を変えるとOBSにもすぐ反映されます。

OBS用のページは`%APPDATA%\com.suzunayui.yuiclock\obs`にファイルとして書き出され、OBSはそれをローカルファイルとして開きます。そのため、YuiClockを終了していてもOBSには最後の設定で表示されます。ドラッグできないときは、設定の「URLをコピー」でコピーしたURLをブラウザソースのURLに貼ってください。

## 設定の保存先

設定は`%APPDATA%\com.suzunayui.yuiclock\settings.json`に保存されます。アンインストールしても残ります。

## インストール

[Releases](https://github.com/suzunayui/YuiClock/releases)の`YuiClock-Setup-<バージョン>.exe`を実行してください。管理者権限は不要です。[YuiLauncher](https://github.com/suzunayui/YuiLauncher)からもインストールと更新ができます。

署名なしのため、SmartScreenの警告が出ることがあります。その場合は「詳細情報」から「実行」を選んでください。

## 開発

Rustが必要です。Tauriで動きます。

```bash
npm install
npm start
```

```bash
npm test
```

## リリース

バージョン番号は「年.月日.その日の連番」です。例：`2026.1108.1`は2026年11月8日の1版目です。

```bash
npm version 2026.1108.1 --no-git-tag-version
```

```bash
npm run dist
```

`release`にインストーラーとチェックサムが作られます。この2つをGitHub Releaseに公開します。YuiLauncherはチェックサムを確かめてからインストールします。
