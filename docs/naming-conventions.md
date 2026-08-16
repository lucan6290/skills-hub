# 前后端字段命名规范

本文件定义 Skills Hub 前后端交互中的字段/参数命名规则。
**核心原则：跨端通信字段统一 snake_case，前端内部变量使用 camelCase。前后端字段名完全一致，禁止任何转换。**

---

## 一、规则

| 层级 | 命名风格 | 示例 |
|------|---------|------|
| **JSON 线上传输** | `snake_case` | `skill_id`, `source_type`, `created_at` |
| **Python 后端 Pydantic 字段** | `snake_case` | `skill_id: str`, `source_type: str` |
| **TypeScript 前端 DTO 类型** | `snake_case` | `skill_id: string`, `source_type: string` |
| **TypeScript 前端 API 调用参数** | `snake_case` | `{ skill_id: 'x', tag_ids: [1, 2] }` |
| **前端 useState / Props / 函数名** | `camelCase` | `managedSkills`, `searchQuery`, `handleDelete` |
| **前端组件文件名 / Props 类型** | `PascalCase` | `SkillCard.tsx`, `SkillCardProps` |

### 一句话总结

**跨 API 边界的字段全部 snake_case 且前后端一致；前端内部变量和组件使用 camelCase/PascalCase。不允许任何自动转换。**

---

## 二、禁止事项

1. **禁止**使用 `toSnakeCase()` / `toCamelCase()` 等转换函数在前后端之间自动转换字段名
2. **禁止**后端 Pydantic 模型中使用 `Field(alias="camelCaseName")` 来兼容 camelCase 输入
3. **禁止**前后端字段名不一致 —— 后端叫 `skill_id`，前端 DTO 也必须叫 `skill_id`

---

## 三、各层详细规范

### 3.1 JSON 线上传输

所有 HTTP 请求/响应的 JSON body **统一使用 `snake_case`**：

```json
// 正确
{ "skill_id": "abc", "source_type": "community", "created_at": 1720000000 }

// 错误
{ "skillId": "abc", "sourceType": "community", "createdAt": 1720000000 }
```

命名约定：
- 多单词字段用下划线连接，全小写
- 布尔字段用 `is_` / `has_` 前缀：`is_link`, `has_conflict`
- ID 字段用 `_id` 后缀：`skill_id`, `tag_id`
- 时间戳字段用 `_at` 后缀：`created_at`, `updated_at`, `synced_at`
- 计数/数量字段用 `_count` 后缀：`skill_count`, `view_count`, `sync_count`

### 3.2 Python 后端（Pydantic 模型）

**字段名使用 snake_case**，与 JSON 完全一致，**不需要 alias**：

```python
class ManagedSkillDto(BaseModel):
    id: str
    name: str
    source_type: str
    community_path: str
    created_at: int
    updated_at: int
    sort_order: float = 0.0
```

请求模型同样直接使用 snake_case：

```python
class SetSkillTagsRequest(BaseModel):
    skill_id: str
    tag_ids: list[int]
```

### 3.3 TypeScript 前端（DTO 类型）

**DTO 类型（与后端交互的数据结构）字段名使用 snake_case**：

```typescript
// frontend/src/components/skills/types.ts

export type ManagedSkill = {
  id: string
  name: string
  source_type: string
  community_path: string
  created_at: number
  updated_at: number
  sort_order: number
  tags: TagDto[]
  targets: {
    tool: string
    scope: string
    project_path?: string | null
    target_path: string
    synced_at?: number | null
  }[]
}
```

### 3.4 TypeScript 前端（API 调用参数）

**调用 `apiCall()` / `apiGet()` 时参数使用 snake_case**：

```typescript
// 正确
await apiCall('set_skill_tags', { skill_id: 'abc', tag_ids: [1, 2] })
await apiCall('sync_skill_to_tool', {
  skill_id: 'abc',
  tool: 'claude',
  source_path: '/path/to/skill',
})

// 错误
await apiCall('set_skill_tags', { skillId: 'abc', tagIds: [1, 2] })
```

### 3.5 TypeScript 前端（内部状态与 Props）

**组件内部 useState、组件 Props、函数名使用 camelCase**：

```typescript
// 正确：内部状态用 camelCase
const [managedSkills, setManagedSkills] = useState<ManagedSkill[]>([])
const [searchQuery, setSearchQuery] = useState('')

// Props 用 camelCase
type SkillCardProps = {
  skill: ManagedSkill
  sortBy: string
  onDelete: (skillId: string) => void
}
```

---

## 四、常用字段速查

| 字段含义 | 字段名（跨端统一） |
|---------|-------------------|
| 技能 ID | `skill_id` |
| 标签 ID | `tag_id` |
| 标签 ID 列表 | `tag_ids` |
| 来源类型 | `source_type` |
| 来源路径 | `source_path` |
| 来源引用 | `source_ref` |
| 来源子路径 | `source_subpath` |
| 来源 URL | `source_url` |
| 项目路径 | `project_path` |
| 目标路径 | `target_path` |
| 社区路径 | `community_path` |
| 内容哈希 | `content_hash` |
| 创建时间 | `created_at` |
| 更新时间 | `updated_at` |
| 最后同步时间 | `last_sync_at` |
| 同步时间 | `synced_at` |
| 最后查看时间 | `last_viewed_at` |
| 技能数量 | `skill_count` |
| 查看次数 | `view_count` |
| 同步次数 | `sync_count` |
| 排序序号 | `sort_order` |
| 是否为链接 | `is_link` |
| 是否有冲突 | `has_conflict` |
| 是否支持项目范围 | `supports_project_scope` |
| 试运行 | `dry_run` |
| 覆盖写入 | `overwrite` |
| 模式 | `mode` |
| 范围 | `scope` |
| 状态 | `status` |
| 工具 | `tool` |
| 名称 | `name` |

---

## 五、新增字段流程

当需要添加新的前后端交互字段时：

1. **[后端 schemas.py]** 用 snake_case 定义 Pydantic 字段
2. **[前端 types.ts]** 用 snake_case 添加对应的 DTO 类型字段
3. **[前端调用处]** 用 snake_case 传参
4. **[前端组件内部]** 如需将该字段存入 useState 或作为 Props 传递，使用 camelCase 命名的中间变量

**前三步的字段名必须一模一样。**

---

## 六、常见错误

| 错误 | 正确 |
|------|------|
| DTO 类型写 `sourceType: string` | DTO 类型写 `source_type: string` |
| API 参数传 `{ skillId: 'x' }` | API 参数传 `{ skill_id: 'x' }` |
| 使用转换函数自动改字段名 | 直接写 snake_case，不做转换 |
| 后端写 `Field(alias="skillId")` | 直接用 snake_case 字段名，无需 alias |
| JSON 响应返回 `{ "skillId": 1 }` | JSON 响应返回 `{ "skill_id": 1 }` |
| 前端组件 Props 用 snake_case | 组件 Props 用 camelCase（DTO 字段除外） |
