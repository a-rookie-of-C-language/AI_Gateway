CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    request_id VARCHAR(64) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    app_id VARCHAR(64) NOT NULL,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(512) NOT NULL,
    status_code SMALLINT NOT NULL,
    duration_ms INTEGER NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_request_id ON audit_logs(request_id);
CREATE INDEX idx_audit_logs_tenant_app ON audit_logs(tenant_id, app_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
