-- Add migration script here
-- Add indexes for performance optimization

-- User table indexes
CREATE INDEX IF NOT EXISTS idx_user_username ON "User"(username);
CREATE INDEX IF NOT EXISTS idx_user_email ON "User"(email);
CREATE INDEX IF NOT EXISTS idx_user_token ON "User"(token);
CREATE INDEX IF NOT EXISTS idx_user_val_token ON "User"(val_token);
CREATE INDEX IF NOT EXISTS idx_user_is_admin ON "User"(is_admin);
CREATE INDEX IF NOT EXISTS idx_user_created_at ON "User"(created_at);
CREATE INDEX IF NOT EXISTS idx_user_last_login ON "User"(last_login);

-- Genre table indexes
CREATE INDEX IF NOT EXISTS idx_genre_title ON "Genre"(title);
CREATE INDEX IF NOT EXISTS idx_genre_created_at ON "Genre"(created_at);

-- Book table indexes
CREATE INDEX IF NOT EXISTS idx_book_title ON "Book"(title);
CREATE INDEX IF NOT EXISTS idx_book_author ON "Book"(author);
CREATE INDEX IF NOT EXISTS idx_book_popular ON "Book"(popular);
CREATE INDEX IF NOT EXISTS idx_book_language ON "Book"(language);
CREATE INDEX IF NOT EXISTS idx_book_status ON "Book"(status);
CREATE INDEX IF NOT EXISTS idx_book_release_date ON "Book"(release_date);
CREATE INDEX IF NOT EXISTS idx_book_created_at ON "Book"(created_at);
CREATE INDEX IF NOT EXISTS idx_book_updated_at ON "Book"(updated_at);
CREATE INDEX IF NOT EXISTS idx_book_popular_language ON "Book"(popular, language);

-- BookGenre table indexes
CREATE INDEX IF NOT EXISTS idx_bookgenre_book_id ON "BookGenre"(book_id);
CREATE INDEX IF NOT EXISTS idx_bookgenre_genre_id ON "BookGenre"(genre_id);

-- Bookmark table indexes
CREATE INDEX IF NOT EXISTS idx_bookmark_user_id ON "Bookmark"(user_id);
CREATE INDEX IF NOT EXISTS idx_bookmark_book_id ON "Bookmark"(book_id);
CREATE INDEX IF NOT EXISTS idx_bookmark_created_at ON "Bookmark"(created_at);
CREATE INDEX IF NOT EXISTS idx_bookmark_user_book ON "Bookmark"(user_id, book_id);

-- Chapter table indexes
CREATE INDEX IF NOT EXISTS idx_chapter_book_id ON "Chapter"(book_id);
CREATE INDEX IF NOT EXISTS idx_chapter_chapter_num ON "Chapter"(chapter_num);
CREATE INDEX IF NOT EXISTS idx_chapter_created_at ON "Chapter"(created_at);
CREATE INDEX IF NOT EXISTS idx_chapter_book_num ON "Chapter"(book_id, chapter_num);