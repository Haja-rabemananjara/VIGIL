
-- 1. USERS
CREATE TABLE users (
                       id            UUID PRIMARY KEY,
                       email         TEXT NOT NULL,
                       password_hash TEXT NOT NULL,
                       display_name  TEXT NOT NULL,
                       language      TEXT NOT NULL DEFAULT 'en',
                       created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

                       CONSTRAINT users_language_check CHECK (language IN ('fr', 'en'))
    );

CREATE UNIQUE INDEX users_email_key ON users (lower(email));


-- 2. SESSIONS
CREATE TABLE sessions (
                          id         UUID PRIMARY KEY,
                          user_id    UUID NOT NULL REFERENCES users(id),
                          token_hash BYTEA NOT NULL,
                          created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                          expires_at TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX sessions_token_hash_key ON sessions (token_hash);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);


-- 3. TEAMS
CREATE TABLE teams (
                       id         UUID PRIMARY KEY,
                       name       TEXT NOT NULL,
                       created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                       created_by UUID NOT NULL REFERENCES users(id)
);


-- 4. TEAM_MEMBERS
CREATE TABLE team_members (
                              id        UUID PRIMARY KEY,
                              team_id   UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                              user_id   UUID NOT NULL REFERENCES users(id),
                              role      TEXT NOT NULL,
                              status    TEXT NOT NULL DEFAULT 'active',
                              joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                              CONSTRAINT team_members_role_check   CHECK (role IN ('observer', 'responder', 'manager')),
                              CONSTRAINT team_members_status_check CHECK (status IN ('active', 'kicked'))
);

CREATE UNIQUE INDEX team_members_team_user_key ON team_members (team_id, user_id);
CREATE INDEX team_members_user_id_idx ON team_members (user_id);

-- Invariant: exactly one active Manager per team
CREATE UNIQUE INDEX team_members_one_manager_idx
    ON team_members (team_id)
    WHERE role = 'manager' AND status = 'active';


-- 5. INVITATIONS
CREATE TABLE invitations (
                             id         UUID PRIMARY KEY,
                             team_id    UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                             code       TEXT NOT NULL,
                             created_by UUID NOT NULL REFERENCES users(id),
                             created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                             expires_at TIMESTAMPTZ,
                             max_uses   INTEGER,
                             uses       INTEGER NOT NULL DEFAULT 0,
                             status     TEXT NOT NULL DEFAULT 'active',

                             CONSTRAINT invitations_status_check   CHECK (status IN ('active', 'revoked')),
                             CONSTRAINT invitations_max_uses_check CHECK (max_uses IS NULL OR max_uses > 0),
                             CONSTRAINT invitations_uses_check     CHECK (uses >= 0)
);

CREATE UNIQUE INDEX invitations_code_key ON invitations (code);
CREATE INDEX invitations_team_id_idx ON invitations (team_id);


-- 6. TEAM_BANS
CREATE TABLE team_bans (
                           id         UUID PRIMARY KEY,
                           team_id    UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                           user_id    UUID NOT NULL REFERENCES users(id),
                           created_by UUID NOT NULL REFERENCES users(id),
                           reason     TEXT,
                           expires_at TIMESTAMPTZ,
                           status     TEXT NOT NULL DEFAULT 'active',
                           created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                           CONSTRAINT team_bans_status_check CHECK (status IN ('active', 'lifted')),
                           CONSTRAINT team_bans_expiry_check CHECK (expires_at IS NULL OR expires_at > created_at)
);

-- Invariant: one active ban per (team, user)
CREATE UNIQUE INDEX team_bans_one_active_idx
    ON team_bans (team_id, user_id)
    WHERE status = 'active';
CREATE INDEX team_bans_team_id_idx ON team_bans (team_id);


-- 7. INCIDENTS
CREATE TABLE incidents (
                           id              UUID PRIMARY KEY,
                           team_id         UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                           title           TEXT NOT NULL,
                           body            TEXT NOT NULL DEFAULT '',
                           severity        TEXT NOT NULL,
                           status          TEXT NOT NULL DEFAULT 'open',
                           created_by      UUID NOT NULL REFERENCES users(id),
                           created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                           updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                           acknowledged_at TIMESTAMPTZ,
                           escalated_at    TIMESTAMPTZ,
                           resolved_at     TIMESTAMPTZ,

                           CONSTRAINT incidents_severity_check CHECK (severity IN ('low', 'medium', 'high', 'critical')),
                           CONSTRAINT incidents_status_check   CHECK (status IN ('open', 'acknowledged', 'escalated', 'resolved'))
);

CREATE INDEX incidents_team_id_idx       ON incidents (team_id);
CREATE INDEX incidents_team_status_idx   ON incidents (team_id, status);
CREATE INDEX incidents_team_severity_idx ON incidents (team_id, severity);


-- 8. INCIDENT_ASSIGNMENTS
CREATE TABLE incident_assignments (
                                      id            UUID PRIMARY KEY,
                                      incident_id   UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
                                      user_id       UUID NOT NULL REFERENCES users(id),
                                      assigned_by   UUID NOT NULL REFERENCES users(id),
                                      status        TEXT NOT NULL DEFAULT 'active',
                                      assigned_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                                      unassigned_at TIMESTAMPTZ,

                                      CONSTRAINT assignments_status_check CHECK (status IN ('active', 'replaced', 'removed'))
);

-- Invariant: one active assignee per incident
CREATE UNIQUE INDEX assignments_one_active_idx
    ON incident_assignments (incident_id)
    WHERE status = 'active';
CREATE INDEX assignments_incident_id_idx ON incident_assignments (incident_id);
CREATE INDEX assignments_user_id_idx ON incident_assignments (user_id);


-- 9. TIMELINE_ENTRIES
CREATE TABLE timeline_entries (
                                  id          UUID PRIMARY KEY,
                                  incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
                                  author_id   UUID NOT NULL REFERENCES users(id),
                                  kind        TEXT NOT NULL DEFAULT 'message',
                                  content     TEXT NOT NULL,
                                  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                                  edited_at   TIMESTAMPTZ,

                                  CONSTRAINT timeline_entries_kind_check CHECK (kind IN ('message', 'system'))
);

CREATE INDEX timeline_entries_incident_chrono_idx
    ON timeline_entries (incident_id, created_at);


-- 10. TIMELINE_REACTIONS
CREATE TABLE timeline_reactions (
                                    id         UUID PRIMARY KEY,
                                    entry_id   UUID NOT NULL REFERENCES timeline_entries(id) ON DELETE CASCADE,
                                    user_id    UUID NOT NULL REFERENCES users(id),
                                    emoji      TEXT NOT NULL,
                                    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                                    CONSTRAINT reactions_emoji_check CHECK (emoji IN ('+1', '-1', 'eyes', 'warning', 'check', 'fire'))
);

-- Invariant: one reaction per (entry, user, emoji)
CREATE UNIQUE INDEX reactions_unique_idx ON timeline_reactions (entry_id, user_id, emoji);
CREATE INDEX reactions_entry_id_idx ON timeline_reactions (entry_id);


-- 11. RELEASES
CREATE TABLE releases (
                          id           UUID PRIMARY KEY,
                          team_id      UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                          title        TEXT NOT NULL,
                          body         TEXT NOT NULL DEFAULT '',
                          status       TEXT NOT NULL DEFAULT 'created',
                          created_by   UUID NOT NULL REFERENCES users(id),
                          created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                          updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                          started_at   TIMESTAMPTZ,
                          completed_at TIMESTAMPTZ,
                          cancelled_at TIMESTAMPTZ,

                          CONSTRAINT releases_status_check CHECK (status IN ('created', 'in_progress', 'completed', 'cancelled', 'blocked'))
);

CREATE INDEX releases_team_id_idx     ON releases (team_id);
CREATE INDEX releases_team_status_idx ON releases (team_id, status);


-- 12. RELEASE_STEPS
CREATE TABLE release_steps (
                               id           UUID PRIMARY KEY,
                               release_id   UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
                               name         TEXT NOT NULL,
                               position     INTEGER NOT NULL,
                               validated_by UUID REFERENCES users(id),
                               validated_at TIMESTAMPTZ,
                               created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

                               CONSTRAINT steps_position_positive CHECK (position > 0),
                               CONSTRAINT steps_validation_pair   CHECK (
                                   (validated_by IS NULL AND validated_at IS NULL) OR
                                   (validated_by IS NOT NULL AND validated_at IS NOT NULL)
                                   )
);

CREATE UNIQUE INDEX steps_release_position_idx ON release_steps (release_id, position);
CREATE UNIQUE INDEX steps_release_name_idx     ON release_steps (release_id, name);
CREATE INDEX        steps_release_id_idx       ON release_steps (release_id);


-- 13. RELEASE_INCIDENT_LINKS
CREATE TABLE release_incident_links (
                                        id          UUID PRIMARY KEY,
                                        release_id  UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
                                        incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
                                        linked_by   UUID NOT NULL REFERENCES users(id),
                                        linked_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                                        status      TEXT NOT NULL DEFAULT 'active',

                                        CONSTRAINT links_status_check CHECK (status IN ('active', 'removed'))
);

-- Invariant: one active link per (release, incident)
CREATE UNIQUE INDEX links_active_idx
    ON release_incident_links (release_id, incident_id)
    WHERE status = 'active';
CREATE INDEX links_release_id_idx  ON release_incident_links (release_id);
CREATE INDEX links_incident_id_idx ON release_incident_links (incident_id);


-- 14. PRIVATE_MESSAGES
CREATE TABLE private_messages (
                                  id           UUID PRIMARY KEY,
                                  sender_id    UUID NOT NULL REFERENCES users(id),
                                  recipient_id UUID NOT NULL REFERENCES users(id),
                                  content      TEXT NOT NULL,
                                  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

                                  CONSTRAINT messages_content_length CHECK (char_length(content) <= 2000),
                                  CONSTRAINT messages_not_self       CHECK (sender_id != recipient_id)
    );

-- Functional index for bidirectional conversation lookup
CREATE INDEX pm_conversation_idx ON private_messages (
                                                      LEAST(sender_id, recipient_id),
                                                      GREATEST(sender_id, recipient_id),
                                                      created_at
    );
CREATE INDEX pm_recipient_idx ON private_messages (recipient_id, created_at);


-- 15. SERVICE_CONNECTIONS
CREATE TABLE service_connections (
                                     id              UUID PRIMARY KEY,
                                     user_id         UUID NOT NULL REFERENCES users(id),
                                     service         TEXT NOT NULL,
                                     encrypted_token BYTEA NOT NULL,
                                     created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                                     updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

                                     CONSTRAINT connections_service_check CHECK (service IN ('github', 'gitlab', 'discord'))
);

CREATE UNIQUE INDEX connections_user_service_idx ON service_connections (user_id, service);


-- 16. RULES
CREATE TABLE rules (
                       id               UUID PRIMARY KEY,
                       team_id          UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                       name             TEXT NOT NULL,
                       enabled          BOOLEAN NOT NULL DEFAULT true,
                       trigger_service  TEXT NOT NULL,
                       trigger_event    TEXT NOT NULL,
                       trigger_filters  JSONB NOT NULL DEFAULT '{}',
                       reaction_type    TEXT NOT NULL,
                       reaction_payload JSONB NOT NULL DEFAULT '{}',
                       created_by       UUID NOT NULL REFERENCES users(id),
                       created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
                       updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rules_team_id_idx ON rules (team_id);
CREATE INDEX rules_trigger_idx
    ON rules (trigger_service, trigger_event)
    WHERE enabled = true;


-- 17. WEBHOOK_DELIVERIES
CREATE TABLE webhook_deliveries (
                                    id           UUID PRIMARY KEY,
                                    service      TEXT NOT NULL,
                                    event_type   TEXT NOT NULL,
                                    payload      JSONB NOT NULL,
                                    headers      JSONB,
                                    source       TEXT,
                                    hmac_valid   BOOLEAN,
                                    received_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
                                    processed_at TIMESTAMPTZ
);

CREATE INDEX deliveries_received_idx ON webhook_deliveries (received_at);
CREATE INDEX deliveries_service_idx  ON webhook_deliveries (service, received_at);


-- 18. RULE_EXECUTIONS
CREATE TABLE rule_executions (
                                 id          UUID PRIMARY KEY,
                                 rule_id     UUID NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
                                 delivery_id UUID REFERENCES webhook_deliveries(id),
                                 status      TEXT NOT NULL,
                                 result      TEXT,
                                 error       TEXT,
                                 incident_id UUID REFERENCES incidents(id),
                                 executed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

                                 CONSTRAINT executions_status_check CHECK (status IN ('success', 'failure'))
);

CREATE INDEX executions_rule_id_idx     ON rule_executions (rule_id);
CREATE INDEX executions_delivery_id_idx ON rule_executions (delivery_id);
CREATE INDEX executions_executed_idx    ON rule_executions (executed_at);


-- 19. AUDIT_LOG
CREATE TABLE audit_log (
                           id          UUID PRIMARY KEY,
                           team_id     UUID,
                           actor_id    UUID,
                           action      TEXT NOT NULL,
                           entity_type TEXT NOT NULL,
                           entity_id   UUID NOT NULL,
                           metadata    JSONB NOT NULL DEFAULT '{}',
                           created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_team_id_idx  ON audit_log (team_id, created_at);
CREATE INDEX audit_actor_id_idx ON audit_log (actor_id, created_at);
CREATE INDEX audit_entity_idx   ON audit_log (entity_type, entity_id);
CREATE INDEX audit_action_idx   ON audit_log (action, created_at);
