# HOI4Reader

[English](../README.md) · **简体中文** · [正體中文](README.zh-Hant.md) · [日本語](README.ja.md)

钢铁雄心 IV 模组剧情阅读器。选一个国家，沿国策和事件阅读，并决定剧情分支。

![TFR 主题的主页（示例数据）](screenshots/tfr-home.png)

- 自动寻找本机 Steam 库和创意工坊模组，也支持手动指定目录。
- 按脚本串联国策、事件、决议和国际新闻。保存阅读进度，并允许撤回分支选择。
- 从本机模组读取插图、超事件和音乐。新曲默认立即切换，可在设置中改为排队播放。
- 支持 Material Design 与 The Fire Rises 主题。默认英语，也可选择简体中文、正体中文和日语。剧情译文取自已安装的本地化文件，缺译时优先显示可用的英文。

**The Fire Rises（烈焰升腾）** 是目前阅读流程最完善的模组。The New Order 可以扫描，路线支持较有限。部分游戏脚本条件仍有待解析。

从源码运行需安装 Node.js、Rust 和 [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)。在项目目录执行 `npm ci`、`npm --prefix frontend ci`、`npm run tauri:dev`。安装包可通过 `npm run tauri:build` 构建。
