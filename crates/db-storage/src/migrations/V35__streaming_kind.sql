-- Add new streaming kind alternatives.
-- We keep this in a separate migration file to avoid errors like this later on:
--   unsafe use of new value of enum type streaming_kind
--   HINT: New enum values must be committed before they can be used.
ALTER TYPE streaming_kind ADD VALUE 'builtin' BEFORE 'custom';
ALTER TYPE streaming_kind ADD VALUE 'provider' AFTER 'custom';
