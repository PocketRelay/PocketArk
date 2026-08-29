CREATE TABLE IF NOT EXISTS "shared_data" (
    "user_id" INTEGER NOT NULL PRIMARY KEY,
    "active_character_id" TEXT,
    "shared_stats" TEXT NOT NULL,
    "shared_equipment" TEXT NOT NULL,
    "shared_progression" TEXT NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
