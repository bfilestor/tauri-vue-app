# Git 分支规范

本项目当前采用单人开发、双长期分支模式，不开启分支保护，日常直接在长期分支上开发。

适用范围：
- 开发者只有 1 人
- 当前需要同时维护无用户版和多用户版
- 日常不走 Pull Request 流程

## 1. 分支职责

当前长期分支如下：
- `no-user`：无用户体系版本
- `multi-user`：多用户体系版本

历史分支说明：
- `master`：历史默认分支，不再作为日常开发主线
- `type001`：历史开发分支，保留用于追溯，后续不再作为主开发分支

规则：
- 无用户相关需求只在 `no-user` 开发
- 多用户相关需求只在 `multi-user` 开发
- 不在 `master` 上直接开发
- 不把 `no-user` 整体合并到 `multi-user`
- 不把 `multi-user` 整体合并回 `no-user`

说明：
- 两条长期分支后续会逐渐产生结构差异，整体互相 merge 很容易把不属于当前版本的能力带过去。
- 两边都需要的修复，使用 `cherry-pick` 同步提交，不使用整分支合并。

## 2. 日常开发规则

本项目允许直接在长期分支上开发，但必须遵守以下纪律：

- 开发前先确认自己所在分支
- 每次开始工作前先执行一次 `git pull`
- 一个需求尽量拆成多个小提交，不要堆成一个超大提交
- 需求做到一半如果要切换分支，先提交或 `stash`
- 不在一个分支里顺手改另一个版本专属逻辑
- 不使用 `git push --force`
- 不使用 `git reset --hard` 清理未确认的修改

建议工作节奏：

```bash
git switch no-user
git pull
```

或

```bash
git switch multi-user
git pull
```

开发完成后：

```bash
git add .
git commit -m "feat: xxx"
git push
```

## 3. 什么时候允许新建临时分支

虽然平时直接在长期分支开发，但以下场景建议临时切分支：

- 大范围重构
- 涉及数据库结构调整
- 涉及 Tauri Rust 端与前端联动的大改动
- 预计超过 1 天才能完成的需求
- 你不确定最终方案是否稳定

推荐命名：
- `temp/no-user-xxx`
- `temp/multi-user-xxx`

示例：

```bash
git switch no-user
git pull
git switch -c temp/no-user-refactor-ocr
```

完成后再合回对应长期分支：

```bash
git switch no-user
git merge --ff-only temp/no-user-refactor-ocr
git push
```

如果不能 fast-forward，说明长期分支期间已有其他提交，需要先整理历史后再合并。

## 4. 两个版本之间同步代码的唯一方式

如果某个修复两边都需要，按下面流程处理。

先在源分支提交：

```bash
git switch no-user
git add .
git commit -m "fix: 修复 OCR 结果为空时的异常"
git push
```

查看提交：

```bash
git log --oneline -n 5
```

假设提交号为 `abc1234`，同步到另一条分支：

```bash
git switch multi-user
git pull
git cherry-pick abc1234
git push
```

规则：
- 通用修复优先使用 `cherry-pick`
- 只拣具体提交，不整分支 merge
- 如果 `cherry-pick` 冲突，按当前目标分支的业务语义解决

## 5. 提交信息规范

建议统一使用下面格式：

```text
类型: 简短说明
```

常用类型：
- `feat`：新增功能
- `fix`：修复问题
- `refactor`：重构
- `docs`：文档调整
- `style`：样式调整
- `chore`：构建或杂项维护

示例：

```text
feat: 新增报告导出功能
fix: 修复趋势图日期排序错误
refactor: 拆分 OCR 结果处理逻辑
docs: 补充分支开发规范
```

如果已在仓库中启用提交模板，执行下面命令时会自动带出模板：

```bash
git commit
```

当前仓库推荐使用根目录的 `.gitmessage.txt` 作为提交模板。

## 6. 切换分支前检查清单

每次准备从 `no-user` 切到 `multi-user`，或反过来切换前，先检查：

```bash
git status
```

如果工作区干净，直接切换：

```bash
git switch multi-user
```

如果有未提交修改，按以下三选一处理：

1. 直接提交

```bash
git add .
git commit -m "wip: 保存当前进度"
git push
```

2. 临时存起

```bash
git stash push -m "wip no-user"
git switch multi-user
```

3. 确认无用后手动删除文件修改

不建议直接使用破坏性命令批量清空。

## 7. 发布与回溯规则

每当某个版本达到可发布状态，都打标签。

无用户版示例：

```bash
git switch no-user
git pull
git tag no-user-v0.1.0
git push origin no-user-v0.1.0
```

多用户版示例：

```bash
git switch multi-user
git pull
git tag multi-user-v0.1.0
git push origin multi-user-v0.1.0
```

建议：
- 无用户版标签统一用 `no-user-vx.y.z`
- 多用户版标签统一用 `multi-user-vx.y.z`

这样后面回滚和排查问题会非常直接。

## 8. 禁止事项

以下操作默认禁止：

- 在 `master` 上继续开发
- 用 `merge` 把 `no-user` 整体并入 `multi-user`
- 用 `merge` 把 `multi-user` 整体并回 `no-user`
- 使用 `git push --force`
- 使用 `git reset --hard` 覆盖未确认修改
- 在未 `pull` 的情况下直接跨设备或跨时段推送

## 9. 当前项目的推荐最小流程

无用户版开发：

```bash
git switch no-user
git pull
git add .
git commit -m "feat: xxx"
git push
```

多用户版开发：

```bash
git switch multi-user
git pull
git add .
git commit -m "feat: xxx"
git push
```

跨分支同步公共修复：

```bash
git cherry-pick <commit-id>
```

这就是当前项目最适合的最小 Git 规范：
- 两条长期分支各自演进
- 日常允许直接提交
- 共用修复靠 `cherry-pick`
- 发布节点必须打 tag
