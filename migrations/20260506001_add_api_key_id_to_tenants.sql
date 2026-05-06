ALTER TABLE tenants
ADD COLUMN IF NOT EXISTS api_key_id VARCHAR(64);

UPDATE tenants
SET api_key_id = 'legacy_' || substr(md5(tenant_id || ':' || app_id), 1, 16)
WHERE api_key_id IS NULL OR api_key_id = '';

ALTER TABLE tenants
ALTER COLUMN api_key_id SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_tenants_api_key_id
ON tenants(api_key_id);
