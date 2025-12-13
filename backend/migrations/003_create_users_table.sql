-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    -- 主键：用户唯一标识符
    id VARCHAR(64) PRIMARY KEY,

    -- 认证信息
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,

    -- 用户信息
    name VARCHAR(255),

    -- 时间戳（Unix 秒级时间戳）
    created_at BIGINT NOT NULL,
    updated_at BIGINT
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- 注释
COMMENT ON TABLE users IS 'EchoKit Console 用户表';
COMMENT ON COLUMN users.id IS '用户唯一标识符';
COMMENT ON COLUMN users.email IS '用户邮箱（用于登录）';
COMMENT ON COLUMN users.password_hash IS '密码哈希值';
COMMENT ON COLUMN users.name IS '用户显示名称';
COMMENT ON COLUMN users.created_at IS '创建时间（Unix 时间戳）';
COMMENT ON COLUMN users.updated_at IS '最后更新时间（Unix 时间戳）';
