-- Create ENUM types
CREATE TYPE "Language" AS ENUM (
    'English',
    'Japanese',
    'Korean'
);

CREATE TYPE "Status" AS ENUM (
    'Ongoing',
    'Completed',
    'Drop'
);

-- Create tables
CREATE TABLE "User" (
                        id TEXT PRIMARY KEY,
                        profile_pic TEXT,
                        username TEXT NOT NULL UNIQUE,
                        email TEXT NOT NULL UNIQUE,
                        name TEXT,
                        password TEXT NOT NULL,
                        bio TEXT,
                        created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP(3) NOT NULL,
                        last_login TIMESTAMP(3),
                        token TEXT DEFAULT '',
                        val_token TEXT DEFAULT '',
                        is_admin BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE "Genre" (
                         id TEXT PRIMARY KEY,
                         title TEXT NOT NULL,
                         description TEXT NOT NULL,
                         created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                         updated_at TIMESTAMP(3) NOT NULL
);

CREATE TABLE "Book" (
                        asset TEXT,
                        id TEXT PRIMARY KEY,
                        cover TEXT NOT NULL,
                        title TEXT NOT NULL,
                        author TEXT NOT NULL,
                        description TEXT NOT NULL,
                        created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at TIMESTAMP(3) NOT NULL,
                        popular BOOLEAN NOT NULL DEFAULT false,
                        language "Language" NOT NULL DEFAULT 'Korean',
                        release_date INTEGER,
                        status "Status" NOT NULL DEFAULT 'Ongoing'
);

CREATE TABLE "BookGenre" (
                             book_id TEXT NOT NULL,
                             genre_id TEXT NOT NULL,
                             PRIMARY KEY (book_id, genre_id),
                             FOREIGN KEY (book_id) REFERENCES "Book"(id) ON DELETE RESTRICT ON UPDATE CASCADE,
                             FOREIGN KEY (genre_id) REFERENCES "Genre"(id) ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE TABLE "Bookmark" (
                            id TEXT PRIMARY KEY,
                            user_id TEXT NOT NULL,
                            book_id TEXT NOT NULL,
                            created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                            updated_at TIMESTAMP(3) NOT NULL,
                            FOREIGN KEY (user_id) REFERENCES "User"(id) ON DELETE RESTRICT ON UPDATE CASCADE,
                            FOREIGN KEY (book_id) REFERENCES "Book"(id) ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE TABLE "Chapter" (
                           id TEXT PRIMARY KEY,
                           title TEXT NOT NULL,
                           book_id TEXT NOT NULL,
                           description TEXT NOT NULL,
                           created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
                           updated_at TIMESTAMP(3) NOT NULL,
                           content TEXT NOT NULL,
                           chapter_num INTEGER NOT NULL,
                           FOREIGN KEY (book_id) REFERENCES "Book"(id) ON DELETE RESTRICT ON UPDATE CASCADE
);

-- Insert sample data for Genres
INSERT INTO "Genre" (id, title, description, created_at, updated_at) VALUES
                                                                         ('cm7ody0bf0001d9xs701z1q1x', 'fantasy', 'fantasy', '2025-02-28 06:21:36.411', '2025-02-28 06:21:36.411'),
                                                                         ('cm7oelnrh0000d9r4dcz098h6', 'romance', 'romance', '2025-02-28 06:39:59.886', '2025-02-28 06:39:59.886'),
                                                                         ('cm7r9bql60000kq0xtlyhvr7d', 'action', 'action', '2025-03-02 06:35:37.435', '2025-03-02 06:35:37.435'),
                                                                         ('cm7s9hz410000gw0xdjwuon1t', 'adventure', 'adventure', '2025-03-02 23:28:14.593', '2025-03-02 23:29:27.146'),
                                                                         ('cm7s9jo8q0001gw0xy0wfigvx', 'mystery', 'mystery', '2025-03-02 23:29:33.819', '2025-03-02 23:29:33.819'),
                                                                         ('cm7s9k9rf0002gw0xz8x6ih8k', 'comedy', 'comedy', '2025-03-02 23:30:01.707', '2025-03-02 23:30:01.707');

-- Insert sample data for Users
INSERT INTO "User" (id, profile_pic, username, email, name, password, bio, created_at, updated_at, last_login, token, val_token, is_admin) VALUES
                                                                                                                                               ('cm7odusv20000d9xsjx7aq15p', 'https://little-wood-d8cd.tigfiragnafatur1933.workers.dev/1743605656159-111233.webp', 'wign', 'wign625@gmail.com', 'wign', '$2b$10$tIvj5cKe4RKgaDb90Jma9.CU9BQzk6SQ3wLIunvh6PQPFlO1jvg4K', NULL, '2025-02-28 06:19:06.782', '2025-10-08 03:43:31.142', '2025-10-08 03:43:31.141', '14b58f25-e53c-4ab8-9d3e-f9e467051c3d', '', true),
                                                                                                                                               ('cm7pv079r0000gj0xwe42re1v', 'https://little-wood-d8cd.tigfiragnafatur1933.workers.dev/1741666539733-111233.webp', 'tigfir', 'tigfiragnafatur1933@gmail.com', 'tigfir', '$2b$10$IF00iGjfIyvWWqCirQfbqeGEuBnYYMotudTeg7pJu3mgRSH7NcstK', NULL, '2025-03-01 07:06:58.383', '2025-10-08 06:46:24.321', NULL, '', '2e50dd6d-3000-4658-8096-464f1c2d9a3b', false);

-- Insert sample data for Books
INSERT INTO "Book" (asset, id, cover, title, author, description, created_at, updated_at, popular, language, realase_date, status) VALUES
                                                                                                                                       (NULL, 'cm7oer4zi0001d9r45f8ylp9f', 'https://cdn.othinus.cloud/1741661245932-453857926_458096460473172_3740908029410987569_n.webp', 'Modern Villainess: It''s Not Easy Building a Corporate Empire Before the Crash', 'Futsukaichi Tofurou', 'I was reincarnated as a villainous daughter in an otome game based on the modern world.', '2025-02-28 06:44:14.031', '2025-07-11 01:19:02.55', false, 'English', 2025, 'Ongoing'),
                                                                                                                                       (NULL, 'cm7ofwxf20002d9r4akimvgt8', 'https://cdn.othinus.cloud/1741660754085-Fake-Saint-Of-The-Year.webp', 'Fake Saint Of The Year', 'Wakaranai Man', '[The Eternal Scattering Flowers: Fiore Caduto Eterna] is a game ill-suited of being known as a galge.', '2025-02-28 07:16:45.059', '2025-07-11 01:19:31.883', false, 'Korean', 2024, 'Ongoing'),
                                                                                                                                       (NULL, 'cm7q95t8l0004i60xfrydi1nj', 'https://cdn.othinus.cloud/1741661325433-Syl.webp', 'The Villain Wants to Live', 'Jee Gab Song', 'The mid-level boss of my company''s AAA-game. Deculein, a villain who dies in 999 out of the 1000 playthroughs.', '2025-03-01 13:43:14.735', '2025-07-11 01:20:04.897', false, 'Korean', 2021, 'Ongoing'),
                                                                                                                                       (NULL, 'cm7qal7p70007i60xtalkjwu5', 'https://cdn.othinus.cloud/1741660553874-121c842eed7d511eff13c323ae5072d2_551458_ori.webp', 'Childhood Friend of the Zenith', 'Nara Yeo', E'Gu Yangcheon committed a lot of evil while serving the Heavenly Demon.\nFilled with burden of his past crimes, he embarks on a new journey.', '2025-03-01 14:23:12.804', '2025-07-11 01:20:50.757', false, 'Korean', 2021, 'Ongoing'),
                                                                                                                                       (NULL, 'cm825nsdv0000ff0xbceqg7f6', 'https://cdn.othinus.cloud/1741659454525-I-Became-a-Law-School-Genius.webp', 'I Became a Law School Genius', '노칼', 'Aku berhasil melewati tahap kedua ujian bar, tetapi serangkaian kemalangan menghalangiku.', '2025-03-09 21:38:29.017', '2025-07-11 01:23:58.494', false, 'Korean', 2024, 'Ongoing'),
                                                                                                                                       (NULL, 'cm82d3jj40002h00x9cwx88ne', 'https://cdn.othinus.cloud/1741656953052-Deep-Sea-Fish-Hunting-Specialty-Broadcast.webp', 'Deep Sea Fish Hunting Specialty Broadcast', '준호킴', 'Suatu hari, sebuah Menara muncul, disertai dengan proklamasi dari Tuhan.', '2025-03-10 01:06:40.873', '2025-07-11 01:24:21.579', false, 'Korean', 2024, 'Ongoing'),
                                                                                                                                       (NULL, 'cm8h6q4zp0000eg0xc2dlj1e2', 'https://cdn.othinus.cloud/1742465087619-b4f1e80f775262b0ead16c605020c6d8_384637_ori.webp', 'Rather Than Zhang Ran, Zhang Bao''s Eldest Daughter', '저택성', 'A Bonapartist history scholar who had dreamed of becoming Jean Lannes.', '2025-03-20 10:04:50.921', '2025-07-11 01:24:40.518', false, 'Korean', 2021, 'Ongoing'),
                                                                                                                                       (NULL, 'cm7rseuw00002e20xrwbb9w2l', 'https://cdn.othinus.cloud/1741660463503-Game-of-the-World-Tree%20(1).webp', 'Game of the World Tree', 'Bang', 'In The Kingdom of Elves, the War of Gods 1000 years ago caused the fall of the World Tree.', '2025-03-02 15:29:55.658', '2025-07-11 01:21:10.681', false, 'Korean', 2021, 'Ongoing'),
                                                                                                                                       (NULL, 'cm7w3r5tf0005h20xxcnsry4q', 'https://cdn.othinus.cloud/1741660326197-DSE-C-1.webp', 'Deep Sea Embers', 'Yuan Tong', 'On that day, the fog blocked everything.', '2025-03-05 15:58:30.176', '2025-07-11 01:23:34.89', false, 'Japanese', 2024, 'Ongoing');

-- Insert BookGenre relationships
INSERT INTO "BookGenre" (book_id, genre_id) VALUES
                                                ('cm7oer4zi0001d9r45f8ylp9f', 'cm7oelnrh0000d9r4dcz098h6'),
                                                ('cm7ofwxf20002d9r4akimvgt8', 'cm7ody0bf0001d9xs701z1q1x'),
                                                ('cm7q95t8l0004i60xfrydi1nj', 'cm7ody0bf0001d9xs701z1q1x'),
                                                ('cm7qal7p70007i60xtalkjwu5', 'cm7ody0bf0001d9xs701z1q1x'),
                                                ('cm7qal7p70007i60xtalkjwu5', 'cm7oelnrh0000d9r4dcz098h6'),
                                                ('cm7qal7p70007i60xtalkjwu5', 'cm7r9bql60000kq0xtlyhvr7d');

-- Insert sample Chapters
INSERT INTO "Chapter" (id, title, book_id, description, created_at, updated_at, content, chapter_num) VALUES
                                                                                                          ('cm7pqh4680001d9usfi8la1y7', 'Aku Bereinkarnasi sebagai Santo Palsu', 'cm7ofwxf20002d9r4akimvgt8', 'prolog', '2025-03-01 05:00:09.434', '2025-03-01 05:00:09.434', '<p>Di sebuah hutan yang sunyi, sebuah tragedi sedang terjadi...</p>', 1),
                                                                                                          ('cm7rf046e001kkq0x3k758vll', '(Fake) Saint Elrise', 'cm7ofwxf20002d9r4akimvgt8', '(Fake) Saint Elrise', '2025-03-02 09:14:32.87', '2025-03-02 09:14:32.87', '<p>Noo… Ini bikin aku down.</p>', 2),
                                                                                                          ('cm81nvkdj0001gt0x58bp7wup', '15 September 2008', 'cm7oer4zi0001d9r45f8ylp9f', 'prolog', '2025-03-09 13:20:38.887', '2025-03-09 13:20:38.887', '<p>Aku berdiri di lantai teratas Menara Teia di Daiba, Tokyo.</p>', 1),
                                                                                                          ('cm8261n8l0002ff0xxkznc6c2', 'Chapter 1', 'cm825nsdv0000ff0xbceqg7f6', 'Chapter 1', '2025-03-09 21:49:15.064', '2025-03-09 21:49:15.064', '<p><strong>—Ingatlah ini.</strong></p>', 1),
                                                                                                          ('cm82dxrog0004h00x6q4bj0zy', 'Chapter 2', 'cm825nsdv0000ff0xbceqg7f6', 'Chapter 2', '2025-03-10 01:30:11.109', '2025-03-10 01:30:11.109', '<p>Episode pertama dari webtoon</p>', 2),
                                                                                                          ('cm8320ptp0003gv0xtx95u63u', 'Yu Parang (1)', 'cm82d3jj40002h00x9cwx88ne', 'Yu Parang (1)', '2025-03-10 12:44:19.455', '2025-03-10 12:44:19.455', '<p><strong>"Aku tidak ingin bekerja…"</strong></p>', 1),
                                                                                                          ('cm832iawb0005gv0xpdxyuwav', 'Kabut Tebal Hari Itu', 'cm7w3r5tf0005h20xxcnsry4q', 'Kabut Tebal Hari Itu', '2025-03-10 12:58:00.491', '2025-03-10 12:58:00.491', '<p>Hari itu, kabut tebal menyelimuti dunia...</p>', 1),
                                                                                                          ('cm832xrvg0007gv0xzsezoznj', 'Chapter 3', 'cm825nsdv0000ff0xbceqg7f6', 'Chapter 3', '2025-03-10 13:10:02.333', '2025-03-10 13:10:02.333', '<p>Sekarang Setelah Kupikirkan Lagi...</p>', 3),
                                                                                                          ('cm848i7t40001i20xwiwvfece', 'Chapter 4', 'cm825nsdv0000ff0xbceqg7f6', 'Chapter 4', '2025-03-11 08:33:39.788', '2025-03-11 08:33:39.788', '<p><em>"Itu adalah kesalahan fakta...</em></p>', 4),
                                                                                                          ('cm849hkv50006i20xdir9g3fp', 'Chapter 5', 'cm825nsdv0000ff0xbceqg7f6', 'Chapter 5', '2025-03-11 09:01:10.241', '2025-03-11 09:01:10.241', '<p><em>"Itu adalah kesalahan fakta terkait pembenaran...</em></p>', 5),
                                                                                                          ('cm8h6ymt20002eg0xmcv6y561', 'Chapter 1', 'cm8h6q4zp0000eg0xc2dlj1e2', 'Chapter 1', '2025-03-20 10:11:27.351', '2025-03-20 10:11:27.351', '<p><strong>Roh Gunung.</strong></p>', 1),
                                                                                                          ('cm8h7moz30004eg0xavjklxjk', 'Chapter 2', 'cm8h6q4zp0000eg0xc2dlj1e2', 'Chapter 2', '2025-03-20 10:30:09.904', '2025-03-20 10:30:09.904', '<p>Romance of the Three Kingdoms</p>', 2);