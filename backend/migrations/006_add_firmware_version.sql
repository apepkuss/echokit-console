-- 添加固件版本字段
ALTER TABLE devices ADD COLUMN firmware_version VARCHAR(32);
