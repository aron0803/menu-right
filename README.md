# Windows 右鍵選單工具 (複製路徑 + CMD/PowerShell/Claude 啟動器)

這是一個極簡、免安裝、免管理員權限的 Windows 右鍵選單工具。
完全使用 Rust 語言呼叫 Win32 FFI 實作，具備圖形化介面（GUI），支援動態啟用或關閉各項功能。

## 功能特點
* **複製檔案路徑**：右鍵點選「複製檔案路徑」，將路徑以標準反斜線 `\` 格式複製，不帶雙引號。
* **在此處開啟 CMD**：在檔案、資料夾或資料夾空白處右鍵開啟 CMD。若點擊檔案，會自動在該檔案所在的父目錄開啟。
* **在此處開啟 PowerShell**：在檔案、資料夾或資料夾空白處右鍵開啟 PowerShell。若點擊檔案，會自動在該檔案所在的父目錄開啟。
* **在此處開啟 Claude (YOLO)**：在檔案、資料夾或資料夾空白處右鍵開啟 CMD，並自動執行 `claude --dangerously-skip-permissions` 指令啟動 Claude 的 YOLO 模式。
* **支援中文與 Unicode**：中文字元與特殊路徑複製不會亂碼。
* **免管理員權限**：寫入使用者註冊表 (HKCU)，一般使用者帳號即可順利完成安裝與移除。
* **自帶一鍵開關 GUI**：雙擊 `CopyPathTool.exe` 即可開啟設定介面，動態設定要開啟的功能。開啟功能後，程式會自動複製自己到 `%ProgramData%\CopyPathTool\` 下儲存。

## 使用方式
1. 執行 `build.ps1` 進行編譯。
2. 雙擊執行產出的 `build\CopyPathTool.exe` 啟動設定精靈。
3. 勾選想要開啟的功能，點選「套用設定」即可。
4. 若欲卸載，將所有功能取消勾選，並點選「套用設定」即可徹底清除。

## 部署至其他電腦方式
如果您想要將此工具部署至其他電腦，您只需要拷貝編譯好的單一檔案 `CopyPathTool.exe`，並選擇以下任一種方式安裝：

### 方式一：手動介面安裝
* 直接在目標電腦上雙擊 `CopyPathTool.exe`，勾選所需要的功能，並點擊「**套用設定**」即可完成安裝。

### 方式二：命令列靜默安裝 (適合大量派送/指令稿)
* **一鍵背景靜默安裝 (預設開啟全部選單功能)**：
  ```cmd
  CopyPathTool.exe --install
  ```
* **一鍵背景靜默移除**：
  ```cmd
  CopyPathTool.exe --uninstall
  ```
*(執行靜默指令時，程式會自動在背景將自己複製至該電腦的 `%ProgramData%\CopyPathTool\CopyPathTool.exe` 並完成 HKCU 註冊表登錄，無需管理員權限且完全不彈出 any 視窗，極度適合 AD GPO、SCCM 或 `.bat` 指令稿大量部署！)*
