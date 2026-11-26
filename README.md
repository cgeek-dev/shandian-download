<div align="center">

<img src="docs/images/icon.png" width="88" alt="闪电全能下载图标">

# 闪电全能下载 · Windows 普通版

下载直链、磁力、种子、网页媒体和视频合集。<br>
已测试 **Windows 10 / Windows 11**，当前发布为 **x64 · 简体中文**。

[下载最新版](https://github.com/cgeek-dev/shandian-download/releases/latest) · [构建说明](windows/README.md) · [更新记录](windows/CHANGELOG.md)

</div>

## 下载与安装

发布页提供 **安装版** 和 **便携版**。安装版按界面提示安装。便携版完整解压后运行「闪电全能下载.exe」。如需接管浏览器下载，可按闪电的提示在 Chrome 或 Edge 中添加扩展。

安装版可在「设置 → 关于」检查更新。发现新版后点「立即更新」，程序会下载安装包、校验并启动安装。便携版可从发布页获取新版。

## 主要功能

| 场景 | 可用操作 |
| --- | --- |
| 文件下载 | 添加 HTTP/HTTPS 直链、磁力、种子和 ed2k 任务，查看进度、暂停和继续。 |
| 视频与合集 | 支持 YouTube、哔哩哔哩等站点，可选择分集。来源支持时可下载和处理字幕。 |
| 网页资源 | Chrome 和 Edge 扩展支持接管下载、查找音视频与字幕、侦测正在播放的资源。 |
| 网盘和文档附件 | 在支持的网页选择文件后下载，实际可用操作取决于站点和账号权限。 |
| 文件抽屉 | 展开查看分集或文件进度。支持多选、框选和对选中项执行可用的字幕或复制操作。 |
| AI 工具 | 本机 MCP 提供新建下载、查进度、暂停和继续。配置入口在「设置 → AI 工具」。 |

## 源码

Windows 普通版使用 Tauri 和 WebView2，下载内核为 Gopeed，视频站下载使用 yt-dlp。Windows 源码位于 `windows/`。

[已合并的 Windows PR](https://github.com/chenmozhe008/shandian-download/pull/2) · [GPL-3.0 许可证](LICENSE)
