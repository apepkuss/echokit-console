-- 创建容器表（用于存储 EchoKit Server 信息）
CREATE TABLE IF NOT EXISTS containers (
    -- 主键：容器 ID
    id VARCHAR(64) PRIMARY KEY,

    -- 基本信息
    name VARCHAR(255) NOT NULL,

    -- 服务器配置
    host VARCHAR(255) NOT NULL,          -- 主机地址（如 localhost 或 dallas.echokit.dev）
    port INTEGER,                        -- 端口号（NULL 表示使用默认 443 或 80）
    use_tls BOOLEAN NOT NULL DEFAULT false, -- 是否使用 TLS (wss://)

    -- 元数据
    is_default BOOLEAN NOT NULL DEFAULT false, -- 是否为默认服务器
    is_external BOOLEAN NOT NULL DEFAULT false, -- 是否为外部服务器

    -- 时间戳
    created_at BIGINT NOT NULL,
    updated_at BIGINT,

    -- 约束
    CONSTRAINT chk_port CHECK (port IS NULL OR (port > 0 AND port <= 65535))
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_containers_is_default ON containers(is_default);

-- 插入默认官方服务器
INSERT INTO containers (
    id,
    name,
    host,
    port,
    use_tls,
    is_default,
    is_external,
    created_at
) VALUES (
    'official-dallas',
    'EchoKit Official (Dallas)',
    'dallas.echokit.dev',
    80,
    false,
    true,
    true,
    EXTRACT(EPOCH FROM NOW())::BIGINT
);

-- 注释
COMMENT ON TABLE containers IS 'EchoKit Server 容器/服务器配置表';
COMMENT ON COLUMN containers.id IS '容器/服务器唯一标识符';
COMMENT ON COLUMN containers.name IS '容器/服务器名称';
COMMENT ON COLUMN containers.host IS '主机地址';
COMMENT ON COLUMN containers.port IS '端口号';
COMMENT ON COLUMN containers.use_tls IS '是否使用 TLS (wss://)';
COMMENT ON COLUMN containers.is_default IS '是否为默认服务器';
COMMENT ON COLUMN containers.is_external IS '是否为外部服务器（非本地Docker）';
COMMENT ON COLUMN containers.created_at IS '创建时间（Unix 时间戳）';
COMMENT ON COLUMN containers.updated_at IS '最后更新时间（Unix 时间戳）';
