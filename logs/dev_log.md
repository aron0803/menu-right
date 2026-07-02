# 開發日誌 (Development Log)

## [2026-07-02 17:06] 修正關鍵 WM_COMMAND 常數定義錯誤 (0x0011 -> 0x0111)
- **異動檔案**：更新 `src/main.rs`, `logs/dev_log.md`
- **異動原因**：排查發現 `WM_COMMAND` 在 Rust 程式中被錯誤定義為 `0x0011`，而 Win32 標準之 `WM_COMMAND` 應為 `0x0111`。此定義錯誤導致整個視窗程序中所有的按鈕點擊、複選框切換等交互事件完全無法被匹配與執行。修正為 `0x0111` 並將版本升級至 v1.1.2，成功修復套用按鈕無法運作的根本 Bug。

## [2026-07-02 17:02] 新增版本控制與各項點擊事件除錯日誌
- **異動檔案**：更新 `src/main.rs`, `logs/dev_log.md`
- **異動原因**：應使用者要求，將工具程式升級至 v1.1.1。在啟動時記錄目前版本號，並在 GUI 視窗標題列顯示版本。此外，特別在 `WM_COMMAND` 事件中新增了所有複選框（Checkbox）點擊事件以及套用按鈕（Apply button）點擊的詳細日誌，以利排查按鈕點擊通訊狀態。

## [2026-07-02 16:58] 修復 64 位元 Windows 按鈕點擊無效問題
- **異動檔案**：更新 `src/main.rs`, `logs/dev_log.md`
- **異動原因**：修復 `wnd_proc` 中使用 `wparam as i32` 取得控制項 ID，在 64 位元 Windows 系統下因 wparam 內高位元組包含額外通知碼或垃圾資料，導致 `id == ID_BTN_APPLY` 判斷不成立、「套用設定」按鈕點擊完全失效的 Bug。改用 `(wparam & 0xFFFF) as i32` 進行精確低 16 位元遮罩擷取，並新增 `WM_COMMAND` 的 ID 與 Code 除錯日誌紀錄。

## [2026-07-02 16:54] 新增調試日誌輸出功能
- **異動檔案**：更新 `src/main.rs`, `logs/dev_log.md`
- **異動原因**：在執行檔同目錄下輸出 `copypathtool_debug.log`，紀錄程式啟動參數、登錄檔讀寫過程與 API 回傳碼，便於在其他電腦上調試排查狀態保存失敗之具體原因。

## [2026-07-02 16:50] 修復跨使用者權限與登錄檔寫入驗證機制
- **異動檔案**：更新 `src/main.rs`, `logs/dev_log.md`
- **異動原因**：修復當以系統管理員權限執行時，登錄檔被寫入至 Administrator 的 HKCU，導致以一般使用者重新開啟時無法讀取勾選狀態的 Bug。新增 `get_registry_root()` 動態判斷權限，當有權限時寫入/讀取 HKLM (機器範圍)，無權限時回退至 HKCU (使用者範圍)；並於 `check_key_exists` 中提供 HKLM/HKCU 雙重回退讀取。另外將 `set_registry_string` 改為傳回 `bool` 並在 `install_and_register` 進行強驗證，避免寫入失敗時仍顯示成功。

## [2026-07-02 16:45] 發布工具至 GitHub Tools 倉庫並更新下載網頁
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：將編譯完的 `CopyPathTool.exe` 打包為 `CopyPathTool.zip`，複製至本地克隆的 `Tools` 倉庫，更新 `tools.json` 及 `README.md` 說明，並透過執行 `publish.ps1` 將其推送發布至 GitHub Releases 與 GitHub Pages，完成網頁下載頁面更新。

## [2026-07-02 16:35] 完成編譯與靜默部署
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：執行 `build.ps1` 完成最新版本編譯，並執行 `CopyPathTool.exe --install` 進行靜默部署，完成包含「在此處開啟 Claude (YOLO)」在內的所有右鍵選單註冊。

## [2026-07-02 16:21] 新增「在此處開啟 Claude (YOLO)」功能
- **異動檔案**：更新 `src/main.rs`, `README.md`, `CHANGELOG.md`, `logs/dev_log.md`
- **異動原因**：新增右鍵選單項目與 GUI 選項，支援在選定目錄下快速以 YOLO 模式開啟 Claude 互動式命令行工具（`claude --dangerously-skip-permissions`），提升開發自動化與 AI 輔助效率。

## [2026-06-30 20:36] 支援部署功能與更新說明文件
- **異動檔案**：更新 `src/main.rs`, `README.md`, `logs/dev_log.md`
- **異動原因**：實作 `--install` 與 `--uninstall` 靜默部署參數，允許在不需要顯示 GUI 視窗的情況下一鍵完成所有功能註冊與清除，方便大量派送與自動化指令部署。

## [2026-06-30 20:34] 修復登錄設定並新增 CMD/PowerShell 啟動器
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：使用 Python `winreg` 修復並完整註冊包含「複製檔案路徑」、「在此開啟 CMD」和「在此開啟 PowerShell」的右鍵選單項目，設定目標為 `%ProgramData%\CopyPathTool\CopyPathTool.exe`，解決登錄鍵缺失及舊版 PowerShell 轉譯錯誤的問題。

## [2026-06-30 20:23] 完成編譯與手動驗證
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：安裝 Rust GNU 工具鏈，完成 `build.ps1` 執行與 Rust EXE 編譯，並成功通過剪貼簿與進程啟動測試，專案功能全數開發完成。

## [2026-06-30 20:22] 建立打包與編譯指令稿與標準專案檔案
- **異動檔案**：建立 `build.ps1`, `plan.md`, `README.md`, `readme.html`, `CHANGELOG.md`
- **異動原因**：撰寫一鍵編譯與打包腳本，並補齊標準專案結構所需的所有說明文件。

## [2026-06-30 20:21] 建立 Cargo.toml 專案設定檔
- **異動檔案**：建立 `Cargo.toml`
- **異動原因**：定義 Rust 專案基本設定並設定 release 大小與 LTO 最佳化。

## [2026-06-30 20:20] 清理舊有批次檔與 PowerShell 腳本
- **異動檔案**：刪除 `install.bat`, `uninstall.bat`, `install.ps1`, `uninstall.ps1`
- **異動原因**：切換回 Rust 開發，移除純註冊表方案的所有腳本檔案以保持專案乾淨。

## [2026-06-30 20:19] 再次切換為 Rust 開發並開始安裝 Rust 環境
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：純註冊表方式在部分路徑與環境下仍存在解析與亂碼問題，使用者要求重回 Rust 解決方案，重新開始下載安裝 Rust 開發工具鏈。

## [2026-06-30 20:13] 建立右鍵選單移除批次檔
- **異動檔案**：建立 `uninstall.bat`
- **異動原因**：實作一鍵移除選單功能，方便使用者安全清理註冊表殘留資訊。

## [2026-06-30 20:12] 建立右鍵選單安裝批次檔
- **異動檔案**：建立 `install.bat`
- **異動原因**：實作一鍵安裝選單功能，透過 CMD 單行指令直接複製路徑到剪貼簿，且附帶標準複製圖示。

## [2026-06-30 20:11] 清理 Rust 暫存檔案並變更為純註冊表開發
- **異動檔案**：刪除 `Cargo.toml`, `test.bat`, `rustup-init.exe`
- **異動原因**：使用者選擇純註冊表 (Pure Registry) 方式實作「複製檔案路徑」，取消安裝 Rust 環境並清理相關臨時檔案。

## [2026-06-30 20:10] 清理舊有 Python 檔案與依賴
- **異動檔案**：刪除 `src/utils/` 目錄，清空 `requirements.txt`
- **異動原因**：切換為 Rust 開發，清理舊專案留下的 Python 相關檔案以保持工作區乾淨。

## [2026-06-30 20:09] 變更專案方向為 Rust 開發並開始安裝 Rust 環境
- **異動檔案**：更新 `logs/dev_log.md`
- **異動原因**：回應使用者變更，將改用 Rust 實作無依賴右鍵複製工具，並準備下載安裝 Rust 軟體包管理器 (rustup)。

## [2026-06-30 20:08] 建立剪貼簿核心工具
- **異動檔案**：建立 `src/utils/clipboard.py`
- **異動原因**：使用 Windows ctypes API 呼叫系統剪貼簿 API，實作免第三方庫依賴、支援 Unicode 的路徑複製工具。

## [2026-06-30 20:07] 建立專案套件依賴檔案
- **異動檔案**：建立 `requirements.txt`
- **異動原因**：定義設定 GUI 使用的 `ttkbootstrap` 庫及打包工具 `pyinstaller` 的依賴。

## [2026-06-30 20:06] 初始化專案結構與開發日誌
- **異動檔案**：建立 `logs/dev_log.md` 以及專案核心結構目錄 (`src/`, `logs/`, `assets/`, `docs/`, `build/`)
- **異動原因**：依照全域規則初始化專案，建立開發日誌以記錄後續所有的開發與異動歷史。
