-- Each team configures its own integrations. The Manager enters a secret
-- or token, the server encrypts it (AES-256-GCM, same master_key), and
-- generates a unique webhook endpoint /webhooks/<connection_id>.

CREATE TABLE team_service_connections (
    id              UUID PRIMARY KEY,
    team_id         UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    service         TEXT NOT NULL,
    encrypted_token BYTEA NOT NULL,
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT team_conn_service_check
        CHECK (service IN ('github', 'discord'))
);

-- One connection per service per team
CREATE UNIQUE INDEX team_conn_team_service_idx
    ON team_service_connections (team_id, service);

-- Track which connection received a webhook delivery (debugging/audit)
ALTER TABLE webhook_deliveries
    ADD COLUMN connection_id UUID REFERENCES team_service_connections(id);