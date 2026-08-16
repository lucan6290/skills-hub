# 主题与样式规范

> 本文件描述前端 CSS 变量系统、主题切换机制和浅色/暗色模式开发策略。修改样式后同步更新。

## 1. 核心原则

**浅色模式为主要开发模式，暗色模式仅做兼容支持。**

## 2. 开发优先级

- ✅ **优先开发浅色模式**：所有新功能、新组件首先在浅色模式下完成开发和测试
- ✅ **浅色模式精细化优化**：确保浅色模式的视觉效果、交互体验完美
- ⚠️ **暗色模式基础兼容**：仅保证暗色模式下基础功能可用，不做精细化优化

## 3. 样式文件位置

| 文件 | 职责 |
|------|------|
| `src/index.css` | CSS 变量定义（`:root` 浅色 + `:root[data-theme='dark']` 暗色）+ Tailwind 导入 + 全局 reset |
| `src/App.css` | 所有组件样式（6800+ 行，集中管理） |

**禁止**在组件目录下创建独立 CSS 文件。所有组件样式统一写入 `App.css`。

## 4. 开发流程

1. 新增组件/功能时，首先编写浅色模式样式（使用 `:root` 中的 CSS 变量）
2. 确保浅色模式下所有交互、状态正常
3. 最后添加暗色模式兼容（如需要新增 CSS 变量，在 `:root[data-theme='dark']` 中添加对应覆盖值）
4. 暗色模式只需保证可读性和基础功能

## 5. CSS 变量系统

### 5.1 浅色模式（`:root`，默认）

| 类别 | 变量名 | 值 |
|------|--------|-----|
| **背景** | `--bg-app` | `#f6f4ee` |
| | `--bg-panel` | `#fffefa` |
| | `--bg-element` | `#ece8dc` |
| | `--bg-element-hover` | `#e3ded0` |
| | `--bg-header` | `rgba(255, 254, 250, 0.82)` |
| | `--bg-badge` | `#eee9db` |
| | `--bg-hover` | `rgba(31, 42, 55, 0.05)` |
| **边框** | `--border-subtle` | `#ddd6c7` |
| | `--border-strong` | `#bfb39f` |
| | `--border-faint` | `#ebe5d8` |
| | `--border-hover` | `#bfb39f` |
| **文字** | `--text-primary` | `#1f2933` |
| | `--text-secondary` | `#58616d` |
| | `--text-tertiary` | `#8b8174` |
| **强调色** | `--accent-primary` | `#1d7180` |
| | `--accent-primary-hover` | `#155d69` |
| | `--accent-primary-fg` | `#ffffff` |
| | `--accent-soft-bg` | `#e5f2f1` |
| | `--accent-soft-border` | `#a8cecc` |
| **状态色** | `--status-success` | `#26734d` |
| | `--status-warning` | `#b26119` |
| | `--status-error` | `#c2413d` |
| | `--status-info` | `#1d7180` |
| **软背景** | `--success-soft-bg` | `#e9f7ef` |
| | `--success-soft-border` | `#add8bd` |
| | `--warning-soft-bg` | `#fff4d8` |
| | `--warning-soft-border` | `#dfb35f` |
| | `--danger-soft-bg` | `#fff0ed` |
| | `--danger-soft-border` | `#efc0b7` |
| | `--danger-soft-bg-strong` | `#ffe7e2` |
| **字体** | `--font-ui` | `"IBM Plex Sans", "Microsoft YaHei UI", "PingFang SC", system-ui, ...` |
| | `--font-mono` | `"IBM Plex Mono", ui-monospace, SFMono-Regular, ...` |
| **圆角** | `--radius-sm` / `--radius-md` / `--radius-lg` | `4px` / `8px` / `8px` |
| **阴影** | `--shadow-sm` | `0 1px 2px 0 rgb(36 28 15 / 0.08)` |
| | `--shadow-lg` | `0 18px 40px -24px rgb(36 28 15 / 0.45)` |

### 5.2 暗色模式（`:root[data-theme='dark']`，覆盖）

暗色模式覆盖上述所有变量。关键差异：

- 背景使用深灰色而非纯黑（`#151715` / `#1d201d`）
- 文字使用暖白色（`#f5f1e7`）
- 强调色偏亮（`#67c4bf`），前景色为深色（`#101513`）
- 阴影更深（`rgb(0 0 0 / 0.38)` / `rgb(0 0 0 / 0.72)`）
- 额外设置 `color-scheme: dark`

## 6. 主题切换机制

### 6.1 useTheme Hook（`src/hooks/useTheme.ts`）

三层实现：

1. **用户偏好**（`themePreference`）：`'system'` | `'light'` | `'dark'`，持久化到 `localStorage`（key: `skills-theme`）
2. **系统主题检测**：监听 `window.matchMedia('(prefers-color-scheme: dark)')`
3. **应用主题**：计算 `resolvedTheme`，设置 `document.documentElement.dataset.theme = resolvedTheme`

### 6.2 CSS 生效机制

- `:root`（浅色）是默认主题，始终生效
- `:root[data-theme='dark']` 覆盖浅色变量，仅当 `document.documentElement.dataset.theme === 'dark'` 时生效
- 用户在 `SettingsPage` 选择主题偏好 → `handleThemeChange(preference)` → 更新状态 + localStorage → 触发 DOM 更新 → CSS 变量切换

## 7. 代码规范

### 7.1 硬性规则

- **禁止硬编码颜色值**：所有颜色必须通过 `var(--xxx)` 引用 CSS 变量
- **禁止内联样式颜色**：`style={{ color: '#xxx' }}` 不允许，应使用 CSS 类 + 变量
- **新增 CSS 变量必须同时定义浅色和暗色值**：在 `:root` 中定义默认值，在 `:root[data-theme='dark']` 中添加覆盖值
- **CSS 类名使用 kebab-case**：`.skill-card`、`.modal-backdrop`、`.filter-bar`

### 7.2 响应式断点

| 断点 | 适用场景 |
|------|---------|
| `@media (max-width: 920px)` | 平板/窄屏 |
| `@media (max-width: 720px)` | 手机横屏 |
| `@media (max-width: 520px)` | 手机竖屏 |

### 7.3 全局背景

`body` 使用网格背景叠加：

```css
background:
  linear-gradient(90deg, rgba(31, 41, 51, 0.035) 1px, transparent 1px),
  linear-gradient(rgba(31, 41, 51, 0.028) 1px, transparent 1px),
  var(--bg-app);
background-size: 36px 36px, 36px 36px, auto;
```

## 8. 暗色模式兼容要求

- 保证文字可读（对比度达标）
- 背景不刺眼（使用深灰色而非纯黑）
- 基础交互状态可见（hover、active、focus）
- 无需追求与浅色模式完全一致的视觉效果

## 9. 检查清单

开发完成后确认：

- [ ] 浅色模式下所有状态正常显示
- [ ] 浅色模式下交互反馈清晰可见
- [ ] 所有颜色使用 CSS 变量，无硬编码颜色值
- [ ] 新增 CSS 变量在浅色和暗色中都有定义
- [ ] 暗色模式下文字可读、功能可用
- [ ] 新增样式未破坏现有主题机制
