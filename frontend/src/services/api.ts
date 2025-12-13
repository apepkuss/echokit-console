import axios from 'axios';
import type {
  DeployRequest,
  DeployResponse,
  ContainerInfo,
  HealthCheckResult,
} from '../types';
import { tokenManager } from './authApi';

const TOKEN_KEY = 'echokit_token';

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 30000,
});

// 请求拦截器：添加 Authorization header
api.interceptors.request.use(
  (config) => {
    const token = tokenManager.getToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器：处理 401 错误
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Token 过期或无效，清除本地存储
      localStorage.removeItem(TOKEN_KEY);
      // 触发自定义事件，通知应用需要重新登录
      window.dispatchEvent(new CustomEvent('auth:logout'));
    }
    return Promise.reject(error);
  }
);

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
