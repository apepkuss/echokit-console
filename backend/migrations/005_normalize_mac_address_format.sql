-- 统一 MAC 地址格式为 12 位小写十六进制（无冒号）
--
-- 旧格式: "98:A3:16:F0:B1:E5" (大写带冒号)
-- 新格式: "98a316f0b1e5" (小写无冒号)
--
-- 此迁移将 devices 表中的 device_id 和 mac_address 字段转换为新格式

-- 更新 device_id 字段
UPDATE devices
SET device_id = LOWER(REPLACE(device_id, ':', ''))
WHERE device_id LIKE '%:%';

-- 更新 mac_address 字段
UPDATE devices
SET mac_address = LOWER(REPLACE(mac_address, ':', ''))
WHERE mac_address LIKE '%:%';

-- 更新注释
COMMENT ON COLUMN devices.device_id IS '设备唯一标识符（12位小写十六进制，如 98a316f0b1e5）';
COMMENT ON COLUMN devices.mac_address IS 'WiFi MAC 地址（12位小写十六进制，如 98a316f0b1e5）';
