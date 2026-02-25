<p align="center">
  <a href="./README.md">English</a> · <a href="./README.zh-CN.md">简体中文</a>
</p>


<p align="center">
  <img src="./docs/logo.jpg" alt="Xybrid Logo" width="180"/>
</p>

<h1 align="center">Xybrid</h1>

<p align="center">
  <strong>移动端、桌面端、边缘端的设备端 AI。</strong><br/>
  在本地运行语音、语言和视觉模型——隐私、离线、快速。<br/>
  适用于任何应用，包括游戏 🕹️
</p>

<p align="center">
  <a href="https://docs.xybrid.dev">文档</a> ·
  <a href="#sdk">SDK</a> ·
  <a href="https://www.xybrid.ai/models">模型</a> ·
  <a href="https://discord.gg/YhFHHkhbad">加入 Discord</a> ·
  <a href="https://github.com/xybrid-ai/xybrid/issues">问题反馈</a>
</p>

<p align="center">

[![Website][website-shield]][website-url]
[![Discord][discord-shield]][discord-url]
[![Twitter][twitter-shield]][twitter-url]
<br>
[![License][license-shield]][license-url]
[![Build][build-shield]][build-url]
[![Stars][stars-shield]][stars-url]
[![Release][release-shield]][release-url]
[![Release Date][release-date-shield]][release-url]
<br>
[![pub.dev][pubdev-shield]][pubdev-url]
[![Maven Central][maven-shield]][maven-url]

</p>

[website-shield]: https://img.shields.io/badge/xybrid.ai-4285F4?style=flat
[website-url]: https://www.xybrid.ai/
[discord-shield]: https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fdiscord.com%2Fapi%2Finvites%2FYhFHHkhbad%3Fwith_counts%3Dtrue&query=%24.approximate_member_count&logo=discord&logoColor=white&label=Discord&color=5865F2&suffix=%20members
[discord-url]: https://discord.gg/YhFHHkhbad
[twitter-shield]: https://img.shields.io/twitter/follow/xybrid_ai
[twitter-url]: https://x.com/xybrid_ai
[license-shield]: https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=flat
[license-url]: https://opensource.org/licenses/Apache-2.0
[build-shield]: https://img.shields.io/github/actions/workflow/status/xybrid-ai/xybrid/ci.yml?branch=master&style=flat
[build-url]: https://github.com/xybrid-ai/xybrid/actions
[stars-shield]: https://img.shields.io/github/stars/xybrid-ai/xybrid?style=flat
[stars-url]: https://github.com/xybrid-ai/xybrid/stargazers
[release-shield]: https://img.shields.io/github/v/release/xybrid-ai/xybrid?style=flat&sort=semver
[release-url]: https://github.com/xybrid-ai/xybrid/releases
[release-date-shield]: https://img.shields.io/github/release-date/xybrid-ai/xybrid?style=flat
[pubdev-shield]: https://img.shields.io/pub/v/xybrid_flutter?style=flat&label=pub.dev
[pubdev-url]: https://pub.dev/packages/xybrid_flutter
[maven-shield]: https://img.shields.io/maven-central/v/ai.xybrid/xybrid-kotlin?style=flat&label=Maven%20Central
[maven-url]: https://central.sonatype.com/artifact/ai.xybrid/xybrid-kotlin

---

<p align="center">
  <img src="docs/demo-desktop.gif" alt="Desktop demo" width="540">&nbsp;&nbsp;&nbsp;&nbsp;
  <img src="docs/demo-android.gif" alt="Android demo" width="150">
</p>

## SDK

Xybrid 是一个 **Rust 驱动的运行时**，为所有主流平台提供原生绑定。选择你的 SDK：

| SDK | 平台 | 安装 | 状态 | 示例 |
|-----|------|------|------|------|
| **[Flutter](bindings/flutter/)** | iOS, Android, macOS, Linux, Windows | [pub.dev](https://pub.dev/packages/xybrid_flutter) | 可用 | [README](examples/flutter/README.md) |
| **[Unity](bindings/unity/)** | macOS, Windows, Linux | [见下方](#安装) | 可用 | [Unity 3D AI 酒馆](https://github.com/xybrid-ai/xybrid-unity-tavern) |
| **[Swift](bindings/apple/)** | iOS, macOS | Swift Package Manager | 即将推出 | [README](examples/ios/README.md) |
| **[Kotlin](bindings/kotlin/)** | Android | Maven Central | 可用 | [README](examples/android/README.md) |
| **[CLI](https://github.com/xybrid-ai/xybrid/releases)** | macOS, Linux, Windows | [下载二进制文件](https://github.com/xybrid-ai/xybrid/releases) | 可用 | — |
| **[Rust](crates/)** | 全平台 | `xybrid-core` / `xybrid-sdk` | 可用 | — |

所有 SDK 封装同一个 Rust 核心——跨平台行为和模型支持完全一致。

### 安装

**Unity** — Package Manager → 通过 git URL 添加：

```unity
https://github.com/xybrid-ai/xybrid.git?path=bindings/unity
```

**Flutter** — 添加到你的 `pubspec.yaml`：

```yaml
dependencies:
  xybrid_flutter: ^0.1.0
```

**Kotlin (Android)** — 添加到你的 `build.gradle.kts`：

```gradle
dependencies {
    implementation("ai.xybrid:xybrid-kotlin:0.1.0-alpha7")
}
```

---

## 快速开始

各平台的详细设置请参阅对应 SDK 的 README：[Flutter](bindings/flutter/) · [Unity](bindings/unity/) · [Swift](bindings/apple/) · [Kotlin](bindings/kotlin/) · [Rust](crates/)

### 单模型

通过 CLI 一行命令运行模型，或在任何 SDK 中三行代码搞定：

**CLI:**
```sh
xybrid run kokoro-82m --input "国破山河在，城春草木深" -o output.wav
```

**Flutter:**
```dart
final model = await Xybrid.model(modelId: 'kokoro-82m').load();
final result = await model.run(envelope: Envelope.text(text: '国破山河在，城春草木深'));
// result → 24kHz WAV 音频
```

**Kotlin:**
```kotlin
val model = Xybrid.model(modelId = "kokoro-82m").load()
val result = model.run(envelope = XybridEnvelope.Text("国破山河在，城春草木深"))
// result → 24kHz WAV 音频
```

**Swift:**
```swift
let model = try Xybrid.model(modelId: "kokoro-82m").load()
let result = try model.run(envelope: .text("国破山河在，城春草木深"))
// result → 24kHz WAV 音频
```

**Unity (C#):**
```csharp
var model = Xybrid.Model(modelId: "kokoro-82m").Load();
var result = model.Run(envelope: Envelope.Text("国破山河在，城春草木深"));
// result → 24kHz WAV 音频
```

**Rust:**
```rust
let model = Xybrid::model("kokoro-82m").load()?;
let result = model.run(&Envelope::text("国破山河在，城春草木深"))?;
// result → 24kHz WAV 音频
```

### 管道

将模型链接在一起——用 3 行 YAML 构建语音助手：

```yaml
# voice-assistant.yaml
name: voice-assistant
stages:
  - model: whisper-tiny    # 语音 → 文本
  - model: qwen2.5-0.5b    # 用 LLM 处理
  - model: kokoro-82m      # 文本 → 语音
```

**CLI:**
```sh
xybrid run voice-assistant.yaml --input question.wav -o response.wav
```

**Flutter:**
```dart
final pipeline = await Xybrid.pipeline(yamlContent: yamlString).load();
await pipeline.loadModels();
final result = await pipeline.run(envelope: Envelope.audio(bytes: audioBytes));
```

**Kotlin:**
```kotlin
val pipeline = Xybrid.pipeline(yamlContent = yamlString).load()
pipeline.loadModels()
val result = pipeline.run(envelope = XybridEnvelope.Audio(bytes = audioBytes))
```

**Swift:**
```swift
let pipeline = try Xybrid.pipeline(yamlContent: yamlString).load()
try pipeline.loadModels()
let result = try pipeline.run(envelope: .audio(bytes: audioBytes))
```

**Unity (C#):**
```csharp
var pipeline = Xybrid.Pipeline(yamlContent: yamlString).Load();
pipeline.LoadModels();
var result = pipeline.Run(envelope: Envelope.Audio(bytes: audioBytes));
```

**Rust:**
```rust
let pipeline = Xybrid::pipeline(&yaml_string).load()?;
pipeline.load_models()?;
let result = pipeline.run(&Envelope::audio(audio_bytes))?;
```
---

## 支持的模型

所有模型完全在设备端运行。无需云端，无需 API 密钥。使用 `xybrid models list` 浏览完整的模型注册表。

### 语音转文本

| 模型 | 参数量 | 格式 | 描述 |
|------|--------|------|------|
| Whisper Tiny | 39M | SafeTensors | 多语言转录（Candle 运行时） |
| Wav2Vec2 Base | 95M | ONNX | 英语 ASR，CTC 解码 |

### 文本转语音

| 模型 | 参数量 | 格式 | 描述 |
|------|--------|------|------|
| Kokoro 82M | 82M | ONNX | 高质量，24 种自然声音 |
| KittenTTS Nano | 15M | ONNX | 超轻量级，8 种声音 |

### 语言模型

| 模型 | 参数量 | 格式 | 描述 |
|------|--------|------|------|
| Gemma 3 1B | 1B | GGUF Q4_K_M | Google 移动端优化 LLM |
| Llama 3.2 1B | 1B | GGUF Q4_K_M | Meta 通用模型，128K 上下文 |
| Qwen 2.5 0.5B | 500M | GGUF Q4_K_M | 紧凑型设备端对话模型 |
| SmolLM2 360M | 360M | GGUF Q4_K_M | 最佳微型 LLM，优秀的质量/体积比 |

### 即将推出

| 模型 | 类型 | 参数量 | 优先级 | 状态 |
|------|------|--------|--------|------|
| Phi-4 Mini | LLM | 3.8B | P2 | 规格就绪（首个多量化：Q4, Q8, FP16） |
| Qwen3 0.6B | LLM | 600M | P2 | 计划中 |
| Trinity Nano | LLM (MoE) | 6B（1B 活跃） | P2 | 计划中 |
| LFM2 700M | LLM | 700M | P2 | 计划中 |
| Nomic Embed Text v1.5 | 嵌入 | 137M | P1 | 阻塞中（需要 Tokenize/MeanPool 步骤） |
| LFM2-VL 450M | 视觉 | 450M | P2 | 计划中 |
| Whisper Tiny CoreML | ASR | 39M | P2 | 计划中 |
| Qwen3-TTS 0.6B | TTS | 600M | P2 | 阻塞中（需要自定义 SafeTensors 运行时） |
| Chatterbox Turbo | TTS | 350M | P3 | 阻塞中（需要 ModelGraph 模板） |

---

## 功能特性

| 能力 | iOS | Android | macOS | Linux | Windows |
|------|-----|---------|-------|-------|---------|
| 语音转文本 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 文本转语音 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 语言模型 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 视觉模型 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 嵌入模型 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 管道编排 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 模型下载与缓存 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 硬件加速 | Metal, ANE | CPU | Metal, ANE | CUDA | CUDA |

---

## 为什么选择 Xybrid？

- **隐私优先** — 所有推理在设备端运行。你的数据永远不会离开设备。
- **离线可用** — 初次模型下载后无需互联网。
- **跨平台** — iOS、Android、macOS、Linux 和 Windows 使用统一 API。
- **管道编排** — 在单次调用中链接多个模型（ASR → LLM → TTS）。
- **自动优化** — 在 Apple Neural Engine、Metal 和 CUDA 上进行硬件加速。

---

## 社区

- [文档](https://docs.xybrid.dev)
- [Discord](https://discord.gg/YhFHHkhbad)
- [GitHub Issues](https://github.com/xybrid-ai/xybrid/issues)

## 贡献

我们欢迎贡献！请参阅 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解开发环境设置、提交 Pull Request 和添加新模型的指南。

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=xybrid-ai/xybrid&type=date&legend=bottom-right)](https://www.star-history.com/#xybrid-ai/xybrid&type=date&legend=bottom-right)

## 许可证

Apache License 2.0 — 详见 [LICENSE](./LICENSE)。
