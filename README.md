# Keyboard Display

Keyboard Display 是一个面向桌面端的按键显示工具，用来给游戏 POV
视频显示实时按键状态。它的目标是做出类似 CS POV 内容里常见的键盘指示器：
观众能直接看到 `W`、`A`、`S`、`D`、鼠标按键、跳跃、蹲伏等输入状态。

项目优先支持桌面端，不面向 iOS 或 Android。核心能力依赖桌面全局输入捕获、
透明置顶窗口和后续视频导出工作流，这些能力不适合移动端运行模型。

英文说明保留在 [`README.ENGLISH.md`](README.ENGLISH.md)。

## 产品方向

应用应提供：

- 用于游戏画面的透明或紧凑桌面按键 Overlay。
- 来自原生桌面层的全局键盘和鼠标捕获。
- 可配置的键位布局、颜色、字号比例和窗口位置。
- 保存输入数据而不是保存渲染后视频帧的录制格式。
- 基于录制输入时间线的回放和视频导出能力。

## 全局输入捕获

全局输入捕获是核心功能，需要作为跨平台原生层设计。项目不应该依赖单一
macOS 实现，也不应该把平台差异隐藏在不稳定的通用库后面。

目标数据流是：

```text
native input backend -> normalized input event -> overlay renderer / recorder
```

基本要求：

- 至少支持 macOS 和 Windows。
- 键盘和鼠标按钮使用统一的 normalized event 模型。
- 平台后端通过共同的 Rust 接口接入。
- macOS 后端避免在后台 hook 线程里通过 AppKit 或输入源 API 转换键码。
- Windows 后端使用合适的低级键盘和鼠标 hook。
- 把系统权限视为使用前提：macOS 可能需要 Accessibility/Input Monitoring
  权限才能进行真实后台捕获。

在平台后端不可用时，前端键盘事件只能作为预览机制，不能代表真实后台捕获。

## 录制策略

这个项目的关键差异在保存格式。应用不把 Overlay 视频作为主要产物保存，
而是保存紧凑的二进制输入录制文件 `.kbdrec`。

用户可以选择采集帧率，例如 `30fps`、`60fps` 或 `120fps`。Profile 也可以
启用自定义采集帧率，当前上限为 `1000fps`。每个采样帧保存当前输入状态：

- 键盘按键状态 bitset。
- 鼠标按钮状态 bitset。
- 帧序号或时间基准。
- 未来可扩展的鼠标移动、布局快照或其他元数据。

这种方式让原始录制保持很小。比如 64 个按键可以放进 8 字节 bitset，
60fps 录制一分钟的核心按键状态大约只有 28.8KB，还不包含头部和可选元数据。

## 回放与导出

二进制输入录制是 source of truth：

```text
.kbdrec -> replay engine -> overlay renderer -> video encoder
```

同一份录制后续可以用不同布局、颜色、尺寸、透明度或输出格式重新渲染。
视频应被视为导出结果，而不是原始保存格式。

视频导出应优先控制文件体积：

- 可配置输出帧率。
- 可配置分辨率和码率。
- 需要透明通道时使用透明 WebM。
- 更重视兼容性时使用 MP4/H.264。
- 主要录制文件不保存完整视频帧。

## 录制工作流

项目支持两类录制思路，细节见
[`docs/recording-workflows.md`](docs/recording-workflows.md)。

实时 OBS 捕获：

1. 加载或编辑 POV profile。
2. 保持 `Show POV overlay` 开启。
3. 保持 `Silent recording` 关闭。
4. 启动 OBS 或其他屏幕录制工具。
5. 启动 Keyboard Display 录制。
6. 需要对齐点时点击 `Add sync marker` 或使用同步热键。
7. 停止 Keyboard Display 录制并停止屏幕录制。

后期导出：

1. 加载或编辑 POV profile。
2. 选择录制目录，或使用默认 app 录制目录。
3. 如果游戏过程中不希望显示实时 POV 窗口，可以启用 `Silent recording`。
4. 启动 Keyboard Display 录制。
5. 用偏好的工具单独录制游戏画面。
6. 在画面中可识别的时刻添加 sync marker。
7. 停止录制。
8. 检查 `.kbdrec` 文件，使用 marker 元数据对齐未来生成的 Overlay 视频。

## 配置文件

默认 POV profile 结构见
[`docs/default-config.json`](docs/default-config.json)。应用级本地配置结构见
[`docs/app-config.json`](docs/app-config.json)。如果需要手写或分享 profile，
请看 [`docs/config-format.md`](docs/config-format.md)。

布局尺寸使用项目自定义单位：

- `overlay.layout.unitPx` 定义一个布局单位对应的像素大小。
- 每个按键的 `widthUnit` 定义按键宽度。
- `overlay.layout.gapUnit` 定义默认间隔。

单位数量在加载时会规范化到最多两位小数。

## 开发

项目使用 Tauri、Vue、TypeScript 和 Vite。

请使用 `.nvmrc` 声明的 Node 版本：

```sh
nvm use
pnpm install
pnpm tauri dev
```

常用验证命令：

```sh
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```
