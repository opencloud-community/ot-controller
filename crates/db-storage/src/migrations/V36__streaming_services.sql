-- Rename column for consisteny.
ALTER TABLE room_streaming_targets RENAME COLUMN streaming_endpoint TO streaming_url;

-- Create streaming services table.
CREATE TABLE streaming_services (
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    kind streaming_kind NOT NULL,
    streaming_url TEXT,
    streaming_key_regex TEXT,
    public_url_regex TEXT
);

-- Create a FK from streaming targets to streaming services (leaving the FK values of existing records empty for now).
ALTER TABLE room_streaming_targets ADD COLUMN service_id UUID REFERENCES streaming_services(id);

-- If there are some streaming targets (which are all of kind 'custom'), create a new 'custom' streaming service and
-- link all existing streaming targets to it.
DO $$
DECLARE
    streaming_targets_existing BOOLEAN;
    streaming_service_id UUID;
BEGIN
    SELECT exists(select 1 FROM room_streaming_targets) INTO streaming_targets_existing;
    IF streaming_targets_existing THEN
        INSERT INTO streaming_services (name, kind)
            VALUES ('Migrated Custom', 'custom')
            RETURNING id INTO streaming_service_id;
        UPDATE room_streaming_targets SET service_id = streaming_service_id;
    END IF;
END;
$$;

-- From now on, enforce FK values linking streaming targets to streaming services.
ALTER TABLE room_streaming_targets ALTER COLUMN service_id SET NOT NULL;

-- Create an additional FK from streaming targets to streaming services, including the redundant kind
-- column which is used for consistency checks.
ALTER TABLE streaming_services ADD UNIQUE (id, kind);
ALTER TABLE room_streaming_targets ADD FOREIGN KEY (service_id, kind)
    REFERENCES streaming_services (id, kind);

-- Allow NULL for these fields as their constraints are a bit more complex and we run more detailed checks.
ALTER TABLE room_streaming_targets ALTER COLUMN streaming_url DROP NOT NULL;
ALTER TABLE room_streaming_targets ALTER COLUMN streaming_key DROP NOT NULL;
ALTER TABLE room_streaming_targets ALTER COLUMN public_url DROP NOT NULL;

-- Create a trigger that sets the (redundant) kind of a new streaming target to the kind of the streaming
-- service it is assigned to.
CREATE OR REPLACE FUNCTION set_room_streaming_target_kind()
    RETURNS TRIGGER
    LANGUAGE PLPGSQL
AS $$
DECLARE
    l_kind streaming_kind;
BEGIN
    SELECT kind INTO STRICT l_kind FROM streaming_services WHERE id = NEW.service_id;
    NEW.kind := l_kind;
    RETURN NEW;
END;
$$;
CREATE TRIGGER insert_room_streaming_target_trigger
    BEFORE INSERT ON room_streaming_targets
    FOR EACH ROW
EXECUTE PROCEDURE set_room_streaming_target_kind();

-- Add checks for the kinds of streaming services.
ALTER TABLE streaming_services ADD CONSTRAINT check_streaming_service_kind
    CHECK (
        -- For 'builtin' none of these values are needed at all
        kind = 'builtin' AND streaming_url IS NULL AND streaming_key_regex IS NULL AND public_url_regex IS NULL
            OR
        -- For 'custom' no values can be set on the service level, they have to be set on the target level
        kind = 'custom' AND streaming_url IS NULL AND streaming_key_regex IS NULL AND public_url_regex IS NULL
            OR
        -- For 'provider' the streaming URL is set on the service level, and so are the rules for the other values
        kind = 'provider' AND streaming_url IS NOT NULL AND streaming_key_regex IS NOT NULL AND public_url_regex IS NOT NULL
    );

-- Add checks for the kinds of room streaming targets.
ALTER TABLE room_streaming_targets ADD CONSTRAINT check_room_streaming_target_kind
    CHECK (
        -- For 'builtin' none of these values are needed at all
        kind = 'builtin' AND streaming_url IS NULL AND streaming_key IS NULL AND public_url IS NULL
            OR
        -- For 'custom' the streaming endpoint is needed but the public URL is optional
        kind = 'custom' AND streaming_url IS NOT NULL AND streaming_key IS NOT NULL
            OR
            -- For 'provider' the streaming endpoint is fetched from the service level and the public URL is optional
        kind = 'provider' AND streaming_url IS NULL AND streaming_key IS NOT NULL
    );
