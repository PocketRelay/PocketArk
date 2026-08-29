CREATE TABLE IF NOT EXISTS "inventory_items" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "item_id" TEXT NOT NULL UNIQUE,
    "user_id" INTEGER NOT NULL,
    "definition_name" TEXT NOT NULL,
    "stack_size" INTEGER NOT NULL,
    "seen" BOOLEAN NOT NULL DEFAULT 0,
    "instance_attributes" TEXT NOT NULL,
    "created" TEXT NOT NULL,
    "last_grant" TEXT NOT NULL,
    "earned_by" TEXT NOT NULL,
    "restricted" BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "idx-item-uid-def" ON "inventory_items" ("user_id", "definition_name");
