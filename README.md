# AIGateway (Rust + 三层架构 + DDD)

AIGateway 是面向大模型应用的高并发模型网关与限流计费系统。

当前版本已切换为 Rust 实现，保留核心接口：
- `GET /v1/health`
- `POST /v1/chat/completions`

## 架构分层

- `domain`：领域模型与端口接口（`ChatGateway`、`RateLimiter`）
- `application`：用例编排（`ChatAppService`）
- `infrastructure`：外部实现（Axum Router、Redis 限流、Provider）
- `interfaces`：HTTP Handler 与中间件（鉴权、限流、请求 ID）

依赖方向：`interfaces -> application -> domain`，`infrastructure` 提供实现。

## 目录

- `src/main.rs`：启动入口
- `src/bootstrap.rs`：依赖装配与服务启动
- `src/config.rs`：环境配置
- `src/domain/*`：领域层
- `src/application/*`：应用层
- `src/infrastructure/*`：基础设施层
- `src/interfaces/*`：接口适配层
- `src/shared/*`：响应工具

## 运行

```bash
cargo check
cargo run
```

## 环境变量

参考 `.env.example`：
- `APP_NAME`
- `HTTP_ADDR`
- `MASTER_API_KEY`
- `REDIS_ADDR`
- `RATE_LIMIT_PER_MIN`

## 下一步建议

1. 将 `MockChatGateway` 替换成真实 OpenAI 兼容 Provider（含 stream）。
2. 限流从 INCR+EXPIRE 升级为 Lua 滑动窗口脚本。
3. 增加 token 统计与计费聚合（PostgreSQL + 异步任务）。
4. 如需贴合你的习惯，可把 `interfaces/http` 迁移到 `rust-spring` 风格 Controller。
