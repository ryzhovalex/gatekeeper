-- Your SQL goes here
CREATE TABLE "user"(
	"id" INTEGER NOT NULL PRIMARY KEY,
	"hpassword" VARCHAR NOT NULL,
	"username" VARCHAR NOT NULL,
	"firstname" VARCHAR,
	"patronym" VARCHAR,
	"surname" VARCHAR,
	"rt" VARCHAR
);
CREATE TABLE "user_change"(
	"id" INTEGER NOT NULL PRIMARY KEY,
	"created" DOUBLE PRECISION NOT NULL,
	"action" VARCHAR NOT NULL,
	"user_id" INTEGER NOT NULL,
	FOREIGN KEY ("user_id") REFERENCES "user"("id")
);

