CREATE TABLE IF NOT EXISTS "strike_team_missions" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL UNIQUE,
    "descriptor" TEXT NOT NULL,
    "mission_type" TEXT NOT NULL,
    "accessibility" INTEGER NOT NULL,
    "waves" TEXT NOT NULL,
    "tags" TEXT NOT NULL,
    "static_modifiers" TEXT NOT NULL,
    "dynamic_modifiers" TEXT NOT NULL,
    "rewards" TEXT NOT NULL,
    "custom_attributes" TEXT NOT NULL,
    "start_seconds" INTEGER NOT NULL,
    "end_seconds" INTEGER NOT NULL,
    "sp_length_seconds" INTEGER NOT NULL
);
