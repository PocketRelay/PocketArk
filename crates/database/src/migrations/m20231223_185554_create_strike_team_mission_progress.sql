CREATE TABLE IF NOT EXISTS "strike_team_mission_progress" (
    "user_id" INTEGER NOT NULL,
    "strike_team_id" INTEGER NOT NULL UNIQUE,
    "mission_id" INTEGER NOT NULL,
    "user_mission_state" INTEGER NOT NULL,
    "seen" BOOLEAN NOT NULL DEFAULT 0,
    "completed" BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY ("user_id", "mission_id"),
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("mission_id") REFERENCES "strike_team_missions" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("strike_team_id") REFERENCES "strike_teams" ("id") ON DELETE CASCADE
);
