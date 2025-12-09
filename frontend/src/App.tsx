import { useState, useCallback } from 'react';
import { BrowserRouter, Routes, Route, useNavigate, useLocation } from 'react-router-dom';
import { ConfigProvider, Layout, Typography, Button, theme, Space } from 'antd';
import { MenuFoldOutlined, PlusOutlined, LaptopOutlined, CloudServerOutlined } from '@ant-design/icons';
import zhCN from 'antd/locale/zh_CN';
import { Dashboard, Device } from './pages';
import { DeployForm } from './components';
import './App.css';

const { Header, Content, Sider } = Layout;

const MIN_WIDTH = 320;
const MAX_WIDTH = 600;
const DEFAULT_WIDTH = 400;

function AppLayout() {
  const [collapsed, setCollapsed] = useState(false);
  const [siderWidth, setSiderWidth] = useState(DEFAULT_WIDTH);
  const [isResizing, setIsResizing] = useState(false);
  const { token } = theme.useToken();
  const navigate = useNavigate();
  const location = useLocation();

  const isServerPage = location.pathname === '/server';

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
        <Space>
          <Button
            type={!isServerPage ? 'primary' : 'text'}
            icon={<LaptopOutlined />}
            onClick={() => navigate('/')}
            style={{ color: !isServerPage ? undefined : '#fff' }}
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
            <Route path="/" element={<Device />} />
            <Route path="/server" element={<Dashboard />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}

function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <BrowserRouter>
        <AppLayout />
      </BrowserRouter>
    </ConfigProvider>
  );
}

export default App;
