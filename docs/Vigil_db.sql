CREATE TABLE "users" (
  "id" uuid PRIMARY KEY,
  "email" text UNIQUE NOT NULL,
  "password_hash" text NOT NULL,
  "display_name" text NOT NULL,
  "language" text NOT NULL DEFAULT 'en',
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "sessions" (
  "id" uuid PRIMARY KEY,
  "user_id" uuid NOT NULL,
  "token_hash" bytea UNIQUE NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "expires_at" timestamptz NOT NULL
);

CREATE TABLE "teams" (
  "id" uuid PRIMARY KEY,
  "name" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "created_by" uuid NOT NULL
);

CREATE TABLE "team_members" (
  "id" uuid PRIMARY KEY,
  "user_id" uuid NOT NULL,
  "team_id" uuid NOT NULL,
  "role" text NOT NULL,
  "status" text NOT NULL DEFAULT 'active',
  "joined_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "invitations" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid NOT NULL,
  "code" text UNIQUE NOT NULL,
  "created_by" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "expires_at" timestamptz,
  "max_uses" int,
  "uses" int NOT NULL DEFAULT 0,
  "status" text NOT NULL DEFAULT 'active'
);

CREATE TABLE "team_bans" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "created_by" uuid NOT NULL,
  "reason" text,
  "expires_at" timestamptz,
  "status" text NOT NULL DEFAULT 'active',
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "incidents" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid NOT NULL,
  "title" text NOT NULL,
  "body" text NOT NULL DEFAULT '',
  "severity" text NOT NULL,
  "status" text NOT NULL DEFAULT 'open',
  "created_by" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now()),
  "acknowledged_at" timestamptz,
  "escalated_at" timestamptz,
  "resolved_at" timestamptz
);

CREATE TABLE "incident_assignments" (
  "id" uuid PRIMARY KEY,
  "incident_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "assigned_by" uuid NOT NULL,
  "status" text NOT NULL DEFAULT 'active',
  "assigned_at" timestamptz NOT NULL DEFAULT (now()),
  "unassigned_at" timestamptz
);

CREATE TABLE "timeline_entries" (
  "id" uuid PRIMARY KEY,
  "incident_id" uuid NOT NULL,
  "author_id" uuid NOT NULL,
  "kind" text NOT NULL DEFAULT 'message',
  "content" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "edited_at" timestamptz
);

CREATE TABLE "timeline_reactions" (
  "id" uuid PRIMARY KEY,
  "entry_id" uuid NOT NULL,
  "user_id" uuid NOT NULL,
  "emoji" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "releases" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid NOT NULL,
  "title" text NOT NULL,
  "body" text NOT NULL DEFAULT '',
  "status" text NOT NULL DEFAULT 'created',
  "created_by" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now()),
  "started_at" timestamptz,
  "completed_at" timestamptz,
  "cancelled_at" timestamptz
);

CREATE TABLE "release_steps" (
  "id" uuid PRIMARY KEY,
  "release_id" uuid NOT NULL,
  "name" text NOT NULL,
  "position" int NOT NULL,
  "validated_by" uuid,
  "validated_at" timestamptz,
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "release_incident_links" (
  "id" uuid PRIMARY KEY,
  "release_id" uuid NOT NULL,
  "incident_id" uuid NOT NULL,
  "linked_by" uuid NOT NULL,
  "linked_at" timestamptz NOT NULL DEFAULT (now()),
  "status" text NOT NULL DEFAULT 'active'
);

CREATE TABLE "private_messages" (
  "id" uuid PRIMARY KEY,
  "sender_id" uuid NOT NULL,
  "recipient_id" uuid NOT NULL,
  "content" text NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "service_connections" (
  "id" uuid PRIMARY KEY,
  "user_id" uuid NOT NULL,
  "service" text NOT NULL,
  "encrypted_token" bytea NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "rules" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid NOT NULL,
  "name" text NOT NULL,
  "enabled" boolean NOT NULL DEFAULT true,
  "trigger_service" text NOT NULL,
  "trigger_event" text NOT NULL,
  "trigger_filters" jsonb NOT NULL DEFAULT '{}',
  "reaction_type" text NOT NULL,
  "reaction_payload" jsonb NOT NULL DEFAULT '{}',
  "created_by" uuid NOT NULL,
  "created_at" timestamptz NOT NULL DEFAULT (now()),
  "updated_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "webhook_deliveries" (
  "id" uuid PRIMARY KEY,
  "service" text NOT NULL,
  "event_type" text NOT NULL,
  "payload" jsonb NOT NULL,
  "headers" jsonb,
  "source" text,
  "hmac_valid" boolean,
  "received_at" timestamptz NOT NULL DEFAULT (now()),
  "processed_at" timestamptz
);

CREATE TABLE "rule_executions" (
  "id" uuid PRIMARY KEY,
  "delivery_id" uuid,
  "rule_id" uuid NOT NULL,
  "status" text NOT NULL,
  "result" text,
  "error" text,
  "incident_id" uuid,
  "executed_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE TABLE "audit_log" (
  "id" uuid PRIMARY KEY,
  "team_id" uuid,
  "actor_id" uuid,
  "action" text NOT NULL,
  "entity_type" text NOT NULL,
  "entity_id" uuid NOT NULL,
  "metadata" jsonb NOT NULL DEFAULT '{}',
  "created_at" timestamptz NOT NULL DEFAULT (now())
);

CREATE UNIQUE INDEX ON "team_members" ("team_id", "user_id");

CREATE INDEX ON "team_members" ("user_id");

CREATE INDEX "invitations_team_id_idx" ON "invitations" ("team_id");

CREATE INDEX "team_bans_one_active_idx" ON "team_bans" ("team_id", "user_id");

CREATE INDEX "team_bans_team_id_idx" ON "team_bans" ("team_id");

CREATE INDEX "incidents_team_id_idx" ON "incidents" ("team_id");

CREATE INDEX "incidents_team_status_idx" ON "incidents" ("team_id", "status");

CREATE INDEX "incidents_team_severity_idx" ON "incidents" ("team_id", "severity");

CREATE INDEX "assignments_one_active_idx" ON "incident_assignments" ("incident_id");

CREATE INDEX "assignments_incident_id_idx" ON "incident_assignments" ("incident_id");

CREATE INDEX "assignments_user_id_idx" ON "incident_assignments" ("user_id");

CREATE INDEX "timeline_entries_incident_chrono_idx" ON "timeline_entries" ("incident_id", "created_at");

CREATE UNIQUE INDEX "reactions_unique_idx" ON "timeline_reactions" ("entry_id", "user_id", "emoji");

CREATE INDEX "reactions_entry_id_idx" ON "timeline_reactions" ("entry_id");

CREATE INDEX "releases_team_id_idx" ON "releases" ("team_id");

CREATE INDEX "releases_team_status_idx" ON "releases" ("team_id", "status");

CREATE UNIQUE INDEX "steps_release_position_idx" ON "release_steps" ("release_id", "position");

CREATE UNIQUE INDEX "steps_release_name_idx" ON "release_steps" ("release_id", "name");

CREATE INDEX "steps_release_id_idx" ON "release_steps" ("release_id");

CREATE INDEX "links_active_idx" ON "release_incident_links" ("release_id", "incident_id");

CREATE INDEX "links_release_id_idx" ON "release_incident_links" ("release_id");

CREATE INDEX "links_incident_id_idx" ON "release_incident_links" ("incident_id");

CREATE INDEX "pm_recipient_idx" ON "private_messages" ("recipient_id", "created_at");

CREATE UNIQUE INDEX "connections_user_service_idx" ON "service_connections" ("user_id", "service");

CREATE INDEX "rules_team_id_idx" ON "rules" ("team_id");

CREATE INDEX "rules_trigger_idx" ON "rules" ("trigger_service", "trigger_event");

CREATE INDEX "deliveries_received_idx" ON "webhook_deliveries" ("received_at");

CREATE INDEX "deliveries_service_idx" ON "webhook_deliveries" ("service", "received_at");

CREATE INDEX "executions_rule_id_idx" ON "rule_executions" ("rule_id");

CREATE INDEX "executions_delivery_id_idx" ON "rule_executions" ("delivery_id");

CREATE INDEX "executions_executed_idx" ON "rule_executions" ("executed_at");

CREATE INDEX "audit_team_id_idx" ON "audit_log" ("team_id", "created_at");

CREATE INDEX "audit_actor_id_idx" ON "audit_log" ("actor_id", "created_at");

CREATE INDEX "audit_entity_idx" ON "audit_log" ("entity_type", "entity_id");

CREATE INDEX "audit_action_idx" ON "audit_log" ("action", "created_at");

COMMENT ON COLUMN "users"."language" IS 'CHECK in (''fr'',''en'')';

COMMENT ON COLUMN "team_members"."role" IS 'CHECK in (''observer'',''responder'',''manager'')';

COMMENT ON COLUMN "team_members"."status" IS 'CHECK in (''active'',''kicked'')';

COMMENT ON COLUMN "invitations"."expires_at" IS 'null = never';

COMMENT ON COLUMN "invitations"."max_uses" IS 'null = unlimited';

COMMENT ON COLUMN "invitations"."status" IS '''active'' | ''revoked''';

COMMENT ON COLUMN "team_bans"."expires_at" IS 'null = permanent';

COMMENT ON COLUMN "team_bans"."status" IS '''active'' | ''lifted''';

COMMENT ON COLUMN "incidents"."severity" IS 'CHECK in (''low'',''medium'',''high'',''critical'')';

COMMENT ON COLUMN "incidents"."status" IS 'CHECK in (''open'',''acknowledged'',''escalated'',''resolved'')';

COMMENT ON COLUMN "incident_assignments"."status" IS 'CHECK in (''active'',''replaced'',''removed'')';

COMMENT ON COLUMN "timeline_entries"."kind" IS 'CHECK in (''message'',''system'')';

COMMENT ON COLUMN "timeline_reactions"."emoji" IS 'CHECK in (''+1'',''-1'',''eyes'',''warning'',''check'',''fire'')';

COMMENT ON COLUMN "releases"."status" IS 'CHECK in (''created'',''in_progress'',''completed'',''cancelled'',''blocked'')';

COMMENT ON COLUMN "release_steps"."position" IS 'CHECK > 0';

COMMENT ON COLUMN "release_incident_links"."status" IS 'CHECK in (''active'',''removed'')';

COMMENT ON COLUMN "private_messages"."content" IS 'CHECK char_length <= 2000';

COMMENT ON COLUMN "service_connections"."service" IS 'CHECK in (''github'',''gitlab'',''discord'')';

COMMENT ON COLUMN "webhook_deliveries"."source" IS 'ex: my-org/my-repo';

COMMENT ON COLUMN "webhook_deliveries"."hmac_valid" IS 'null = no HMAC configured';

COMMENT ON COLUMN "rule_executions"."status" IS 'CHECK in (''success'',''failure'')';

COMMENT ON COLUMN "rule_executions"."result" IS 'ex: incident_created';

COMMENT ON COLUMN "audit_log"."team_id" IS 'no FK — survives team deletion';

COMMENT ON COLUMN "audit_log"."actor_id" IS 'null = system action, no FK';

COMMENT ON COLUMN "audit_log"."action" IS 'ex: member_kicked, ban_created, release_cancelled';

COMMENT ON COLUMN "audit_log"."entity_type" IS 'ex: team_member, incident, rule';

COMMENT ON COLUMN "audit_log"."entity_id" IS 'no FK — survives entity deletion';

ALTER TABLE "sessions" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "teams" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_members" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_members" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "invitations" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "invitations" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_bans" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") ON DELETE CASCADE DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_bans" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "team_bans" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incidents" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incidents" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incidents" ADD FOREIGN KEY ("status") REFERENCES "incidents" ("created_by") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incident_assignments" ADD FOREIGN KEY ("incident_id") REFERENCES "incidents" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incident_assignments" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "incident_assignments" ADD FOREIGN KEY ("assigned_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "timeline_entries" ADD FOREIGN KEY ("incident_id") REFERENCES "incidents" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "timeline_entries" ADD FOREIGN KEY ("author_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "timeline_reactions" ADD FOREIGN KEY ("entry_id") REFERENCES "timeline_entries" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "timeline_reactions" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "releases" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "releases" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "release_steps" ADD FOREIGN KEY ("release_id") REFERENCES "releases" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "release_steps" ADD FOREIGN KEY ("validated_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "release_incident_links" ADD FOREIGN KEY ("release_id") REFERENCES "releases" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "release_incident_links" ADD FOREIGN KEY ("incident_id") REFERENCES "incidents" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "release_incident_links" ADD FOREIGN KEY ("linked_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "private_messages" ADD FOREIGN KEY ("sender_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "private_messages" ADD FOREIGN KEY ("recipient_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "service_connections" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "rules" ADD FOREIGN KEY ("team_id") REFERENCES "teams" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "rules" ADD FOREIGN KEY ("created_by") REFERENCES "users" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "rule_executions" ADD FOREIGN KEY ("delivery_id") REFERENCES "webhook_deliveries" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "rule_executions" ADD FOREIGN KEY ("rule_id") REFERENCES "rules" ("id") DEFERRABLE INITIALLY IMMEDIATE;

ALTER TABLE "rule_executions" ADD FOREIGN KEY ("incident_id") REFERENCES "incidents" ("id") DEFERRABLE INITIALLY IMMEDIATE;
