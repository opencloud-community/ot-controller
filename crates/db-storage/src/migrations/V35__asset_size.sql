ALTER TABLE assets
	ADD COLUMN file_size bigint CHECK (file_size >= 0);

UPDATE
	assets
SET
	file_size = 0
WHERE
	file_size IS NULL;

ALTER TABLE assets ALTER COLUMN file_size SET NOT NULL;
