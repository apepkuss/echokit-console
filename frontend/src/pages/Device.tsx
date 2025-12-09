import { useState, useEffect } from 'react';
import {
  Card,
  Table,
  Button,
  Space,
  Tag,
  Popconfirm,
  Select,
  message,
  Typography,
} from 'antd';
import { PlusOutlined, DeleteOutlined, LinkOutlined, DisconnectOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import RegisterDeviceModal from '../components/RegisterDeviceModal';
import { deviceService } from '../services/deviceApi';
import { deployService } from '../services/api';
import type { Device, DeviceStatus } from '../types/device';
import type { ContainerInfo } from '../types';

const { Text } = Typography;

export function Device() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [containers, setContainers] = useState<ContainerInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [registerModalVisible, setRegisterModalVisible] = useState(false);
  const [bindingDeviceId, setBindingDeviceId] = useState<string | null>(null);

  // 加载设备列表
  const loadDevices = async () => {
    setLoading(true);
    try {
      const data = await deviceService.listDevices();
      setDevices(data);
    } catch (error) {
      message.error('加载设备列表失败');
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  // 加载容器列表（用于绑定下拉框）
  const loadContainers = async () => {
    try {
      const data = await deployService.listContainers();
      setContainers(data);
    } catch (error) {
      console.error('加载容器列表失败:', error);
    }
  };

  useEffect(() => {
    loadDevices();
    loadContainers();
  }, []);

  // 删除设备
  const handleDelete = async (deviceId: string) => {
    try {
      await deviceService.deleteDevice(deviceId);
      message.success('设备删除成功');
      loadDevices();
    } catch (error) {
      message.error('设备删除失败');
      console.error(error);
    }
  };

  // 绑定设备到服务器
  const handleBind = async (deviceId: string, containerId: string) => {
    // 查找当前设备和目标容器的信息
    const device = devices.find(d => d.deviceId === deviceId);
    const targetContainer = containers.find(c => c.id === containerId);
    const previousContainer = device?.boundContainerId
      ? containers.find(c => c.id === device.boundContainerId)
      : null;

    console.log('[前端] 切换服务器请求:', {
      deviceId,
      deviceName: device?.name,
      previousServer: previousContainer ? (previousContainer.name || previousContainer.id) : '(未绑定)',
      targetServer: targetContainer?.name || containerId,
    });

    setBindingDeviceId(deviceId);
    try {
      await deviceService.bindDeviceToServer(deviceId, containerId);
      console.log('[前端] 切换服务器成功:', {
        deviceId,
        targetServer: targetContainer?.name || containerId,
      });
      message.success('设备绑定成功');
      loadDevices();
    } catch (error) {
      console.error('[前端] 切换服务器失败:', error);
      message.error('设备绑定失败');
    } finally {
      setBindingDeviceId(null);
    }
  };

  // 解绑设备
  const handleUnbind = async (deviceId: string) => {
    const device = devices.find(d => d.deviceId === deviceId);
    const previousContainer = device?.boundContainerId
      ? containers.find(c => c.id === device.boundContainerId)
      : null;

    console.log('[前端] 解绑服务器请求:', {
      deviceId,
      deviceName: device?.name,
      previousServer: previousContainer ? (previousContainer.name || previousContainer.id) : '(未绑定)',
    });

    setBindingDeviceId(deviceId);
    try {
      await deviceService.unbindDevice(deviceId);
      console.log('[前端] 解绑服务器成功:', { deviceId });
      message.success('设备解绑成功');
      loadDevices();
    } catch (error) {
      console.error('[前端] 解绑服务器失败:', error);
      message.error('设备解绑失败');
    } finally {
      setBindingDeviceId(null);
    }
  };

  // 注册成功回调
  const handleRegisterSuccess = () => {
    setRegisterModalVisible(false);
    loadDevices();
    message.success('设备注册成功');
  };

  // 格式化时间戳
  const formatTimestamp = (timestamp?: number) => {
    if (!timestamp) return '-';
    return new Date(timestamp * 1000).toLocaleString('zh-CN');
  };

  // 渲染设备状态
  const renderStatus = (status: DeviceStatus) => {
    const statusConfig = {
      online: { color: 'success', text: '在线' },
      offline: { color: 'default', text: '离线' },
      unknown: { color: 'warning', text: '未知' },
    };
    const config = statusConfig[status];
    return <Tag color={config.color}>{config.text}</Tag>;
  };

  // 表格列定义
  const columns: ColumnsType<Device> = [
    {
      title: '设备名称',
      dataIndex: 'name',
      key: 'name',
      width: 200,
      align: 'center',
    },
    {
      title: 'MAC 地址',
      dataIndex: 'macAddress',
      key: 'macAddress',
      width: 180,
      align: 'center',
      render: (mac: string) => <Text code>{mac}</Text>,
    },
    {
      title: '状态',
      dataIndex: 'status',
      key: 'status',
      width: 100,
      align: 'center',
      render: renderStatus,
    },
    {
      title: 'EchoKit 服务器',
      dataIndex: 'boundContainerId',
      key: 'boundContainerId',
      width: 250,
      align: 'center',
      render: (containerId: string | undefined, record: Device) => (
        <Select
          style={{ width: '100%' }}
          placeholder="选择服务器"
          value={containerId}
          onChange={(value) => handleBind(record.deviceId, value)}
          loading={bindingDeviceId === record.deviceId}
          allowClear
          onClear={() => handleUnbind(record.deviceId)}
        >
          {containers.map((container) => (
            <Select.Option key={container.id} value={container.id}>
              {container.name || container.id.slice(0, 12)}
            </Select.Option>
          ))}
        </Select>
      ),
    },
    {
      title: '最后连接时间',
      dataIndex: 'lastConnectedAt',
      key: 'lastConnectedAt',
      width: 180,
      align: 'center',
      render: formatTimestamp,
    },
    {
      title: '创建时间',
      dataIndex: 'createdAt',
      key: 'createdAt',
      width: 180,
      align: 'center',
      render: formatTimestamp,
    },
    {
      title: '操作',
      key: 'actions',
      width: 150,
      align: 'center',
      fixed: 'right',
      render: (_, record: Device) => (
        <Space>
          {record.boundContainerId ? (
            <Button
              type="link"
              size="small"
              icon={<DisconnectOutlined />}
              onClick={() => handleUnbind(record.deviceId)}
              loading={bindingDeviceId === record.deviceId}
            >
              解绑
            </Button>
          ) : (
            <Button
              type="link"
              size="small"
              icon={<LinkOutlined />}
              disabled
            >
              绑定
            </Button>
          )}
          <Popconfirm
            title="确定要删除此设备吗？"
            description="删除后将无法恢复"
            onConfirm={() => handleDelete(record.deviceId)}
            okText="确定"
            cancelText="取消"
          >
            <Button
              type="link"
              danger
              size="small"
              icon={<DeleteOutlined />}
            >
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ padding: 24 }}>
      <Card
        title={
          <Space>
            <span>设备列表</span>
            <Button
              type="primary"
              size="small"
              icon={<PlusOutlined />}
              onClick={() => setRegisterModalVisible(true)}
            >
              注册设备
            </Button>
          </Space>
        }
      >
        <Table
          columns={columns}
          dataSource={devices}
          rowKey="deviceId"
          loading={loading}
          pagination={{
            showSizeChanger: true,
            showTotal: (total) => `共 ${total} 个设备`,
          }}
          scroll={{ x: 1200 }}
        />
      </Card>

      <RegisterDeviceModal
        open={registerModalVisible}
        onCancel={() => setRegisterModalVisible(false)}
        onSuccess={handleRegisterSuccess}
      />
    </div>
  );
}
