-- Drop tables in reverse order (respecting foreign key constraints)
DROP TABLE IF EXISTS "Chapter";
DROP TABLE IF EXISTS "Bookmark";
DROP TABLE IF EXISTS "BookGenre";
DROP TABLE IF EXISTS "Book";
DROP TABLE IF EXISTS "Genre";
DROP TABLE IF EXISTS "User";

-- Drop ENUM types
DROP TYPE IF EXISTS "Status";
DROP TYPE IF EXISTS "Language";