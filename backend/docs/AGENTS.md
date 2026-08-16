# 后端文档目录 Agent 规则

> **定位**：`backend/docs/` 目录的模块级规则文件，继承 `backend/AGENTS.md` 和根 `AGENTS.md` 的所有约束。
> **关系**：本文件仅补充文档目录特有的编辑规范；通用编码规范见上级 AGENTS.md，各专题规范见同目录下对应文件。

适用于 `backend/docs/` 下的所有文档修改。

## 修改前

1. 先读本文件了解文档组织规则。
2. 确认该信息的权威位置，避免创建第二份同义规范。
3. 涉及实现事实时先核对当前代码；不要从旧文档反推现状。

## 修改时

- 保留"入口短、专题清晰、按需加载"的渐进式披露结构。
- 规则使用"必须/禁止/可以"表达，说明适用条件和例外。
- 不维护易漂移的绝对行号；优先引用文件和符号名。
- 示例不能包含真实凭证或敏感值。
- 新增、移动或删除文档时同步更新 `backend/AGENTS.md` 中的导航链接。

## 完成证据

- Markdown 相对链接均能解析
- 入口到新文档存在可达路径
- 没有引入与上级 AGENTS.md 冲突的规则

## 文档清单

| 文件 | 内容 |
|------|------|
| [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md) | 后端系统与目录地图 |
| [API_STANDARD.md](./API_STANDARD.md) | API 设计规范 |
| [DATABASE_STANDARD.md](./DATABASE_STANDARD.md) | 数据库规范 |
| [TESTING_STANDARD.md](./TESTING_STANDARD.md) | 测试与安全规范 |
