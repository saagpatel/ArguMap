-- Add strength scoring (1-5) to nodes
ALTER TABLE nodes ADD COLUMN strength INTEGER CHECK(strength IS NULL OR (strength >= 1 AND strength <= 5));
