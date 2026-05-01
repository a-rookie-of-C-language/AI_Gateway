# AIGateway (Rust + 三层架构 + DDD)

AIGateway 是面向大模型应用的高并发模型网关与限流计费系统。

当前版本已切换为 Rust 实现，保留核心接口：
- `GET /v1/health`
- `POST /v1/chat/completions`
- `POST /v1/chat/stream`（SSE）

### 流式事件格式（`/v1/chat/stream`）

- `event: raw`，`data: <上游原始JSON>`  
- `event: done`，`data: {"finish_reason":"stop|error"}`  
- `event: error`，`data: {"message":"..."}`（出现异常时）

## 架构分层

- `domain`：领域模型与端口接口（`ChatGateway`、`RateLimiter`）
- `application`：用例编排（`ChatAppService`）
- `infrastructure`：外部实现（Axum Router、Redis 限流、Provider）
- `interfaces`：HTTP Handler 与中间件（鉴权、限流、请求 ID）

依赖方向：`interfaces -> application -> domain`，`infrastructure` 提供实现。

## 领域划分（先定义再实现）

### 核心域（Core Domain）

1. 模型网关编排域（Model Gateway Orchestration）
- 统一入口
- 模型路由策略
- 熔断与降级
- 请求生命周期编排

2. 配额计费域（Quota & Billing）
- Token 统计
- 调用计量
- 套餐配额与扣费
- 账单汇总

3. 租户与访问控制域（Tenant Access Control）
- API Key 管理
- 应用身份识别
- 租户隔离
- 权限模型

### 支撑域（Supporting Domain）

1. 限流与流量治理域（Traffic Governance）
- 分布式限流
- 队列削峰
- 并发控制

2. 可观测与审计域（Observability & Audit）
- 调用日志
- Trace 追踪
- 指标采集
- 审计记录

3. 提供商适配域（Provider Integration）
- 多模型提供商对接
- 协议转换
- 错误语义映射

### 通用域（Generic Domain）

1. 配置与环境管理（Config）
2. HTTP 传输与中间件（Web Adapter）
3. Redis/PostgreSQL 基础访问组件
4. 统一错误码与响应封装
5. 启动装配与依赖注入

## 目录

- `src/main.rs`：启动入口
- `src/bootstrap/*`：依赖装配与服务启动
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

1. 将网关流式链路升级为真正边读边推（当前为聚合后推送）。
2. 限流从 INCR+EXPIRE 升级为 Lua 滑动窗口脚本。
3. 增加 token 统计与计费聚合（PostgreSQL + 异步任务）。
4. 如需贴合你的习惯，可把 `interfaces/http` 迁移到 `rust-spring` 风格 Controller。
