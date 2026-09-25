# AzurPilot Launcher 构建说明

## 唯一构建来源

正式 launcher 的构建源码是 fork 上的 `personal/alas-branding-v2` 分支（基于上游最新 main，只叠加个人改动）。

正式运行产物是：

```text
D:\模拟器\AzurPilot\alas-launcher.exe
```

`D:\Andrew\Code\AzurLane\alas-launcher` 是个人主工作树，允许存在实验性未提交修改；生产构建只从已提交的 `personal/alas-branding-v2` 进行。

## 固定行为约束

### 标题栏

Windows 主窗口必须使用 ALAS 自定义 Windows 风格标题栏：矩形按钮、灰色图标、关闭按钮悬停变红。不能改回 Windows 原生标题栏，也不能使用 Apple 彩色圆点版本。上游标题栏脚本负责位置与拖拽，外观由 `src/branding.rs` 的 `TITLEBAR_CSS` 覆盖。

### 托盘

托盘最小化沿用上游行为：Windows 上 `destroy()` 主窗口释放 WebView 资源，恢复时重新创建。

## 构建与部署

```powershell
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

确认测试通过后，退出正式 launcher，备份当前 exe，再复制：

```powershell
Copy-Item .\target\release\alas-launcher.exe D:\模拟器\AzurPilot\alas-launcher.exe -Force
```

部署后记录正式 exe 的 SHA256，并手动验收：启动、最小化到托盘、恢复、最大化、关闭菜单和完全退出。
