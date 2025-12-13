-- 为现有表添加 user_id 字段，实现数据按用户隔离

-- 1. devices 表添加 user_id 字段（NOT NULL，每个设备必须属于一个用户）
ALTER TABLE devices ADD COLUMN IF NOT EXISTS user_id VARCHAR(64);

-- 添加外键约束
ALTER TABLE devices ADD CONSTRAINT fk_devices_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_devices_user ON devices(user_id);

-- 注释
COMMENT ON COLUMN devices.user_id IS '所属用户 ID';

-- 2. containers 表添加 user_id 字段（可为 NULL，NULL 表示全局共享服务器）
ALTER TABLE containers ADD COLUMN IF NOT EXISTS user_id VARCHAR(64);

-- 添加外键约束
ALTER TABLE containers ADD CONSTRAINT fk_containers_user
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_containers_user ON containers(user_id);

-- 注释
COMMENT ON COLUMN containers.user_id IS '所属用户 ID（NULL 表示全局共享服务器）';

-- 注意：官方服务器（official-dallas）的 user_id 保持 NULL，表示全局共享
