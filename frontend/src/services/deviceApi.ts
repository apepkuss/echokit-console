import api from './api';
import type { Device, RegisterDeviceRequest } from '../types/device';

// Mock 数据（用于前端开发）
const MOCK_DEVICES: Device[] = [
  {
    deviceId: 'AA:BB:CC:DD:EE:FF',
    name: '客厅音箱',
    macAddress: 'AA:BB:CC:DD:EE:FF',
    boundContainerId: 'container_001',
    createdAt: Date.now() / 1000 - 86400 * 7,
    lastConnectedAt: Date.now() / 1000 - 3600,
    status: 'online',
  },
  {
    deviceId: '11:22:33:44:55:66',
    name: '卧室音箱',
    macAddress: '11:22:33:44:55:66',
    createdAt: Date.now() / 1000 - 86400 * 3,
    status: 'offline',
  },
];

// 是否使用 Mock 数据（开发阶段设为 true，生产环境设为 false）
const USE_MOCK = false;

export const deviceService = {
  // 获取设备列表
  listDevices: async (): Promise<Device[]> => {
    if (USE_MOCK) {
      // 模拟网络延迟
      await new Promise(resolve => setTimeout(resolve, 500));
      return MOCK_DEVICES;
    }

    const response = await api.get<Device[]>('/devices');
    return response.data;
  },

  // 获取单个设备信息
  getDevice: async (deviceId: string): Promise<Device> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 300));
      const device = MOCK_DEVICES.find(d => d.deviceId === deviceId);
      if (!device) throw new Error('Device not found');
      return device;
    }

    const response = await api.get<Device>(`/devices/${encodeURIComponent(deviceId)}`);
    return response.data;
  },

  // 注册新设备
  registerDevice: async (request: RegisterDeviceRequest): Promise<Device> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 800));
      const newDevice: Device = {
        ...request,
        createdAt: Date.now() / 1000,
        status: 'unknown',
      };
      MOCK_DEVICES.push(newDevice);
      return newDevice;
    }

    const response = await api.post<Device>('/devices', request);
    return response.data;
  },

  // 更新设备信息
  updateDevice: async (deviceId: string, updates: Partial<Device>): Promise<Device> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 500));
      const index = MOCK_DEVICES.findIndex(d => d.deviceId === deviceId);
      if (index === -1) throw new Error('Device not found');
      MOCK_DEVICES[index] = { ...MOCK_DEVICES[index], ...updates };
      return MOCK_DEVICES[index];
    }

    const response = await api.put<Device>(`/devices/${encodeURIComponent(deviceId)}`, updates);
    return response.data;
  },

  // 删除设备
  deleteDevice: async (deviceId: string): Promise<void> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 400));
      const index = MOCK_DEVICES.findIndex(d => d.deviceId === deviceId);
      if (index === -1) throw new Error('Device not found');
      MOCK_DEVICES.splice(index, 1);
      return;
    }

    await api.delete(`/devices/${encodeURIComponent(deviceId)}`);
  },

  // 绑定设备到服务器
  bindDeviceToServer: async (deviceId: string, containerId: string): Promise<void> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 500));
      const device = MOCK_DEVICES.find(d => d.deviceId === deviceId);
      if (device) {
        device.boundContainerId = containerId;
      }
      return;
    }

    await api.post(`/devices/${encodeURIComponent(deviceId)}/bind`, { containerId });
  },

  // 解绑设备
  unbindDevice: async (deviceId: string): Promise<void> => {
    if (USE_MOCK) {
      await new Promise(resolve => setTimeout(resolve, 400));
      const device = MOCK_DEVICES.find(d => d.deviceId === deviceId);
      if (device) {
        device.boundContainerId = undefined;
      }
      return;
    }

    await api.delete(`/devices/${encodeURIComponent(deviceId)}/bind`);
  },
};
