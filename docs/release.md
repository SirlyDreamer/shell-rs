# 发布

[返回 README](../README.zh-CN.md)

在 `CHANGELOG.md` 里写好 `## [x.y.z]` 一节，把 `Cargo.toml` 与 `Cargo.lock` 中 ShellRS 的版本改成 `x.y.z`，提交后推送标签 `vx.y.z`。沿用原有版本格式，例如 `0.1.11-beta.3`。

`.github/workflows/release.yml` 只通过 GitHub 发布：

1. 核对标签与 `Cargo.toml` 版本一致，并提取对应更新日志。
2. 并行构建 macOS Apple Silicon、macOS Intel、Windows x86_64 和 Linux x86_64。
3. macOS 生成 `.app.zip` 和 `.dmg`，Windows 使用 Inno Setup 生成安装程序，Linux 生成 AppImage；Linux 要求 glibc 2.31 或更新版本。
4. 所有构建成功后生成 `SHA256SUMS`，用 GitHub 自带的 `GITHUB_TOKEN` 创建当前仓库的 Release，上传全部安装包和校验文件。

版本号带 `-` 的发布为 GitHub prerelease，正式版本发布为普通 Release。手动运行 workflow 只生成 Actions 构建产物，不创建 Release。如果同版本 Release 已经手动创建，发布任务会补充或替换同名安装包与校验文件。

此流程不需要额外 secret 或 `release` 审批环境，不再调用 Cloudflare/R2、CDN 缓存刷新、官网部署挂钩和更新清单发布。构建阶段只需仓库读取权限，发布任务单独申请 `contents: write`。

安装包不启用上游的自动更新通道，后续更新从当前仓库的 GitHub Release 下载。macOS 包只使用本地 ad-hoc 签名，未经 Apple Developer ID 签名和公证，Gatekeeper 可能阻止首次打开；Windows 安装程序也未作代码签名。

`windows.yml` 和 `shells.yml` 保留平台检查与真实 shell 集成测试。它们按代码路径触发，也可以在 Actions 中手动运行。Workflow 显示已启用只代表可以触发，成功与否以具体运行结果为准。
