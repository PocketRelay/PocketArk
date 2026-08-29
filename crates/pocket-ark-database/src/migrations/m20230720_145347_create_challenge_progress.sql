CREATE TABLE IF NOT EXISTS "challenge_progress" (
    "user_id" INTEGER NOT NULL,
    "challenge_id" TEXT NOT NULL,
    "state" INTEGER NOT NULL,
    "counters" TEXT NOT NULL,
    "times_completed" INTEGER NOT NULL,
    "last_completed" TEXT,
    "first_completed" TEXT,
    "last_changed" TEXT NOT NULL,
    "rewarded" BOOLEAN NOT NULL,
    PRIMARY KEY ("user_id", "challenge_id"),
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
