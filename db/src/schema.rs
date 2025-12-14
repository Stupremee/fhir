//! The SQL code for all tables inside the `fhir` schema
//! that are required for this extension to work.

use pgrx::extension_sql;

extension_sql!(
    r#"
CREATE TABLE "fhir"."entity" (
    "id" UUID PRIMARY KEY,
    "resource_type" TEXT NOT NULL,
    "data" JSONB NOT NULL,

    CONSTRAINT "valid_schema" CHECK ("public"."fhir_is_valid"("resource_type", "data"))
);

CREATE INDEX "entity_resource_type_idx" ON "fhir"."entity" ("resource_type");
    "#,
    name = "entity_table",
    requires = [fhir_is_valid]
);

extension_sql!(
    r#"
CREATE TYPE "fhir"."history_operation" AS ENUM (
    'insert',
    'update',
    'delete'
);

CREATE TABLE "fhir"."entity_history" (
    "id" BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

    -- No FK because when deleting an entity, we want to keep the history
    "entity_id" UUID NOT NULL,
    "timestamp" TIMESTAMPTZ NOT NULL,
    "operation" "fhir"."history_operation" NOT NULL,

    "data" JSONB,

    "update_removed_values" JSONB,
    "update_changed_values" JSONB,
    "update_added_values" JSONB
);

CREATE INDEX "entity_history_entity_id_idx" ON "fhir"."entity_history" ("entity_id");

CREATE TRIGGER "entity_history_trigger"
AFTER INSERT OR UPDATE OR DELETE ON "fhir"."entity"
FOR EACH ROW
EXECUTE FUNCTION "public"."fhir_log_entity_history"();
    "#,
    name = "entity_history",
    requires = ["entity_table", fhir_log_entity_history]
);

// The `index_text` table is used to search for entities by string values.
//
// `entity_id` is the reference to the `entity` table.
// `key` is the name of the parameter to search for.
// `value` is the value that can be used for searching.
extension_sql!(
    r#"
CREATE TABLE "fhir"."entity_index_text" (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    entity_id UUID NOT NULL REFERENCES "fhir"."entity" ("id") ON DELETE CASCADE,
    entity TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL
);

-- This index is mainly used for deleting and updating entities, so we can quickly find all values for a single entity.
CREATE INDEX "entity_index_text_entity_id_idx" ON "fhir"."entity_index_text" ("entity_id");
CREATE INDEX "entity_index_text_key_value_idx" ON "fhir"."entity_index_text" ("entity", "key", "value");
CREATE INDEX "entity_index_text_key_idx" ON "fhir"."entity_index_text" ("entity", "key");
CREATE INDEX "entity_index_text_value_gin_idx" ON "fhir"."entity_index_text" USING GIN ("value" gin_trgm_ops);
    "#,
    name = "entity_index_text",
    requires = ["entity_table"]
);

extension_sql!(
    r#"
CREATE TABLE "fhir"."entity_index_date" (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    entity_id UUID NOT NULL REFERENCES "fhir"."entity" ("id") ON DELETE CASCADE,
    entity TEXT NOT NULL,
    key TEXT NOT NULL,
    value DATE NOT NULL
);

CREATE INDEX "entity_index_date_entity_id_idx" ON "fhir"."entity_index_date" ("entity_id");
CREATE INDEX "entity_index_date_key_value_idx" ON "fhir"."entity_index_date" ("entity", "key", "value");
CREATE INDEX "entity_index_date_key_idx" ON "fhir"."entity_index_date" ("entity", "key");
    "#,
    name = "entity_index_date",
    requires = ["entity_table"]
);
