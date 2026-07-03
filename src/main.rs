#![windows_subsystem = "windows"]

const VERSION: &str = "1.3.1";

use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::fs;
use std::sync::OnceLock;

// --- Win32 FFI Declarations ---

type HWND = *mut std::ffi::c_void;
type HGLOBAL = *mut std::ffi::c_void;

#[repr(C)]
struct WNDCLASSW {
    style: u32,
    lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: *mut std::ffi::c_void,
    hIcon: *mut std::ffi::c_void,
    hCursor: *mut std::ffi::c_void,
    hbrBackground: *mut std::ffi::c_void,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    fn UpdateWindow(hWnd: HWND) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn PostMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn EnableWindow(hWnd: HWND, bEnable: i32) -> i32;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut std::ffi::c_void;
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    fn OpenClipboard(hWndNewOwner: HWND) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(uFormat: u32, hMem: HGLOBAL) -> HGLOBAL;
    fn CloseClipboard() -> i32;
    fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
}

#[repr(C)]
struct SYSTEMTIME {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> *mut std::ffi::c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> i32;
    fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> *mut std::ffi::c_void;
    fn ReleaseMutex(hMutex: *mut std::ffi::c_void) -> i32;
    fn WaitForSingleObject(hHandle: *mut std::ffi::c_void, dwMilliseconds: u32) -> u32;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
    fn GetLastError() -> u32;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetStockObject(fn_object: i32) -> *mut std::ffi::c_void;
}



// Win32 constants
const COLOR_WINDOW: *mut std::ffi::c_void = 6 as *mut std::ffi::c_void;
const CS_HREDRAW: u32 = 2;
const CS_VREDRAW: u32 = 1;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BS_DEFPUSHBUTTON: u32 = 0x00000001;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;
const WM_SETFONT: u32 = 0x0030;

// Custom messages for background thread communication
const WM_APP: u32 = 0x8000;
const WM_INIT_CHECK_DONE: u32 = WM_APP;      // wParam = bit flags for checkbox states
const WM_APPLY_DONE: u32 = WM_APP + 1;       // wParam: 0=success, 1=uninstall_ok, 2=error

const DEFAULT_GUI_FONT: i32 = 17;

const GMEM_MOVEABLE: u32 = 0x0002;
const CF_UNICODETEXT: u32 = 13;
const CREATE_NO_WINDOW: u32 = 0x08000000;
const ERROR_ALREADY_EXISTS: u32 = 183;

// MessageBox constants
const MB_OK: u32 = 0x00000000;
const MB_ICONINFORMATION: u32 = 0x00000040;
const MB_ICONERROR: u32 = 0x00000010;

// Button States
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BST_CHECKED: usize = 1;
const BST_UNCHECKED: usize = 0;

// Window & Edit/List styles
const WS_BORDER: u32 = 0x00800000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const WS_VSCROLL: u32 = 0x00200000;
const LBS_NOTIFY: u32 = 0x0001;

// Notification Codes
const EN_CHANGE: usize = 0x0300;
const BN_CLICKED: usize = 0;

// ListBox messages
const LB_ADDSTRING: u32 = 0x0180;
const LB_RESETCONTENT: u32 = 0x0184;

// Control IDs - Installer GUI
const ID_CHK_COPY: i32 = 101;
const ID_CHK_CMD: i32 = 102;
const ID_CHK_PS: i32 = 103;
const ID_CHK_RENAME: i32 = 104;
const ID_BTN_APPLY: i32 = 105;
const ID_CHK_CLAUDE: i32 = 106;

// Control IDs - Rename GUI
const ID_TXT_FIND: i32 = 201;
const ID_TXT_REPLACE: i32 = 202;
const ID_TXT_PREFIX: i32 = 203;
const ID_TXT_SUFFIX: i32 = 204;
const ID_CHK_NUM: i32 = 205;
const ID_TXT_NUM_START: i32 = 206;
const ID_BTN_RENAME: i32 = 207;
const ID_LST_PREVIEW: i32 = 208;


// --- Helper Functions ---

fn to_wstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0).into_iter()).collect()
}

fn log_debug(msg: &str) {
    let mut st = SYSTEMTIME {
        wYear: 0, wMonth: 0, wDayOfWeek: 0, wDay: 0,
        wHour: 0, wMinute: 0, wSecond: 0, wMilliseconds: 0,
    };
    unsafe { GetLocalTime(&mut st); }
    let timestamp = format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}]",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    );
    if let Ok(mut exe_path) = env::current_exe() {
        exe_path.pop();
        let log_file = exe_path.join("copypathtool_debug.log");
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let _ = writeln!(file, "{} {}", timestamp, msg);
        }
    }
}

// --- State for Rename GUI ---
use std::sync::Mutex as StdMutex;

struct RenameState {
    paths: Vec<String>,
    hwnd_find: HWND,
    hwnd_replace: HWND,
    hwnd_prefix: HWND,
    hwnd_suffix: HWND,
    hwnd_chk_num: HWND,
    hwnd_num_start: HWND,
    hwnd_listbox: HWND,
}

unsafe impl Send for RenameState {}
unsafe impl Sync for RenameState {}

static RENAME_STATE: StdMutex<Option<RenameState>> = StdMutex::new(None);

use std::io::{Write as _, BufRead as _};

fn collect_paths(current_path: &str) -> Option<Vec<String>> {
    let temp_dir = env::temp_dir();
    let paths_file = temp_dir.join("copypathtool_paths.txt");
    
    let mutex_name = to_wstr("Local\\CopyPathToolMutex");
    unsafe {
        let h_mutex = CreateMutexW(ptr::null_mut(), 0, mutex_name.as_ptr());
        if h_mutex.is_null() {
            return Some(vec![current_path.to_string()]);
        }
        
        let is_leader = GetLastError() != ERROR_ALREADY_EXISTS;
        
        WaitForSingleObject(h_mutex, 0xFFFFFFFF);
        
        if is_leader {
            if let Ok(mut f) = fs::File::create(&paths_file) {
                let _ = writeln!(f, "{}", current_path);
            }
            
            ReleaseMutex(h_mutex);
            
            std::thread::sleep(std::time::Duration::from_millis(250));
            
            WaitForSingleObject(h_mutex, 0xFFFFFFFF);
            
            let mut paths = Vec::new();
            if let Ok(f) = fs::File::open(&paths_file) {
                let reader = std::io::BufReader::new(f);
                for line in reader.lines() {
                    if let Ok(l) = line {
                        let trimmed = l.trim().to_string();
                        if !trimmed.is_empty() && !paths.contains(&trimmed) {
                            paths.push(trimmed);
                        }
                    }
                }
            }
            
            let _ = fs::remove_file(&paths_file);
            
            ReleaseMutex(h_mutex);
            CloseHandle(h_mutex);
            
            paths.sort();
            Some(paths)
        } else {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&paths_file)
            {
                let _ = writeln!(f, "{}", current_path);
            }
            
            ReleaseMutex(h_mutex);
            CloseHandle(h_mutex);
            std::process::exit(0);
        }
    }
}

fn compute_new_name(
    stem: &str,
    ext: &str,
    find: &str,
    replace: &str,
    prefix: &str,
    suffix: &str,
    num_enabled: bool,
    num_start: i32,
    index: usize,
) -> String {
    let mut new_stem = stem.to_string();
    if !find.is_empty() {
        new_stem = new_stem.replace(find, replace);
    }
    new_stem = format!("{}{}{}", prefix, new_stem, suffix);
    if num_enabled {
        let val = num_start + index as i32;
        new_stem = format!("{}_{:02}", new_stem, val);
    }
    if ext.is_empty() {
        new_stem
    } else {
        format!("{}.{}", new_stem, ext)
    }
}

unsafe fn get_window_text(hwnd: HWND) -> String {
    let len = SendMessageW(hwnd, 0x000E, 0, 0) as usize; // WM_GETTEXTLENGTH
    if len == 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len + 1];
    SendMessageW(hwnd, 0x000D, len + 1, buf.as_mut_ptr() as isize); // WM_GETTEXT
    String::from_utf16_lossy(&buf[..len])
}

unsafe fn listbox_clear(hwnd: HWND) {
    SendMessageW(hwnd, LB_RESETCONTENT, 0, 0);
}

unsafe fn listbox_add_string(hwnd: HWND, text: &str) {
    let w_text = to_wstr(text);
    SendMessageW(hwnd, LB_ADDSTRING, 0, w_text.as_ptr() as isize);
}

fn refresh_preview() {
    if let Ok(mut guard) = RENAME_STATE.lock() {
        if let Some(ref mut state) = *guard {
            unsafe {
                listbox_clear(state.hwnd_listbox);
                
                let find = get_window_text(state.hwnd_find);
                let replace = get_window_text(state.hwnd_replace);
                let prefix = get_window_text(state.hwnd_prefix);
                let suffix = get_window_text(state.hwnd_suffix);
                let num_enabled = SendMessageW(state.hwnd_chk_num, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let num_start_str = get_window_text(state.hwnd_num_start);
                let num_start = num_start_str.parse::<i32>().unwrap_or(1);
                
                for (index, path_str) in state.paths.iter().enumerate() {
                    let path = Path::new(path_str);
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                    let ext = path.extension().unwrap_or_default().to_string_lossy().into_owned();
                    let orig_name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    
                    let new_name = compute_new_name(&stem, &ext, &find, &replace, &prefix, &suffix, num_enabled, num_start, index);
                    
                    let display_str = format!("{} -> {}", orig_name, new_name);
                    listbox_add_string(state.hwnd_listbox, &display_str);
                }
            }
        }
    }
}

fn apply_renaming(hwnd: HWND) {
    if let Ok(mut guard) = RENAME_STATE.lock() {
        if let Some(ref mut state) = *guard {
            unsafe {
                let find = get_window_text(state.hwnd_find);
                let replace = get_window_text(state.hwnd_replace);
                let prefix = get_window_text(state.hwnd_prefix);
                let suffix = get_window_text(state.hwnd_suffix);
                let num_enabled = SendMessageW(state.hwnd_chk_num, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let num_start_str = get_window_text(state.hwnd_num_start);
                let num_start = num_start_str.parse::<i32>().unwrap_or(1);
                
                let mut success_count = 0;
                let mut fail_count = 0;
                
                for (index, path_str) in state.paths.iter().enumerate() {
                    let path = Path::new(path_str);
                    let parent = path.parent().unwrap_or_else(|| Path::new(""));
                    let stem = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                    let ext = path.extension().unwrap_or_default().to_string_lossy().into_owned();
                    
                    let new_name = compute_new_name(&stem, &ext, &find, &replace, &prefix, &suffix, num_enabled, num_start, index);
                    let new_path = parent.join(new_name);
                    
                    if fs::rename(path, &new_path).is_ok() {
                        success_count += 1;
                    } else {
                        fail_count += 1;
                    }
                }
                
                let msg = if fail_count == 0 {
                    format!("重新命名成功！共修改了 {} 個檔案。", success_count)
                } else {
                    format!("修改完成。成功：{}，失敗：{}。", success_count, fail_count)
                };
                
                show_message(hwnd, &msg, "批次修改結果", MB_OK | MB_ICONINFORMATION);
                PostQuitMessage(0);
            }
        }
    }
}

// --- Rename Window Procedure ---
unsafe extern "system" fn rename_wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            // Options Controls (Left Side)
            let _lbl_find = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("搜尋文字 (Find):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 20, 280, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_find = CreateWindowExW(
                0, to_wstr("EDIT").as_ptr(), to_wstr("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                20, 40, 280, 25, hwnd, ID_TXT_FIND as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let _lbl_replace = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("取代為 (Replace):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 75, 280, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_replace = CreateWindowExW(
                0, to_wstr("EDIT").as_ptr(), to_wstr("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                20, 95, 280, 25, hwnd, ID_TXT_REPLACE as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let _lbl_prefix = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("新增前綴 (Prefix):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 130, 280, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_prefix = CreateWindowExW(
                0, to_wstr("EDIT").as_ptr(), to_wstr("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                20, 150, 280, 25, hwnd, ID_TXT_PREFIX as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let _lbl_suffix = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("新增後綴 (Suffix):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 185, 280, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_suffix = CreateWindowExW(
                0, to_wstr("EDIT").as_ptr(), to_wstr("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                20, 205, 280, 25, hwnd, ID_TXT_SUFFIX as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let hwnd_chk_num = CreateWindowExW(
                0, to_wstr("BUTTON").as_ptr(), to_wstr(" 啟用數字編號").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                20, 245, 280, 25, hwnd, ID_CHK_NUM as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let _lbl_num_start = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("起始編號:").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 275, 80, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_num_start = CreateWindowExW(
                0, to_wstr("EDIT").as_ptr(), to_wstr("1").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                110, 275, 60, 25, hwnd, ID_TXT_NUM_START as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            let hwnd_btn_rename = CreateWindowExW(
                0, to_wstr("BUTTON").as_ptr(), to_wstr("套用修改").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                20, 350, 280, 40, hwnd, ID_BTN_RENAME as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            // Preview Listbox (Right Side)
            let _lbl_preview = CreateWindowExW(
                0, to_wstr("STATIC").as_ptr(), to_wstr("變更預覽 (Preview):").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                330, 20, 390, 20, hwnd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()
            );
            let hwnd_listbox = CreateWindowExW(
                0, to_wstr("LISTBOX").as_ptr(), to_wstr("").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | WS_VSCROLL | LBS_NOTIFY,
                330, 40, 390, 350, hwnd, ID_LST_PREVIEW as *mut std::ffi::c_void, ptr::null_mut(), ptr::null_mut()
            );

            // Apply system font
            let h_font = GetStockObject(DEFAULT_GUI_FONT);
            SendMessageW(hwnd_find, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_replace, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_prefix, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_suffix, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_chk_num, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_num_start, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_btn_rename, WM_SETFONT, h_font as usize, 1);
            SendMessageW(hwnd_listbox, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_find, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_replace, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_prefix, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_suffix, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_num_start, WM_SETFONT, h_font as usize, 1);
            SendMessageW(_lbl_preview, WM_SETFONT, h_font as usize, 1);

            // Update state holding the handles
            if let Ok(mut guard) = RENAME_STATE.lock() {
                if let Some(ref mut state) = *guard {
                    state.hwnd_find = hwnd_find;
                    state.hwnd_replace = hwnd_replace;
                    state.hwnd_prefix = hwnd_prefix;
                    state.hwnd_suffix = hwnd_suffix;
                    state.hwnd_chk_num = hwnd_chk_num;
                    state.hwnd_num_start = hwnd_num_start;
                    state.hwnd_listbox = hwnd_listbox;
                }
            }

            refresh_preview();
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let code = ((wparam >> 16) & 0xFFFF) as usize;
            
            if (id == ID_TXT_FIND || id == ID_TXT_REPLACE || id == ID_TXT_PREFIX || id == ID_TXT_SUFFIX || id == ID_TXT_NUM_START) && code == EN_CHANGE {
                refresh_preview();
            } else if id == ID_CHK_NUM && code == BN_CLICKED {
                refresh_preview();
            } else if id == ID_BTN_RENAME {
                apply_renaming(hwnd);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn show_message(hwnd: HWND, text: &str, caption: &str, utype: u32) {
    let w_text = to_wstr(text);
    let w_caption = to_wstr(caption);
    unsafe { MessageBoxW(hwnd, w_text.as_ptr(), w_caption.as_ptr(), utype); }
}

fn copy_to_clipboard(text: &str) -> bool {
    let os_str = OsStr::new(text);
    let mut utf16: Vec<u16> = os_str.encode_wide().collect();
    utf16.push(0);

    let byte_len = utf16.len() * std::mem::size_of::<u16>();

    unsafe {
        let h_mem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if h_mem.is_null() {
            return false;
        }

        let ptr = GlobalLock(h_mem);
        if ptr.is_null() {
            return false;
        }

        ptr::copy_nonoverlapping(utf16.as_ptr(), ptr as *mut u16, utf16.len());
        GlobalUnlock(h_mem);

        if OpenClipboard(ptr::null_mut()) == 0 {
            return false;
        }

        EmptyClipboard();
        let res = SetClipboardData(CF_UNICODETEXT, h_mem);
        CloseClipboard();

        !res.is_null()
    }
}

// Cached admin check — only spawns reg.exe once per process lifetime
static IS_ADMIN_CACHE: OnceLock<bool> = OnceLock::new();

fn is_admin() -> bool {
    *IS_ADMIN_CACHE.get_or_init(|| {
        let output = Command::new("reg.exe")
            .args(&["add", "HKLM\\Software\\CopyPathTool_CheckAdmin", "/v", "Test", "/t", "REG_SZ", "/d", "1", "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let _ = Command::new("reg.exe")
                    .args(&["delete", "HKLM\\Software\\CopyPathTool_CheckAdmin", "/f"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
                return true;
            }
        }
        false
    })
}

// Apply result storage for cross-thread communication
static APPLY_RESULT_MSG: StdMutex<Option<(u32, String, String)>> = StdMutex::new(None);

// Check if registry key exists
fn check_key_exists(subkey: &str) -> bool {
    let root_str = if is_admin() { "HKLM" } else { "HKCU" };
    
    // Check primary root
    let output = Command::new("reg.exe")
        .args(&["query", &format!("{}\\{}", root_str, subkey)])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            log_debug(&format!("check_key_exists: {} found under {}.", subkey, root_str));
            return true;
        }
    }
    log_debug(&format!("check_key_exists: {} NOT found under {}.", subkey, root_str));
    
    // If primary is HKLM, fallback to HKCU
    if root_str == "HKLM" {
        let output_hkcu = Command::new("reg.exe")
            .args(&["query", &format!("HKCU\\{}", subkey)])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        if let Ok(out) = output_hkcu {
            if out.status.success() {
                log_debug(&format!("check_key_exists: {} found under HKCU (fallback).", subkey));
                return true;
            }
        }
        log_debug(&format!("check_key_exists: {} NOT found under HKCU fallback either.", subkey));
    }
    
    false
}

// Write a registry string value
fn set_registry_string(root_str: &str, subkey: &str, value_name: &str, value: &str) -> bool {
    log_debug(&format!("set_registry_string: Root: {}, Key: {}, Name: {}, Value: {}", root_str, subkey, value_name, value));
    let mut args = vec!["add".to_string(), format!("{}\\{}", root_str, subkey)];
    if !value_name.is_empty() {
        args.push("/v".to_string());
        args.push(value_name.to_string());
    } else {
        args.push("/ve".to_string());
    }
    args.push("/t".to_string());
    args.push("REG_SZ".to_string());
    args.push("/d".to_string());
    args.push(value.to_string());
    args.push("/f".to_string());

    let output = Command::new("reg.exe")
        .args(&args)
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                log_debug("set_registry_string: Succeeded.");
                true
            } else {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                log_debug(&format!("set_registry_string FAILED: reg.exe returned exit code {}. Error: {}", out.status.code().unwrap_or(-1), err_msg));
                false
            }
        }
        Err(e) => {
            log_debug(&format!("set_registry_string FAILED: failed to run reg.exe: {}", e));
            false
        }
    }
}

// Delete registry key
fn delete_registry_key(root_str: &str, key_path: &str) {
    log_debug(&format!("delete_registry_key: Root: {}, Key: {}", root_str, key_path));
    let _ = Command::new("reg.exe")
        .args(&["delete", &format!("{}\\{}", root_str, key_path), "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}

fn install_and_register(enable_copy: bool, enable_cmd: bool, enable_ps: bool, enable_rename: bool, enable_claude: bool) -> Result<(), String> {
    log_debug(&format!("install_and_register: Copy={}, CMD={}, PS={}, Rename={}, Claude={}", enable_copy, enable_cmd, enable_ps, enable_rename, enable_claude));
    let current_exe = env::current_exe().map_err(|e| format!("無法取得目前執行檔路徑: {}", e))?;
    let program_data = env::var("ProgramData").map_err(|e| format!("無法取得 ProgramData 環境變數: {}", e))?;
    let dest_dir = Path::new(&program_data).join("CopyPathTool");
    let dest_exe = dest_dir.join("CopyPathTool.exe");
    
    log_debug(&format!("install_and_register: current_exe={:?}, dest_exe={:?}", current_exe, dest_exe));
    
    if !dest_dir.exists() {
        log_debug("install_and_register: Creating installation directory...");
        fs::create_dir_all(&dest_dir).map_err(|e| format!("無法建立安裝目錄: {}", e))?;
    }
    
    if current_exe != dest_exe {
        log_debug("install_and_register: Copying current exe to ProgramData...");
        fs::copy(&current_exe, &dest_exe).map_err(|e| format!("無法複製執行檔: {}", e))?;
    }
    
    let exe_path_str = dest_exe.to_str().ok_or("執行檔路徑轉換失敗")?;
    
    let root_str = if is_admin() { "HKLM" } else { "HKCU" };

    let write_reg = |subkey: &str, value_name: &str, value: &str| -> Result<(), String> {
        if !set_registry_string(root_str, subkey, value_name, value) {
            return Err(format!("無法寫入登錄值: {}\\{}，請確認是否有權限或被防毒軟體阻擋。", subkey, value_name));
        }
        Ok(())
    };

    // 1. Copy Path feature
    if enable_copy {
        let targets = vec![
            ("Software\\Classes\\*\\shell\\CopyPath", "複製路徑", "shell32.dll,-16763", format!("\"{}\" \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\shell\\CopyPath", "複製路徑", "shell32.dll,-16763", format!("\"{}\" \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\Background\\shell\\CopyPath", "複製路徑", "shell32.dll,-16763", format!("\"{}\" \"%V\"", exe_path_str)),
        ];
        for (subkey, name, icon, command) in targets {
            write_reg(subkey, "", name)?;
            write_reg(subkey, "Icon", icon)?;
            write_reg(&format!("{}\\command", subkey), "", &command)?;
        }
    } else {
        delete_registry_key(root_str, "Software\\Classes\\*\\shell\\CopyPath");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\shell\\CopyPath");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\Background\\shell\\CopyPath");
    }

    // 2. Open CMD Here feature
    if enable_cmd {
        let targets = vec![
            ("Software\\Classes\\*\\shell\\OpenCMD", "在此處開啟 CMD", "cmd.exe", format!("\"{}\" --cmd \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\shell\\OpenCMD", "在此處開啟 CMD", "cmd.exe", format!("\"{}\" --cmd \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\Background\\shell\\OpenCMD", "在此處開啟 CMD", "cmd.exe", format!("\"{}\" --cmd \"%V\"", exe_path_str)),
        ];
        for (subkey, name, icon, command) in targets {
            write_reg(subkey, "", name)?;
            write_reg(subkey, "Icon", icon)?;
            write_reg(&format!("{}\\command", subkey), "", &command)?;
        }
    } else {
        delete_registry_key(root_str, "Software\\Classes\\*\\shell\\OpenCMD");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\shell\\OpenCMD");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\Background\\shell\\OpenCMD");
    }

    // 3. Open PowerShell Here feature
    if enable_ps {
        let targets = vec![
            ("Software\\Classes\\*\\shell\\OpenPS", "在此處開啟 PowerShell", "powershell.exe", format!("\"{}\" --ps \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\shell\\OpenPS", "在此處開啟 PowerShell", "powershell.exe", format!("\"{}\" --ps \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\Background\\shell\\OpenPS", "在此處開啟 PowerShell", "powershell.exe", format!("\"{}\" --ps \"%V\"", exe_path_str)),
        ];
        for (subkey, name, icon, command) in targets {
            write_reg(subkey, "", name)?;
            write_reg(subkey, "Icon", icon)?;
            write_reg(&format!("{}\\command", subkey), "", &command)?;
        }
    } else {
        delete_registry_key(root_str, "Software\\Classes\\*\\shell\\OpenPS");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\shell\\OpenPS");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\Background\\shell\\OpenPS");
    }

    // 4. Batch Rename feature
    if enable_rename {
        let targets = vec![
            ("Software\\Classes\\*\\shell\\BatchRename", "批次修改檔名", "shell32.dll,-16815", format!("\"{}\" --rename \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\shell\\BatchRename", "批次修改檔名", "shell32.dll,-16815", format!("\"{}\" --rename \"%1\"", exe_path_str)),
        ];
        for (subkey, name, icon, command) in targets {
            write_reg(subkey, "", name)?;
            write_reg(subkey, "Icon", icon)?;
            write_reg(&format!("{}\\command", subkey), "", &command)?;
        }
    } else {
        delete_registry_key(root_str, "Software\\Classes\\*\\shell\\BatchRename");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\shell\\BatchRename");
    }

    // 5. Open Claude YOLO feature
    if enable_claude {
        let targets = vec![
            ("Software\\Classes\\*\\shell\\OpenClaude", "在此處開啟 Claude (YOLO)", "cmd.exe", format!("\"{}\" --claude \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\shell\\OpenClaude", "在此處開啟 Claude (YOLO)", "cmd.exe", format!("\"{}\" --claude \"%1\"", exe_path_str)),
            ("Software\\Classes\\Directory\\Background\\shell\\OpenClaude", "在此處開啟 Claude (YOLO)", "cmd.exe", format!("\"{}\" --claude \"%V\"", exe_path_str)),
        ];
        for (subkey, name, icon, command) in targets {
            write_reg(subkey, "", name)?;
            write_reg(subkey, "Icon", icon)?;
            write_reg(&format!("{}\\command", subkey), "", &command)?;
        }
    } else {
        delete_registry_key(root_str, "Software\\Classes\\*\\shell\\OpenClaude");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\shell\\OpenClaude");
        delete_registry_key(root_str, "Software\\Classes\\Directory\\Background\\shell\\OpenClaude");
    }

    Ok(())
}

fn uninstall_all() {
    let keys = vec![
        "Software\\Classes\\*\\shell\\CopyPath",
        "Software\\Classes\\*\\shell\\OpenCMD",
        "Software\\Classes\\*\\shell\\OpenPS",
        "Software\\Classes\\*\\shell\\BatchRename",
        "Software\\Classes\\*\\shell\\OpenClaude",
        "Software\\Classes\\Directory\\shell\\CopyPath",
        "Software\\Classes\\Directory\\shell\\OpenCMD",
        "Software\\Classes\\Directory\\shell\\OpenPS",
        "Software\\Classes\\Directory\\shell\\BatchRename",
        "Software\\Classes\\Directory\\shell\\OpenClaude",
        "Software\\Classes\\Directory\\Background\\shell\\CopyPath",
        "Software\\Classes\\Directory\\Background\\shell\\OpenCMD",
        "Software\\Classes\\Directory\\Background\\shell\\OpenPS",
        "Software\\Classes\\Directory\\Background\\shell\\BatchRename",
        "Software\\Classes\\Directory\\Background\\shell\\OpenClaude",
    ];
    for k in keys {
        delete_registry_key("HKCU", k);
        delete_registry_key("HKLM", k);
    }
    
    // Attempt deleting folder
    if let Ok(program_data) = env::var("ProgramData") {
        let dest_dir = Path::new(&program_data).join("CopyPathTool");
        let dest_exe = dest_dir.join("CopyPathTool.exe");
        if let Ok(current_exe) = env::current_exe() {
            if current_exe == dest_exe {
                let cmd_str = format!(
                    "ping 127.0.0.1 -n 2 > nul && del /f /q \"{}\" && rmdir \"{}\"",
                    dest_exe.to_str().unwrap(),
                    dest_dir.to_str().unwrap()
                );
                let _ = Command::new("cmd.exe")
                    .args(&["/c", &cmd_str])
                    .current_dir("C:\\")
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
                std::process::exit(0);
            } else {
                if dest_exe.exists() {
                    let _ = fs::remove_file(&dest_exe);
                }
                if dest_dir.exists() {
                    let _ = fs::remove_dir(&dest_dir);
                }
            }
        }
    }
}

// --- Window Procedure ---

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            // Create all controls first (unchecked) — window appears instantly
            // Checkboxes
            let chk_copy = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr(" 啟用「複製路徑」右鍵選單").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 25, 300, 25,
                hwnd,
                ID_CHK_COPY as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            let chk_cmd = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr(" 啟用「在此處開啟 CMD」右鍵選單").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 60, 300, 25,
                hwnd,
                ID_CHK_CMD as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            let chk_ps = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr(" 啟用「在此處開啟 PowerShell」右鍵選單").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 95, 300, 25,
                hwnd,
                ID_CHK_PS as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            let chk_rename = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr(" 啟用「批次修改檔名」右鍵選單").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 130, 300, 25,
                hwnd,
                ID_CHK_RENAME as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            let chk_claude = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr(" 啟用「在此處開啟 Claude (YOLO)」右鍵選單").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 165, 300, 25,
                hwnd,
                ID_CHK_CLAUDE as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            // Apply Button
            let btn_apply = CreateWindowExW(
                0,
                to_wstr("BUTTON").as_ptr(),
                to_wstr("套用設定").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                110, 210, 130, 35,
                hwnd,
                ID_BTN_APPLY as *mut std::ffi::c_void,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            // Modernize fonts
            let h_font = GetStockObject(DEFAULT_GUI_FONT);
            SendMessageW(chk_copy, WM_SETFONT, h_font as usize, 1);
            SendMessageW(chk_cmd, WM_SETFONT, h_font as usize, 1);
            SendMessageW(chk_ps, WM_SETFONT, h_font as usize, 1);
            SendMessageW(chk_rename, WM_SETFONT, h_font as usize, 1);
            SendMessageW(chk_claude, WM_SETFONT, h_font as usize, 1);
            SendMessageW(btn_apply, WM_SETFONT, h_font as usize, 1);

            // Spawn background thread to check registry states (non-blocking)
            let hwnd_val = hwnd as usize;
            std::thread::spawn(move || {
                let has_copy = check_key_exists("Software\\Classes\\*\\shell\\CopyPath");
                let has_cmd = check_key_exists("Software\\Classes\\Directory\\shell\\OpenCMD");
                let has_ps = check_key_exists("Software\\Classes\\Directory\\shell\\OpenPS");
                let has_rename = check_key_exists("Software\\Classes\\*\\shell\\BatchRename");
                let has_claude = check_key_exists("Software\\Classes\\Directory\\shell\\OpenClaude");

                // Pack results into bit flags
                let flags: usize =
                    (has_copy as usize)
                    | ((has_cmd as usize) << 1)
                    | ((has_ps as usize) << 2)
                    | ((has_rename as usize) << 3)
                    | ((has_claude as usize) << 4);

                PostMessageW(hwnd_val as HWND, WM_INIT_CHECK_DONE, flags, 0);
            });

            0
        }
        WM_INIT_CHECK_DONE => {
            // Background thread finished checking registry — update checkbox states
            let has_copy  = (wparam & 1) != 0;
            let has_cmd   = (wparam & 2) != 0;
            let has_ps    = (wparam & 4) != 0;
            let has_rename = (wparam & 8) != 0;
            let has_claude = (wparam & 16) != 0;

            let chk_copy = GetDlgItem(hwnd, ID_CHK_COPY);
            let chk_cmd = GetDlgItem(hwnd, ID_CHK_CMD);
            let chk_ps = GetDlgItem(hwnd, ID_CHK_PS);
            let chk_rename = GetDlgItem(hwnd, ID_CHK_RENAME);
            let chk_claude = GetDlgItem(hwnd, ID_CHK_CLAUDE);

            SendMessageW(chk_copy, BM_SETCHECK, if has_copy { BST_CHECKED } else { BST_UNCHECKED }, 0);
            SendMessageW(chk_cmd, BM_SETCHECK, if has_cmd { BST_CHECKED } else { BST_UNCHECKED }, 0);
            SendMessageW(chk_ps, BM_SETCHECK, if has_ps { BST_CHECKED } else { BST_UNCHECKED }, 0);
            SendMessageW(chk_rename, BM_SETCHECK, if has_rename { BST_CHECKED } else { BST_UNCHECKED }, 0);
            SendMessageW(chk_claude, BM_SETCHECK, if has_claude { BST_CHECKED } else { BST_UNCHECKED }, 0);
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let code = ((wparam >> 16) & 0xFFFF) as usize;
            log_debug(&format!("wnd_proc WM_COMMAND: id={}, code={}", id, code));
            
            if (id >= 101 && id <= 104) || id == ID_CHK_CLAUDE {
                log_debug(&format!("wnd_proc: Checkbox ID={} click event (code={}) received.", id, code));
            }
            
            if id == ID_BTN_APPLY {
                log_debug("wnd_proc: Apply button clicked.");

                // Read checkbox states on UI thread
                let chk_copy = GetDlgItem(hwnd, ID_CHK_COPY);
                let chk_cmd = GetDlgItem(hwnd, ID_CHK_CMD);
                let chk_ps = GetDlgItem(hwnd, ID_CHK_PS);
                let chk_rename = GetDlgItem(hwnd, ID_CHK_RENAME);
                let chk_claude = GetDlgItem(hwnd, ID_CHK_CLAUDE);
                
                let enable_copy = SendMessageW(chk_copy, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let enable_cmd = SendMessageW(chk_cmd, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let enable_ps = SendMessageW(chk_ps, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let enable_rename = SendMessageW(chk_rename, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let enable_claude = SendMessageW(chk_claude, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;

                // Disable apply button to prevent double-click
                let btn_apply = GetDlgItem(hwnd, ID_BTN_APPLY);
                EnableWindow(btn_apply, 0);

                // Spawn background thread for registry operations
                let hwnd_val = hwnd as usize;
                std::thread::spawn(move || {
                    log_debug(&format!("apply_settings called: Copy={}, CMD={}, PS={}, Rename={}, Claude={}", enable_copy, enable_cmd, enable_ps, enable_rename, enable_claude));

                    if !enable_copy && !enable_cmd && !enable_ps && !enable_rename && !enable_claude {
                        uninstall_all();
                        if let Ok(mut guard) = APPLY_RESULT_MSG.lock() {
                            *guard = Some((MB_OK | MB_ICONINFORMATION, "成功".to_string(), "設定已套用！所有右鍵選單功能已成功移除。".to_string()));
                        }
                        PostMessageW(hwnd_val as HWND, WM_APPLY_DONE, 1, 0); // 1 = uninstall, quit after
                        return;
                    }

                    match install_and_register(enable_copy, enable_cmd, enable_ps, enable_rename, enable_claude) {
                        Ok(_) => {
                            log_debug("apply_settings: install_and_register Succeeded.");
                            if let Ok(mut guard) = APPLY_RESULT_MSG.lock() {
                                *guard = Some((MB_OK | MB_ICONINFORMATION, "成功".to_string(), "設定已套用！右鍵選單功能已成功更新。\n程式已複製到本機 ProgramData 目錄儲存。".to_string()));
                            }
                            PostMessageW(hwnd_val as HWND, WM_APPLY_DONE, 0, 0);
                        }
                        Err(e) => {
                            log_debug(&format!("apply_settings FAILED: {}", e));
                            if let Ok(mut guard) = APPLY_RESULT_MSG.lock() {
                                *guard = Some((MB_OK | MB_ICONERROR, "錯誤".to_string(), format!("設定套用失敗：{}", e)));
                            }
                            PostMessageW(hwnd_val as HWND, WM_APPLY_DONE, 2, 0);
                        }
                    }
                });
            }
            0
        }
        WM_APPLY_DONE => {
            // Background apply thread finished — show result on UI thread
            let result = if let Ok(mut guard) = APPLY_RESULT_MSG.lock() {
                guard.take()
            } else {
                None
            };

            if let Some((flags, caption, message)) = result {
                show_message(hwnd, &message, &caption, flags);
            }

            if wparam == 1 {
                // Uninstall completed — quit
                PostQuitMessage(0);
            } else {
                // Re-enable apply button
                let btn_apply = GetDlgItem(hwnd, ID_BTN_APPLY);
                EnableWindow(btn_apply, 1);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// --- Main Entry ---

fn main() {
    let args: Vec<String> = env::args().collect();
    log_debug(&format!("--- App started (v{}) --- args: {:?}", VERSION, args));
    
    if args.len() == 1 {
        // Double-clicked: show settings GUI
        let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
        let class_name = to_wstr("CopyPathToolGUIClass");
        
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: COLOR_WINDOW,
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        
        unsafe {
            RegisterClassW(&wnd_class);
            
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                to_wstr(&format!("右鍵選單工具設定精靈 v{}", VERSION)).as_ptr(),
                WS_OVERLAPPEDWINDOW & !0x00040000 & !0x00010000, // No resize, no maximize
                400, 300, 370, 310,
                ptr::null_mut(),
                ptr::null_mut(),
                h_instance,
                ptr::null_mut(),
            );
            
            if hwnd.is_null() {
                return;
            }
            
            ShowWindow(hwnd, 5); // SW_SHOW
            UpdateWindow(hwnd);
            
            let mut msg = MSG {
                hwnd: ptr::null_mut(),
                message: 0,
                wParam: 0,
                lParam: 0,
                time: 0,
                pt: POINT { x: 0, y: 0 },
            };
            
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    } else if args.len() == 2 && args[1] == "--install" {
        // Silent install all features
        if let Err(e) = install_and_register(true, true, true, true, true) {
            log_debug(&format!("Silent install failed: {}", e));
            eprintln!("Silent install failed: {}", e);
            std::process::exit(1);
        }
    } else if args.len() == 2 && args[1] == "--uninstall" {
        // Silent uninstall
        uninstall_all();
    } else if args.len() >= 3 && args[1] == "--rename" {
        // Batch rename mode
        let path = &args[2];
        if let Some(paths) = collect_paths(path) {
            if paths.is_empty() {
                return;
            }
            
            // Set global state for renaming
            if let Ok(mut guard) = RENAME_STATE.lock() {
                *guard = Some(RenameState {
                    paths,
                    hwnd_find: ptr::null_mut(),
                    hwnd_replace: ptr::null_mut(),
                    hwnd_prefix: ptr::null_mut(),
                    hwnd_suffix: ptr::null_mut(),
                    hwnd_chk_num: ptr::null_mut(),
                    hwnd_num_start: ptr::null_mut(),
                    hwnd_listbox: ptr::null_mut(),
                });
            }
            
            // Show the Batch Rename GUI
            let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
            let class_name = to_wstr("CopyPathToolRenameClass");
            
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(rename_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: ptr::null_mut(),
                hCursor: ptr::null_mut(),
                hbrBackground: COLOR_WINDOW,
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            
            unsafe {
                RegisterClassW(&wnd_class);
                
                let hwnd = CreateWindowExW(
                    0,
                    class_name.as_ptr(),
                    to_wstr("批次重新命名工具").as_ptr(),
                    WS_OVERLAPPEDWINDOW & !0x00040000 & !0x00010000, // No resize, no maximize
                    300, 200, 750, 480,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    h_instance,
                    ptr::null_mut(),
                );
                
                if hwnd.is_null() {
                    return;
                }
                
                ShowWindow(hwnd, 5); // SW_SHOW
                UpdateWindow(hwnd);
                
                let mut msg = MSG {
                    hwnd: ptr::null_mut(),
                    message: 0,
                    wParam: 0,
                    lParam: 0,
                    time: 0,
                    pt: POINT { x: 0, y: 0 },
                };
                
                while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    } else if args.len() >= 3 && (args[1] == "--cmd" || args[1] == "--ps" || args[1] == "--claude") {
        // Command/PowerShell/Claude launcher mode
        let path_str = &args[2];
        let path = Path::new(path_str);
        
        let target_dir = if path.is_file() {
            path.parent().unwrap_or_else(|| Path::new("C:\\"))
        } else {
            path
        };
        
        if args[1] == "--cmd" {
            let _ = Command::new("cmd.exe")
                .current_dir(target_dir)
                .spawn();
        } else if args[1] == "--ps" {
            let _ = Command::new("powershell.exe")
                .current_dir(target_dir)
                .spawn();
        } else if args[1] == "--claude" {
            let _ = Command::new("cmd.exe")
                .args(&["/k", "claude --dangerously-skip-permissions"])
                .current_dir(target_dir)
                .spawn();
        }
    } else if args.len() >= 2 {
        // Copy path mode (supports multiple file paths selection)
        let path = &args[1];
        if let Some(paths) = collect_paths(path) {
            let combined = paths.join("\r\n");
            copy_to_clipboard(&combined);
        }
    }
}
