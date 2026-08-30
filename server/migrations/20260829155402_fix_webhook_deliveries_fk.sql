-- Add migration script here
ALTER TABLE webhook_deliveries
    DROP CONSTRAINT webhook_deliveries_connection_id_fkey,
    ADD CONSTRAINT webhook_deliveries_connection_id_fkey
        FOREIGN KEY (connection_id)
        REFERENCES team_service_connections(id)
        ON DELETE SET NULL;