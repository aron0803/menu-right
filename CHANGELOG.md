# 更新日誌

## [1.2.0] - 2026-07-03
- 修復在其他電腦部署時，右鍵選單操作會不斷閃出黑色控制台視窗的問題。對所有背景執行的 `reg.exe` 與 `cmd.exe` 子程序加入 `CREATE_NO_WINDOW` flag，確保註冊表讀寫與解安裝自刪除操作完全靜默執行。

## [1.1.0] - 2026-07-02
- 新增「在此處開啟 Claude (YOLO)」右鍵選單功能，點擊後會自動在該目錄開啟 CMD 並執行 `claude --dangerously-skip-permissions`。
- 調整 GUI 視窗大小與按鈕版面，新增對應的啟用複選框。

## [1.0.0] - 2026-06-30
- 建立使用 Rust FFI 實作的 Windows 右鍵選單工具。
- 支援複製路徑、在此處開啟 CMD、在此處開啟 PowerShell 三個功能。
- 實作原生 Win32 Checkbox GUI 來管理各個選單功能的開關。
- 啟用功能後程式會自動複製自己到 `%ProgramData%\CopyPathTool\` 下儲存。
