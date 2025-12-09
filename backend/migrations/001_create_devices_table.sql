-- 创建设备表
CREATE TABLE IF NOT EXISTS devices (
    -- 主键：设备唯一标识符（MAC 地址）
    device_id VARCHAR(64) PRIMARY KEY,

    -- 基本信息
    name VARCHAR(255) NOT NULL,
    mac_address VARCHAR(64) NOT NULL UNIQUE,

    -- 绑定关系
    bound_container_id VARCHAR(64),

    -- 时间戳（Unix 秒级时间戳）
    created_at BIGINT NOT NULL,
    last_connected_at BIGINT,
    updated_at BIGINT,

    -- 状态
    status VARCHAR(20) NOT NULL DEFAULT 'unknown',

    -- 约束
    CONSTRAINT chk_status CHECK (status IN ('online', 'offline', 'unknown'))
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(status);
CREATE INDEX IF NOT EXISTS idx_devices_bound_container ON devices(bound_container_id);
CREATE INDEX IF NOT EXISTS idx_devices_created_at ON devices(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_devices_last_connected ON devices(last_connected_at DESC);

-- 注释
COMMENT ON TABLE devices IS 'EchoKit 设备注册表';
COMMENT ON COLUMN devices.device_id IS '设备唯一标识符（MAC 地址）';
COMMENT ON COLUMN devices.name IS '设备名称（用户友好）';
COMMENT ON COLUMN devices.mac_address IS 'WiFi MAC 地址';
COMMENT ON COLUMN devices.bound_container_id IS '绑定的 EchoKit Server 容器 ID';
COMMENT ON COLUMN devices.created_at IS '创建时间（Unix 时间戳）';
COMMENT ON COLUMN devices.last_connected_at IS '最后连接时间（Unix 时间戳）';
COMMENT ON COLUMN devices.updated_at IS '最后更新时间（Unix 时间戳）';
COMMENT ON COLUMN devices.status IS '设备状态：online, offline, unknown';
