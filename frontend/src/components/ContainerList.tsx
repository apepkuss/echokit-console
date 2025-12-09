import { useEffect, useState } from 'react';
import {
  Table,
  Tag,
  Space,
  Button,
  Typography,
  message,
  Modal,
  Tooltip,
  Dropdown,
} from 'antd';
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  DeleteOutlined,
  CopyOutlined,
  ReloadOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  ExclamationCircleOutlined,
  FileTextOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import type { ContainerInfo, ContainerStatus, HealthStatus, HealthCheckResult } from '../types';
import { useDeployStore } from '../stores';
import { deployService } from '../services/api';

const { Text } = Typography;

const statusColors: Record<ContainerStatus, string> = {
  running: 'green',
  stopped: 'orange',
  error: 'red',
  creating: 'blue',
  starting: 'cyan',
};

const statusLabels: Record<ContainerStatus, string> = {
  running: '运行中',
  stopped: '已停止',
  error: '错误',
  creating: '创建中',
  starting: '启动中',
};

const healthColors: Record<HealthStatus, string> = {
  healthy: 'green',
  unhealthy: 'red',
  unknown: 'default',
};

const healthIcons: Record<HealthStatus, React.ReactNode> = {
  healthy: <CheckCircleOutlined />,
  unhealthy: <CloseCircleOutlined />,
  unknown: <ExclamationCircleOutlined />,
};

const healthLabels: Record<HealthStatus, string> = {
  healthy: '健康',
  unhealthy: '异常',
  unknown: '未知',
};

export function ContainerList() {
  const {
    containers,
    loading,
    fetchContainers,
    stopContainer,
    startContainer,
    deleteContainer,
  } = useDeployStore();

  const [logsModalVisible, setLogsModalVisible] = useState(false);
  const [logsContent, setLogsContent] = useState('');
  const [logsTitle, setLogsTitle] = useState('');
  const [healthCheckLoading, setHealthCheckLoading] = useState<string | null>(null);

  useEffect(() => {
    fetchContainers();
  }, [fetchContainers]);

  const copyWsUrl = (url: string) => {
    navigator.clipboard.writeText(url);
    message.success('WebSocket 地址已复制');
  };

  const showLogs = async (containerId: string, containerName: string) => {
    try {
      const logs = await deployService.getContainerLogs(containerId, 100);
      setLogsContent(logs || '暂无日志');
      setLogsTitle(`${containerName} - 日志`);
      setLogsModalVisible(true);
    } catch {
      message.error('获取日志失败');
    }
  };

  const checkHealth = async (containerId: string) => {
    setHealthCheckLoading(containerId);
    try {
      const health = await deployService.getContainerHealth(containerId);
      if (health.status === 'healthy') {
        message.success('容器健康状态正常');
      } else {
        message.warning(`容器状态异常: ${health.errorMessage || '未知错误'}`);
        if (health.logsTail) {
          setLogsContent(health.logsTail);
          setLogsTitle('错误日志');
          setLogsModalVisible(true);
        }
      }
      fetchContainers(); // 刷新列表
    } catch {
      message.error('健康检查失败');
    } finally {
      setHealthCheckLoading(null);
    }
  };

  const renderHealthStatus = (health: HealthCheckResult | undefined, record: ContainerInfo) => {
    if (!health) {
      return record.status === 'running' ? (
        <Button
          type="link"
          size="small"
          onClick={() => checkHealth(record.id)}
          loading={healthCheckLoading === record.id}
        >
          检查
        </Button>
      ) : (
        <Tag color="default">-</Tag>
      );
    }

    const tooltipContent = health.errorMessage ? (
      <div>
        <div>HTTP 可达: {health.httpReachable ? '是' : '否'}</div>
        <div>容器运行: {health.containerRunning ? '是' : '否'}</div>
        <div>错误: {health.errorMessage}</div>
      </div>
    ) : null;

    const tag = (
      <Tag color={healthColors[health.status]} icon={healthIcons[health.status]}>
        {healthLabels[health.status]}
      </Tag>
    );

    return tooltipContent ? <Tooltip title={tooltipContent}>{tag}</Tooltip> : tag;
  };

  const columns: ColumnsType<ContainerInfo> = [
    {
      title: '名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
      align: 'center',
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      align: 'center',
      render: (status: ContainerStatus) => (
        <Tag color={statusColors[status]}>{statusLabels[status]}</Tag>
      ),
    },
    {
      title: '健康',
      dataIndex: 'health',
      key: 'health',
      width: 100,
      align: 'center',
      render: (health: HealthCheckResult | undefined, record: ContainerInfo) =>
        renderHealthStatus(health, record),
    },
    {
      title: '端口',
      dataIndex: 'port',
      key: 'port',
      width: 80,
      align: 'center',
    },
    {
      title: 'WebSocket URL',
      dataIndex: 'wsUrl',
      key: 'wsUrl',
      ellipsis: true,
      align: 'center',
      render: (url: string) => {
        // 去掉 {device_id} 部分，只显示 base URL
        const baseUrl = url.replace(/\/\{device_id\}$/, '');
        return (
          <Space>
            <Text copyable={false} style={{ maxWidth: 300 }} ellipsis>
              {baseUrl}
            </Text>
            <Button
              type="text"
              size="small"
              icon={<CopyOutlined />}
              onClick={() => copyWsUrl(baseUrl)}
            />
          </Space>
        );
      },
    },
    {
      title: '创建时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      width: 180,
      align: 'center',
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: '操作',
      key: 'action',
      width: 80,
      align: 'center',
      render: (_, record) => {
        const items = [
          record.status === 'running'
            ? {
                key: 'stop',
                icon: <PauseCircleOutlined />,
                label: '停止',
                onClick: () => stopContainer(record.id),
              }
            : {
                key: 'start',
                icon: <PlayCircleOutlined />,
                label: '启动',
                disabled: record.status === 'creating',
                onClick: () => startContainer(record.id),
              },
          {
            key: 'logs',
            icon: <FileTextOutlined />,
            label: '日志',
            onClick: () => showLogs(record.id, record.name),
          },
          {
            type: 'divider' as const,
          },
          {
            key: 'delete',
            icon: <DeleteOutlined />,
            label: '删除',
            danger: true,
            onClick: () => {
              Modal.confirm({
                title: '确定删除此容器？',
                okText: '删除',
                cancelText: '取消',
                okButtonProps: { danger: true },
                onOk: () => deleteContainer(record.id),
              });
            },
          },
        ];

        return (
          <Dropdown menu={{ items }} trigger={['click']}>
            <Button type="text" icon={<MoreOutlined style={{ fontSize: 18 }} />} />
          </Dropdown>
        );
      },
    },
  ];

  return (
    <div>
      <div style={{ marginBottom: 16, display: 'flex', justifyContent: 'space-between' }}>
        <Typography.Title level={4} style={{ margin: 0 }}>
          EchoKit 服务器
        </Typography.Title>
        <Button icon={<ReloadOutlined />} onClick={fetchContainers}>
          刷新
        </Button>
      </div>
      <Table
        columns={columns}
        dataSource={containers}
        rowKey="id"
        loading={loading}
        pagination={{ pageSize: 10 }}
      />
      <Modal
        title={logsTitle}
        open={logsModalVisible}
        onCancel={() => setLogsModalVisible(false)}
        footer={null}
        width={800}
      >
        <pre
          style={{
            maxHeight: 500,
            overflow: 'auto',
            backgroundColor: '#1e1e1e',
            color: '#d4d4d4',
            padding: 16,
            borderRadius: 4,
            fontSize: 12,
            fontFamily: 'Consolas, Monaco, "Courier New", monospace',
          }}
        >
          {logsContent}
        </pre>
      </Modal>
    </div>
  );
}
