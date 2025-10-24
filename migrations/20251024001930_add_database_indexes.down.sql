
-- Chapter table indexes
DROP INDEX IF EXISTS idx_chapter_book_num;
DROP INDEX IF EXISTS idx_chapter_created_at;
DROP INDEX IF EXISTS idx_chapter_chapter_num;
DROP INDEX IF EXISTS idx_chapter_book_id;

-- Bookmark table indexes
DROP INDEX IF EXISTS idx_bookmark_user_book;
DROP INDEX IF EXISTS idx_bookmark_created_at;
DROP INDEX IF EXISTS idx_bookmark_book_id;
DROP INDEX IF EXISTS idx_bookmark_user_id;

-- BookGenre table indexes
DROP INDEX IF EXISTS idx_bookgenre_genre_id;
DROP INDEX IF EXISTS idx_bookgenre_book_id;

-- Book table indexes
DROP INDEX IF EXISTS idx_book_popular_language;
DROP INDEX IF EXISTS idx_book_updated_at;
DROP INDEX IF EXISTS idx_book_created_at;
DROP INDEX IF EXISTS idx_book_release_date;
DROP INDEX IF EXISTS idx_book_status;
DROP INDEX IF EXISTS idx_book_language;
DROP INDEX IF EXISTS idx_book_popular;
DROP INDEX IF EXISTS idx_book_author;
DROP INDEX IF EXISTS idx_book_title;

-- Genre table indexes
DROP INDEX IF EXISTS idx_genre_created_at;
DROP INDEX IF EXISTS idx_genre_title;

-- User table indexes
DROP INDEX IF EXISTS idx_user_last_login;
DROP INDEX IF EXISTS idx_user_created_at;
DROP INDEX IF EXISTS idx_user_is_admin;
DROP INDEX IF EXISTS idx_user_val_token;
DROP INDEX IF EXISTS idx_user_token;
DROP INDEX IF EXISTS idx_user_email;
DROP INDEX IF EXISTS idx_user_username;