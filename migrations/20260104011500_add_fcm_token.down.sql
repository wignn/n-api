-- Remove fcm_token column from User table
DROP INDEX IF EXISTS idx_user_fcm_token;
ALTER TABLE "User" DROP COLUMN IF EXISTS fcm_token;
