-- Add fcm_token column to User table for push notifications
ALTER TABLE "User" ADD COLUMN fcm_token TEXT;

-- Create index for faster lookup by fcm_token
CREATE INDEX idx_user_fcm_token ON "User"(fcm_token) WHERE fcm_token IS NOT NULL;
