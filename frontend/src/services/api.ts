import axios from 'axios';
import type {
  DeployRequest,
  DeployResponse,
  ContainerInfo,
  HealthCheckResult,
} from '../types';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 30000,
});

// 部署相关 API
export const deployService = {
  // 部署新的 EchoKit 实例
  deploy: async (request: DeployRequest): Promise<DeployResponse> => {
    const response = await api.post<DeployResponse>('/deploy', request);
    return response.data;
  },

  // 获取所有容器列表
  listContainers: async (): Promise<ContainerInfo[]> => {
    const response = await api.get<ContainerInfo[]>('/containers');
    return response.data;
  },

  // 获取单个容器信息
  getContainer: async (id: string): Promise<ContainerInfo> => {
    const response = await api.get<ContainerInfo>(`/containers/${id}`);
    return response.data;
  },

  // 停止容器
  stopContainer: async (id: string): Promise<void> => {
    await api.post(`/containers/${id}/stop`);
  },

  // 启动容器
  startContainer: async (id: string): Promise<void> => {
    await api.post(`/containers/${id}/start`);
  },

  // 删除容器
  deleteContainer: async (id: string): Promise<void> => {
    await api.delete(`/containers/${id}`);
  },

  // 获取容器日志
  getContainerLogs: async (id: string, tail?: number): Promise<string> => {
    const response = await api.get<string>(`/containers/${id}/logs`, {
      params: { tail },
    });
    return response.data;
  },

  // 获取容器健康状态
  getContainerHealth: async (id: string): Promise<HealthCheckResult> => {
    const response = await api.get<HealthCheckResult>(`/containers/${id}/health`);
    return response.data;
  },
};

export default api;
