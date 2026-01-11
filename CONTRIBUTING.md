# 贡献指南

感谢你对XORE项目的关注！

## 如何贡献

1. Fork本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启Pull Request

## 提交规范

请遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<类型>(<范围>): <简短描述>

<详细描述>

<关联Issue>
```

### 类型说明

- `feat`: 新功能
- `fix`: Bug修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链

## 代码规范

请参阅 [开发规范文档](supplementary/开发规范文档.md)

## 测试要求

- 单元测试覆盖率 >80%
- 所有测试必须通过
- 通过 `cargo fmt` 和 `cargo clippy` 检查

## Pull Request检查清单

- [ ] 代码已格式化
- [ ] 通过Clippy检查
- [ ] 添加了测试
- [ ] 更新了文档
- [ ] 通过了CI

## 问题反馈

请通过 [GitHub Issues](https://github.com/yourusername/xore/issues) 提交问题。

## 行为准则

请保持友善和专业。
