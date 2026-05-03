CREATE TABLE IF NOT EXISTS tenants (
    tenant_id VARCHAR(64) NOT NULL,
    app_id VARCHAR(64) NOT NULL,
    api_key_hash VARCHAR(128) NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, app_id)
);

CREATE INDEX idx_tenants_api_key_hash ON tenants(api_key_hash);
CREATE INDEX idx_tenants_enabled ON tenants(enabled) WHERE enabled = true;
