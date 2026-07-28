# AzurPilot Launcher 构建说明

## 唯一构建来源

正式 launcher 的构建源码是：

```text
D:\模拟器\alas-launcher-repair
```

正式运行产物是：

```text
D:\模拟器\AzurPilot\alas-launcher.exe
```

`D:\Andrew\Code\AzurLane\alas-launcher` 是个人主工作树，允许存在实验性未提交修改；生产构建前必须把已确认的修改同步到 repair 工作树。

## 固定行为约束

### 标题栏

Windows 主窗口必须使用 ALAS 自定义 Windows 风格标题栏：矩形按钮、灰色图标、关闭按钮悬停变红。不能改回 Windows 原生标题栏，也不能使用 Apple 彩色圆点版本。

### 托盘

托盘最小化必须调用 `window.hide()`，保留 WebView 和 PyWebIO session。禁止为了节省内存调用 `window.destroy()`，否则恢复窗口时可能白屏或重载。

## 构建与部署

```powershell
Set-Location D:\模拟器\alas-launcher-repair
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

确认测试通过后，退出正式 launcher，备份当前 exe，再复制：

```powershell
Copy-Item .\target\release\alas-launcher.exe D:\模拟器\AzurPilot\alas-launcher.exe -Force
```

部署后记录正式 exe 的 SHA256，并手动验收：启动、最小化到托盘、恢复、最大化、关闭菜单和完全退出。
