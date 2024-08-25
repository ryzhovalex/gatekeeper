-- Your SQL goes here
CREATE TABLE "appuser"(
	"id" SERIAL PRIMARY KEY,
	"hpassword" VARCHAR NOT NULL,
	"username" VARCHAR NOT NULL UNIQUE,
	"firstname" VARCHAR,
	"patronym" VARCHAR,
	"surname" VARCHAR,
	"rt" VARCHAR
);
CREATE TABLE "user_change"(
	"id" SERIAL PRIMARY KEY,
	"created" DOUBLE PRECISION NOT NULL,
	"action" VARCHAR NOT NULL,
	"user_id" SERIAL NOT NULL,
	FOREIGN KEY ("user_id") REFERENCES "appuser"("id")
);

