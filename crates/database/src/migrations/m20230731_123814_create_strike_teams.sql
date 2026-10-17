CREATE TABLE IF NOT EXISTS "strike_teams" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "icon" TEXT NOT NULL,
    "level" INTEGER NOT NULL,
    "xp" TEXT NOT NULL,
    "equipment" TEXT,
    "positive_traits" TEXT NOT NULL,
    "negative_traits" TEXT NOT NULL,
    "out_of_date" BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "idx-strike-team-uid" ON "strike_teams" ("user_id");
