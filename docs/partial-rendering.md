# 终端增量渲染

从 `0.1.11-beta.4` 起，下列优化默认开启。终端缓存与调度适用于所有平台，
局部绘制补丁仅用于 Windows：

- 终端快照读取 Alacritty 的变化行，复用未变化行的字符数据和排版结果。
  光标闪烁、移动不再重新构造整个屏幕的文字。选择、搜索、跨行链接和
  关键字高亮仍保持原有行为；字体、缩放、颜色和尺寸变化会使相应缓存失效。
- Windows 渲染后端保留完整画面，只清除、重绘并提交变化的矩形区域。
  变化区域内的背景、阴影和浮层会一起重绘；窗口缩放和设备恢复会完整重绘。

持续滚动或全屏程序可能改变大部分画面，此时收益会小于静态终端输入和
光标闪烁。第二轮支持多个变化矩形，避免相距很远的变化扩大重绘面积。
Windows 每个窗口额外保留一张 BGRA 图像，4K 窗口约占 32 MiB。

## 刷新与调度

这些改动已在 `0.1.11-beta.3` 的两轮本地测试包中验证，并随 `0.1.11-beta.4`
一起发布。

- 光标、选区和普通输出刷新不再被转成面板状态变化；连接状态、标题、
  尺寸、延迟与重连代次变化仍会通知工作区。
- URL 识别和关键字高亮复用未变化的逻辑行，编辑换行边界时同时刷新原有
  与新的完整逻辑行。选区不变时的光标闪烁复用装饰结果；鼠标链接查询
  直接使用共享行数据，不再每帧复制整屏单元格索引。打开的搜索仍会在
  文本变化时检查可见匹配，确保焦点匹配及时失效。
- Windows 后端按空间区域比较画面，最多提交 16 个变化矩形。相距很远的
  分屏变化可分开绘制；区域过多或超过约 70% 画面时合并或完整重绘。
- 有输出的可见终端保持 16ms 检查周期，空闲后逐步退至 32/64ms；隐藏
  标签为 100ms。输入、获得焦点和重新显示会立即唤醒检查。后台传输和
  终端解析继续运行，所有可见分屏均按可见终端处理。

这些变化没有禁用光标闪烁、搜索、高亮或广播。原有两个环境变量只回退
终端行缓存和局部绘制；若比较全部优化，请使用原版 beta.3 与 beta.4 安装包。

## 在无显卡服务器上对比

安装后完全退出 ShellRS，可以通过下方环境变量控制两条渲染路径。
之前的本地测试 ZIP 另附下列启动脚本，GitHub Release 只提供标准安装包：

| 脚本 | 终端缓存 | Windows 局部绘制 |
| --- | --- | --- |
| `start-optimized.cmd` | 开启 | 开启 |
| `start-full-render.cmd` | 关闭 | 关闭 |
| `start-cache-only.cmd` | 开启 | 关闭 |
| `start-partial-only.cmd` | 关闭 | 开启 |

脚本默认查找 `%LOCALAPPDATA%\Programs\ShellRS\shellrs.exe`，也可以把
安装路径作为第一个参数传入。仅为本次启动设置环境变量，不修改系统配置。

保持相同 RDP 分辨率、窗口大小和分屏数量，分别对比空闲 CPU 占用、输入
回显、选择文字、切换标签和持续输出。检查中文、搜索高亮、滚动、弹出菜单
及调整窗口大小有无残影。关闭两项优化仍使用新后端的保留画面路径；如需
比较发布前的完整实现，请与原版 beta.3 安装包分别测试。

也可自行设置以下环境变量后启动；取消设置后恢复默认：

```powershell
$env:SHELLRS_DISABLE_TERMINAL_CACHE = '1'
$env:SHELLRS_DISABLE_PARTIAL_RENDER = '1'
& "$env:LOCALAPPDATA\Programs\ShellRS\shellrs.exe"
```

## 开发验证

```powershell
cargo test --locked --lib terminal::
cargo test --locked --test ui terminal::
cargo test --locked --test ui terminal_groups::
cargo test --locked -p gpui-pre-windows partial_warp -- --nocapture
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Windows 后端修改来源、回退行为及验证边界见
[`vendor/gpui-pre-windows/README.shellrs.md`](../vendor/gpui-pre-windows/README.shellrs.md)。
