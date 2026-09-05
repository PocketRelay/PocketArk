CREATE TABLE IF NOT EXISTS "currency" (
    "user_id" INTEGER NOT NULL,
    "ty" INTEGER NOT NULL,
    "balance" BIGINT NOT NULL,
    PRIMARY KEY ("user_id", "ty"),
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS "idx-currency-uid" ON "currency" ("user_id");
