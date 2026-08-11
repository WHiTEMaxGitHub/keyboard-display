# Keyboard Display

Keyboard Display 是一个桌面端 POV 按键显示和录制工具。它通过原生输入后端捕获键盘和鼠标状态，在独立透明窗口中显示实时按键 Overlay，并把输入时间线保存为紧凑的 `.kbdrec` 文件，后续可重新渲染为透明 Overlay 视频。

英文说明见 [`README.ENGLISH.md`](README.ENGLISH.md)。

## 当前能力

- 桌面端 Tauri 应用，前端使用 Vue 3、TypeScript、Vite 和 Tailwind CSS。
- 自绘配置窗口标题栏，macOS 和 Windows 使用各自习惯的窗口按钮位置。
- 独立 POV Overlay 窗口，支持透明背景、置顶、位置调整和自定义布局。
- 原生全局输入捕获：
  - macOS 使用 CGEventTap。
  - Windows 使用 Raw Input 接收键盘输入，并用低级鼠标 hook 捕获鼠标按钮。
  - 其他平台会启动 unsupported 后端，配置 UI 仍可打开，但没有真实后台输入捕获。
- 可视化配置页面：
  - 概览：当前配置状态、实时预览、显示/置顶快捷开关。
  - 布局：摘要和编辑两个子页面，支持行、按键、间距、Key ID、展示名、宽度、`unitPx` 和 `gapUnit` 编辑。
  - 外观：Overlay 缩放、透明度、底板、圆角、按键颜色和透明底板。
  - 窗口：显示状态、置顶、预设位置和可视化调整。
  - 录制：保存目录、FPS、文件名模板、同步闪烁、录制控制热键、后台录制。
  - 录制文件：递归浏览 `.kbdrec`，创建文件夹，检查录制内容，编辑 sidecar 录制详情。
  - 导出：选择 `.kbdrec`、导出 Overlay WebM、选择/安装 ffmpeg、设置字体和渲染线程。
  - 设置：语言、主题、自定义配色、app config 路径和运行状态。
- 主题系统：
  - 内置 `vibrant`、`dark`、`midnight`、`light` 和 `custom`。
  - 自定义主题支持背景模板、色彩通道、HEX/RGBA 调色板和背景板透明度。
- 中英文界面，语言可设为跟随系统、English 或简体中文。

## 输入与 Overlay

输入事件先在 Rust 原生层归一化为稳定的 Key ID，再发送给配置窗口和 POV Overlay：

```text
native input backend -> normalized input event -> config window / POV overlay / recorder
```

常见 Key ID 示例：

- 键盘：`w`、`a`、`s`、`d`、`space`、`shift-left`、`ctrl-left`、`alt-left`、`meta-left`、`escape`
- 鼠标：`mouse-left`、`mouse-right`、`mouse-middle`

macOS 真实后台捕获通常需要 Accessibility/Input Monitoring 权限。平台细节见 [`docs/platform-notes.md`](docs/platform-notes.md)。

## 配置模型

项目有两类 JSON：

- Profile config：可分享、可手写的 POV 配置，包含布局、外观、录制默认值和导出偏好。
- App config：应用本地状态，包含当前配置快照、当前 profile 文件路径、录制目录、导出目录、热键、语言等。

内置 profile 存在于代码中，不要求用户拿到应用时同时带一组 JSON 文件。当前内置模板包括：

- `default`
- `left-keyboard`
- `68-keyboard`

Release 应用启动时会在应用程序同目录查找 `app-config.json`。如果文件不存在，或者已有文件不是合法 JSON，应用会用内置初始配置重新生成。Debug 模式仍使用平台默认 app config 目录，不会在开发二进制同目录生成配置。

初始 `app-config.json` 不预填用户存储路径：

- `currentProfile.sourcePath` 初始为 `null`。用户导出或加载 profile JSON 后，它记录当前配置文件路径，用于“覆盖并应用”写回磁盘。
- 录制目录初始为 `null`。开始录制时如果用户未选择目录，默认使用 app config 同目录下的 `recording-files`。
- 视频导出目录初始为 `null`。导出视频时如果用户未选择目录，默认使用 app config 同目录下的 `export-videos`。

应用不会在启动时创建 `pov-profiles` 之类的 profile 文件夹。Profile JSON 只在用户点击导出或保存配置，并选择文件位置后生成。

配置格式详见 [`docs/config-format.md`](docs/config-format.md)，示例见 [`docs/default-config.json`](docs/default-config.json) 和 [`docs/app-config.json`](docs/app-config.json)。

## 录制

录制文件扩展名是 `.kbdrec`。它保存输入状态时间线，不保存渲染后的视频帧：

```text
input events -> sampled frames -> .kbdrec
```

录制配置包括：

- 预设 FPS：`30`、`60`、`120`
- 自定义 FPS，上限 `1000`
- 文件名模板，默认 `${start}-${end}`
- 同步点和同步闪烁
- 录制控制热键：
  - 默认开始/停止：`ctrl-left + shift-left + r`
  - 默认同步点：`f8`
- 后台录制：录制期间销毁 POV 窗口，结束后恢复。

录制文件页支持读取 `.kbdrec` 摘要、事件、帧状态和 marker，并通过 sidecar metadata 保存显示名、说明、标签和 marker note。

## 导出

`.kbdrec` 是 source of truth，Overlay 视频是导出结果：

```text
.kbdrec -> replay engine -> overlay renderer -> ffmpeg -> .webm
```

当前导出重点是透明 Overlay WebM。导出页会检测三类 ffmpeg 来源：

- 应用托管 ffmpeg。
- 用户手动选择的 ffmpeg。
- 系统 PATH 中的 ffmpeg。

应用不会修改用户已有 ffmpeg。应用托管版本安装在应用数据目录下的 exporter 子目录。

导出线程数由 profile 的 `export.renderThreads` 控制：

- 留空或 `0`：使用当前 CPU 核心数。
- `-1`：高并发模式，使用 CPU 核心数乘以 `4`。
- 正数：使用用户指定线程数，但运行时会限制在 CPU 核心数乘以 `4` 以内。

导出输出目录初始为空。首次导出时如果用户没有选择目录，应用会使用 app config 同目录下的 `export-videos`。

## 运行时文件

Release 下，如果用户不手动选择任何目录，应用目录通常会形成：

```text
Keyboard Display 所在目录/
├─ app-config.json
├─ recording-files/
│  └─ *.kbdrec
└─ export-videos/
   └─ *.webm
```

用户导出的 profile JSON 可以在任意位置，由系统保存对话框决定。`app-config.json` 只记录当前 profile 的 `sourcePath`，不维护“最近配置”历史。

## 开发

请使用 `.nvmrc` 声明的 Node 版本：

```sh
nvm use
pnpm install
pnpm tauri dev
```

常用命令：

```sh
pnpm test
pnpm build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

仅启动前端 Vite：

```sh
pnpm dev
```

构建 Tauri 应用：

```sh
pnpm tauri build
```
