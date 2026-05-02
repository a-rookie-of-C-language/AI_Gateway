CREATE TABLE IF NOT EXISTS token_usage_records (
    request_id        VARCHAR(128) PRIMARY KEY,
    tenant_id         VARCHAR(64)  NOT NULL,
    app_id            VARCHAR(64)  NOT NULL,
    model             VARCHAR(128) NOT NULL,
    prompt_tokens     BIGINT NOT NULL DEFAULT 0,
    completion_tokens BIGINT NOT NULL DEFAULT 0,
    total_tokens      BIGINT NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_usage_tenant_time ON token_usage_records (tenant_id, created_at DESC);
CREATE INDEX idx_usage_model_time  ON token_usage_records (model, created_at DESC);
CREATE INDEX idx_usage_tenant_model_time ON token_usage_records (tenant_id, model, created_at DESC);
