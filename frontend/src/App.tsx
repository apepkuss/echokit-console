import { useState, useCallback, useEffect } from 'react';
import { BrowserRouter, Routes, Route, useNavigate, useLocation, Navigate } from 'react-router-dom';
import { ConfigProvider, Layout, Typography, Button, theme, Space, Dropdown, Avatar, Spin } from 'antd';
import { MenuFoldOutlined, PlusOutlined, LaptopOutlined, CloudServerOutlined, UserOutlined, SettingOutlined, LogoutOutlined, MobileOutlined } from '@ant-design/icons';
import type { MenuProps } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { Dashboard, Device, DeviceActivation, Login, Register, Settings } from './pages';
import { DeployForm, ProtectedRoute } from './components';
import { useAuthStore } from './stores';
import './App.css';

const { Header, Content, Sider } = Layout;

const MIN_WIDTH = 320;
const MAX_WIDTH = 600;
const DEFAULT_WIDTH = 400;

function AppLayout() {
  const [collapsed, setCollapsed] = useState(true);
  const [siderWidth, setSiderWidth] = useState(DEFAULT_WIDTH);
  const [isResizing, setIsResizing] = useState(false);
  const { token } = theme.useToken();
  const navigate = useNavigate();
  const location = useLocation();
  const { user, logout } = useAuthStore();

  const isServerPage = location.pathname === '/server';
  const isActivatePage = location.pathname === '/activate';
  const isDevicePage = location.pathname === '/devices';

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);

    const startX = e.clientX;
    const startWidth = siderWidth;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = e.clientX - startX;
      const newWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, startWidth + delta));
      setSiderWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [siderWidth]);

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  const userMenuItems: MenuProps['items'] = [
    {
      key: 'user-info',
      label: (
        <div style={{ padding: '4px 0' }}>
          <div style={{ fontWeight: 500 }}>{user?.name || '未设置名称'}</div>
          <div style={{ fontSize: 12, color: '#888' }}>{user?.email}</div>
        </div>
      ),
      disabled: true,
    },
    { type: 'divider' },
    {
      key: 'settings',
      icon: <SettingOutlined />,
      label: '账户设置',
      onClick: () => navigate('/settings'),
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: '退出登录',
      onClick: handleLogout,
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header
        style={{
          display: 'flex',
          alignItems: 'center',
          background: '#001529',
          padding: '0 16px',
          gap: 24,
        }}
      >
        <Typography.Title level={4} style={{ color: '#fff', margin: 0 }}>
          EchoKit Console
        </Typography.Title>
        <Space style={{ flex: 1 }}>
          <Button
            type={isActivatePage ? 'primary' : 'text'}
            icon={<MobileOutlined />}
            onClick={() => navigate('/activate')}
            style={{ color: isActivatePage ? undefined : '#fff' }}
          >
            激活设备
          </Button>
          <Button
            type={isDevicePage ? 'primary' : 'text'}
            icon={<LaptopOutlined />}
            onClick={() => navigate('/devices')}
            style={{ color: isDevicePage ? undefined : '#fff' }}
          >
            设备
          </Button>
          <Button
            type={isServerPage ? 'primary' : 'text'}
            icon={<CloudServerOutlined />}
            onClick={() => navigate('/server')}
            style={{ color: isServerPage ? undefined : '#fff' }}
          >
            服务器
          </Button>
        </Space>
        <Dropdown menu={{ items: userMenuItems }} placement="bottomRight">
          <Button type="text" style={{ color: '#fff', padding: '4px 8px' }}>
            <Space>
              <Avatar size="small" icon={<UserOutlined />} />
              <span>{user?.name || user?.email?.split('@')[0] || '用户'}</span>
            </Space>
          </Button>
        </Dropdown>
      </Header>
      <Layout>
        {isServerPage && (
          <Sider
            width={collapsed ? 50 : siderWidth}
            collapsedWidth={50}
            collapsed={collapsed}
            theme="light"
            style={{
              background: token.colorBgContainer,
              borderRight: `1px solid ${token.colorBorderSecondary}`,
              overflow: 'hidden',
              height: 'calc(100vh - 64px)',
              position: 'sticky',
              top: 64,
              left: 0,
              transition: collapsed ? 'width 0.2s' : 'none',
            }}
          >
            <div style={{ display: 'flex', height: '100%' }}>
              <div style={{ flex: 1, overflow: 'auto' }}>
                {collapsed ? (
                  <div style={{ padding: '16px 0', textAlign: 'center' }}>
                    <Button
                      type="primary"
                      icon={<PlusOutlined />}
                      onClick={() => setCollapsed(false)}
                      title="展开部署面板"
                    />
                  </div>
                ) : (
                  <div style={{ padding: 16 }}>
                    <div style={{ marginBottom: 16, display: 'flex', justifyContent: 'flex-end' }}>
                      <Button
                        type="text"
                        icon={<MenuFoldOutlined />}
                        onClick={() => setCollapsed(true)}
                        title="收起"
                      />
                    </div>
                    <DeployForm onSuccess={() => {}} />
                  </div>
                )}
              </div>
              {!collapsed && (
                <div
                  onMouseDown={handleMouseDown}
                  style={{
                    width: 4,
                    cursor: 'col-resize',
                    background: isResizing ? token.colorPrimary : 'transparent',
                    transition: 'background 0.2s',
                    flexShrink: 0,
                  }}
                  onMouseEnter={(e) => {
                    if (!isResizing) {
                      e.currentTarget.style.background = token.colorBorderSecondary;
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (!isResizing) {
                      e.currentTarget.style.background = 'transparent';
                    }
                  }}
                />
              )}
            </div>
          </Sider>
        )}
        <Content style={{ background: '#f5f5f5' }}>
          <Routes>
            <Route path="/" element={<Navigate to="/devices" replace />} />
            <Route path="/devices" element={<Device />} />
            <Route path="/activate" element={<DeviceActivation />} />
            <Route path="/server" element={<Dashboard />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}

function AuthInitializer({ children }: { children: React.ReactNode }) {
  const { initialize, isInitialized } = useAuthStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (!isInitialized) {
    return (
      <div
        style={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <Spin size="large" tip="加载中..." />
      </div>
    );
  }

  return <>{children}</>;
}

function AppRoutes() {
  const { isAuthenticated } = useAuthStore();

  return (
    <Routes>
      {/* 公开路由 */}
      <Route
        path="/login"
        element={isAuthenticated ? <Navigate to="/" replace /> : <Login />}
      />
      <Route
        path="/register"
        element={isAuthenticated ? <Navigate to="/" replace /> : <Register />}
      />

      {/* 受保护的路由 */}
      <Route
        path="/*"
        element={
          <ProtectedRoute>
            <AppLayout />
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}

function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <BrowserRouter>
        <AuthInitializer>
          <AppRoutes />
        </AuthInitializer>
      </BrowserRouter>
    </ConfigProvider>
  );
}

export default App;
