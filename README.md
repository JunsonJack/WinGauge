# WinGauge

**Windows 托盘悬浮系统监控面板** —— 点一下托盘，CPU / 内存 / 网络 / 磁盘与健康度总分一眼看完。

> 工作名 WinGauge · **v0.1.0** · Windows 10/11 · 不需要管理员权限  
> 参考信息架构：macOS 菜单栏弹窗式监控（CatStatus 形态），落地平台为 Windows。

**v0.1 已可安装使用**：NSIS / MSI 安装包见下方「快速开始」。

---

## 为什么做

Windows 上「看电脑现在怎么样」这件事，现有工具都有断点：

| 现有方案 | 问题 |
|---|---|
| 任务管理器 | 要 Alt+Tab、等加载、找选项卡；不是「瞄一眼」的形态 |
| TrafficMonitor / RainMeter | 功能在，但「好看 + 零配置 + 不打扰」不在 |
| HWiNFO / LibreHardwareMonitor | 传感器全览，没有「现在到底好不好」的结论 |

**WinGauge 的差异**：不做传感器大全，做**结论式监控**——每个卡片给「数值 + 一句话判断 + 颜色状态」，顶部再给一个健康度总分。

---

## 功能一览（v0.1）

| 能力 | 状态 | 说明 |
|---|---|---|
| 托盘常驻 | ✅ | 左键弹出面板，右键系统菜单（暂停采样 / 设置 / 退出） |
| 悬浮弹窗 | ✅ | 无边框、透明、置顶、不占任务栏；出现在托盘图标正上方 |
| 失焦收起 | ✅ | 150ms 延迟，避免与「点托盘」竞争闪烁 |
| 钉住 + 拖动 | ✅ | 钉住后失焦不收起、可拖动，重启记住位置 |
| 设备头 | ✅ | 主机名 / 系统版本 / CPU / 内存总量 / 运行时长 |
| 健康度总分 | ✅ | 100 起扣，可展开扣分明细 |
| CPU 卡片 | ✅ | 总使用率、峰值、每核柱状图、负载档位、60s 趋势 |
| 内存卡片 | ✅ | 已用/总量、已提交内存（Windows 语义，不出现「交换」歧义）、趋势 |
| 网络卡片 | ✅ | 主接口下行/上行速率 + 接口友好名、趋势 |
| 磁盘卡片 | ✅ | 系统盘用量、剩余空间 |
| 设置页 | ✅ | 开机自启、采样说明、隐私说明、退出 |
| 开机自启 | ✅ | HKCU Run，默认关闭，可被任务管理器统一管理 |
| 单实例锁 | ✅ | 二次启动会弹出已有实例的面板 |
| NSIS / MSI 安装包 | ✅ | `npm run tauri:build` |
| 厂商温度/风扇 | ⏳ W4b | 联想 Legion EC 通路实测可读，待接入 |
| 历史统计 | ⏳ W5 | SQLite 归档 + 日/周/月视图 |
| 输入量统计 | ⏳ W6 | 默认关闭；只计数不记内容 |

**明确不做**：进程列表/结束进程、超频/风扇写操作、键鼠内容记录、皮肤商店、云同步/账号/遥测。

---

## 快速开始

### 环境要求

- Windows 10 / 11
- [Rust](https://rustup.rs/)（stable ≥ 1.77）
- [Node.js](https://nodejs.org/) ≥ 20
- WebView2 Runtime（Windows 11 自带）

### 安装（推荐）

从本机构建产物安装，或从 GitHub Releases 下载：

```text
target\release\bundle\nsis\WinGauge_0.1.0_x64-setup.exe   # NSIS（推荐）
target\release\bundle\msi\WinGauge_0.1.0_x64_en-US.msi    # MSI
```

安装后在开始菜单或托盘找到 WinGauge。**首次启动托盘图标可能在溢出区**，可固定到任务栏托盘。

### 开发

```powershell
# 一键（会构建前端并拉起 Tauri）
npm install   # 可选，根目录仅脚本
npm run tauri:dev

# 或分步
cd frontend; npm install; npm run build; cd ..
npx --yes @tauri-apps/cli dev
```

### 打包

```powershell
npm run tauri:build
# 产物在 src-tauri 无关，实际在：
#   target/release/bundle/nsis/WinGauge_0.1.0_x64-setup.exe
#   target/release/bundle/msi/WinGauge_0.1.0_x64_en-US.msi
```

### 测试与冒烟

```powershell
npm test    # cargo test -p wingauge-core
npm run smoke
```

`smoke_tick` 期望输出类似：

```text
device  : host=… os=Windows 11 (26200) cores=16 mem=39.8GB uptime=…
cpu     : 34.4% peak=… cores=16
memory  : 82.1% used=… committed=…
network : 以太网 3 down=…/s up=…/s
disk    : C:\ used=85.9% …
health  : 100 (Excellent) — 各项指标正常
SMOKE OK
```

---

## 架构

```
┌────────────────────────────────────────────────┐
│         悬浮弹窗（Vue 3，无边框透明置顶）          │
│  设备头 │ 健康度 │ CPU │ 内存 │ 网络 │ 磁盘        │
└──────────────────┬─────────────────────────────┘
                   │ Tauri IPC：snapshot://tick 推送
┌──────────────────┴─────────────────────────────┐
│  src-tauri（托盘 + 弹窗生命周期 + 采样线程）        │
│  tray / panel / autostart / sampler             │
├────────────────────────────────────────────────┤
│  wingauge-core（框架无关，可被 CLI 复用）          │
│  collector  · 1s 采样，sysinfo 实时通路            │
│  validity   · 假数据拦截（负温度、flatline…）       │
│  score      · 健康度扣分制                         │
│  provider   · 能力探测（Available/Degraded/…）     │
│  snapshot   · 前后端共用的指标模型                  │
└────────────────────────────────────────────────┘
```

### 关键设计决策

1. **推送而非拉取**  
   Rust 侧 1s 定时采样，`emit("snapshot://tick")` 批量推给前端。前端绝不做轮询 IPC。

2. **读不到就整卡消失**  
   每个 provider 声明 `Available / Degraded / Unavailable`。不可用时 UI 隐藏卡片，**不显示 0、不显示 N/A**。

3. **合理性校验**（W0 实测教训）  
   探测通过 ≠ 数据可信。`sysinfo` 温度在本机返回 `-0.15°C` 且 `max == temp`。规则：超出物理范围、`max == temp`、长时间 flatline 而负载波动 → 判无效。

4. **温度只走厂商通路**  
   通用 ACPI 温度通路是假数据。联想 `LENOVO_FAN_METHOD` 实测可读真实 RPM/温度，接入前温度卡片一律不渲染。

5. **网络只认接口身份，不认名字**  
   本机实测 11 个适配器（VMware / Hyper-V / OpenVPN / 蓝牙…）。v0.1 用启发式黑名单选主接口；后续用 IP Helper `ifIndex` 作为唯一可靠键。

6. **「交换」文案禁用**  
   Windows 上提交内存与 pagefile 实际占用可差一个数量级（实测 10.2GB vs 295MB）。UI 统一写 **已提交 X / Y GB**。

7. **性能预算**  
   平均 CPU < 1.5%、常驻内存 < 150MB。弹窗隐藏时降到 2s 一帧，但**不停止采样**（重开瞬间有数据）。

---

## 目录结构

```
WinGauge/
├── Cargo.toml                 # workspace
├── wingauge-core/             # 采集 / 校验 / 评分（无 Tauri 依赖）
│   ├── src/
│   │   ├── collector.rs       # sysinfo 实时采集
│   │   ├── validity.rs        # 合理性校验
│   │   ├── score.rs           # 健康度模型
│   │   ├── snapshot.rs        # 指标模型
│   │   └── provider.rs        # 能力抽象
│   └── examples/smoke_tick.rs
├── src-tauri/                 # 桌面壳
│   └── src/  lib.rs panel.rs tray.rs autostart.rs sampler.rs
├── frontend/                  # Vue 3 + Vite + TS
│   └── src/  App.vue components/ lib/metrics.ts
├── docs/                      # W0 实测笔记 / W1 完成报告
└── probe/                     # Windows 采集能力探测脚本
```

---

## 健康度模型

从 100 起扣，可点开看原因：

| 指标 | 条件 | 扣分 |
|---|---|---|
| CPU 使用率 | 持续 60s > 90% | −15 |
| 内存 | > 85% / > 95% | −20 / −30 |
| 已提交内存 | > 90% 提交上限 | −15 |
| 系统盘剩余 | < 10% | −20 |

档位：≥90 很好 · 70–89 良好 · 50–69 需注意 · &lt;50 异常。

---

## 里程碑

| 阶段 | 内容 | 状态 |
|---|---|---|
| **W0** 调研定标 | 本机四轮采集通路实测 | ✅ |
| **W1** 壳与形态 | 托盘 / 定位 / 钉住 / 自启 / 单实例 | ✅ |
| **W2** 采集层 | collector + validity + score + IPC | ✅ |
| **W3** UI | 设备头 / 健康度 / 四卡 / 迷你趋势 / 设置页 | ✅ v0.1.0 |
| **W4** 打包 | NSIS + MSI | ✅ v0.1.0 |
| **W4b** 厂商传感器 | 联想温度/风扇 + ID 扫描落盘 | ⏳ |
| **W5** 历史统计 | SQLite 归档 + 日/周/月 | ⏳ |
| **W6** 输入统计 | 默认关闭的低级钩子计数 | ⏳ |

---

## 隐私

- 本机自用工具，**无云同步、无账号、无遥测上报**。
- 不记录键鼠内容；未来输入统计默认关闭，只维护计数器。
- 不枚举进程、不结束进程、不写任何硬件寄存器。

---

## 开发笔记

- **W0 实测**：`docs/W0-本机采集实测.md` —— 通用温度通路假数据、联想 EC 可读、网络 ifIndex 结论。
- **W1 完成**：`docs/W1-壳与形态.md` —— 托盘定位、钉住拖动的坑（`dragstart` 撞 DOM 保留字等）。
- **v0.1 完成报告**：`docs/W2-W3-v0.1.0.md`
- 单测不依赖真机：`cargo test -p wingauge-core`。

### 已知限制（v0.1）

- 网络主接口用启发式黑名单选择，未接 IP Helper ifIndex（虚拟网卡流量很大时可能选错）。
- 温度 / 风扇卡片未渲染（通用通路是假数据，厂商 provider 在 W4b）。
- 历史曲线仅前端 60s 内存窗口，无 SQLite 归档。
- bundle identifier 为 `com.wingauge.app`（仅 Windows 无影响，发布前建议改 `.desktop`）。
- 未在开启 UAC 的普通权限机器上完成复测（W0 R10）。

---

## License

待定（私人项目，发布前再定开源协议）。
