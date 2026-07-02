fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "Windows 右鍵選單工具 (複製路徑 + CMD/PS/Claude)");
        res.set("ProductName", "CopyPathTool");
        res.set("OriginalFilename", "CopyPathTool.exe");
        res.set("CompanyName", "Ron Studio");
        res.set("FileVersion", "1.1.2");
        res.set("ProductVersion", "1.1.2");
        res.compile().unwrap();
    }
}
