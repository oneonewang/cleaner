# oneonecleaner

> 基于 **Tauri 2 + Vue 3 + TypeScript + Element Plus** 的 Windows 系统清理器

## 功能

- 🗑️ **系统垃圾清理**:扫描用户临时文件、Windows 临时文件、Prefetch、缩略图缓存、错误报告、DirectX 着色器缓存、回收站等
- 🌐 **浏览器缓存清理**:支持 Chrome / Edge / Brave / Opera / Firefox(多 profile)
- 📦 **大文件/旧文件查找**:自定义目录、最小大小、最少未访问天数
- 🛠️ **注册表清理**:扫描 Run/RunOnce/Uninstall/COM 中的失效项,清理前自动 .reg 备份
- 🌗 **明暗主题 + 中英双语**

## 截图占位

应用运行后,默认打开概览页,可在左侧菜单切换清理器。

## 技术栈

| 类别 | 选型 |
| --- | --- |
| 桌面壳 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite |
| UI 库 | Element Plus 2.x |
| 路由 | Vue Router 4 |
| 状态 | Pinia |
| 国际化 | vue-i18n |
| 后端 | Rust 1.78+ |
| Windows API | `winreg`、`windows-sys`(PowerShell + explorer.exe 辅助) |

## 项目结构

```
oneonecleaner/
├── src/                  # Vue 前端
│   ├── api/              # Tauri invoke 封装
│   ├── components/       # 通用组件
│   ├── locales/          # 中英双语
│   ├── router/
│   ├── stores/           # Pinia
│   ├── styles/
│   ├── utils/
│   └── views/            # 页面
└── src-tauri/            # Rust 后端
    ├── icons/
    ├── capabilities/
    └── src/
        ├── commands/     # #[tauri::command] 入口
        ├── core/         # 业务核心(不依赖 Tauri,方便单测)
        └── models/       # 数据结构
```

## 开发

```bash
# 安装依赖
pnpm install

# 生成图标(可选,已生成在 src-tauri/icons/)
node scripts/gen-icons.cjs

# 开发模式(自动热重载)
pnpm tauri:dev

# 类型检查
pnpm typecheck

# 单独构建前端
pnpm build

# 构建可执行文件(不打包)
pnpm tauri build --no-bundle
```

## 打包安装程序

```bash
pnpm tauri build
```

> 注:NSIS / WiX 工具集首次构建时会从 GitHub 下载,若网络受限可使用 `pnpm tauri build --no-bundle` 仅产出 `oneonecleaner.exe`,或自行放置 NSIS / WiX 到 `C:\Users\<User>\AppData\Local\tauri\`。

## 安全性设计

- 默认**送入回收站**(可在设置中关闭)
- 注册表清理前**强制 .reg 备份**,备份文件位于 `%APPDATA%\oneonecleaner\backups\`
- 排除 `C:\Windows\System32` 等系统目录整目录删除
- 严格 CSP,仅启用必要的 Tauri 权限

## 路径与术语

| 类别 | 路径 |
| --- | --- |
| 用户临时 | `%TEMP%` |
| Windows 临时 | `<系统盘>:\Windows\Temp` |
| 缩略图缓存 | `%LOCALAPPDATA%\Microsoft\Windows\Explorer` |
| DirectX 缓存 | `%LOCALAPPDATA%\D3DSCache` |
| 错误报告 | `%LOCALAPPDATA%\Microsoft\Windows\WER` |
| 回收站 | 通过 PowerShell `Shell.Application` 读取 |

## License

MIT
