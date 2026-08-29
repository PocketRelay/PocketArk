CREATE TABLE IF NOT EXISTS "characters" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "character_id" TEXT NOT NULL UNIQUE,
    "user_id" INTEGER NOT NULL,
    "class_name" TEXT NOT NULL,
    "level" INTEGER NOT NULL,
    "xp" TEXT NOT NULL,
    "promotion" INTEGER NOT NULL,
    "points" TEXT NOT NULL,
    "points_spent" TEXT NOT NULL,
    "points_granted" TEXT NOT NULL,
    "skill_trees" TEXT NOT NULL,
    "attributes" TEXT NOT NULL,
    "bonus" TEXT NOT NULL,
    "equipments" TEXT NOT NULL,
    "customization" TEXT NOT NULL,
    "play_stats" TEXT NOT NULL,
    "last_used" TEXT,
    "promotable" BOOLEAN NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "idx-character-uid-def" ON "characters" ("user_id", "class_name");
