import { create } from 'zustand';
import type { ContainerInfo, EchoKitConfig, DeployResponse } from '../types';
import { deployService } from '../services';

interface DeployState {
  // 容器列表
  containers: ContainerInfo[];
  // 加载状态
  loading: boolean;
  // 部署中状态
  deploying: boolean;
  // 错误信息
  error: string | null;

  // Actions
  fetchContainers: () => Promise<void>;
  deploy: (config: EchoKitConfig, port?: number) => Promise<DeployResponse>;
  stopContainer: (id: string) => Promise<void>;
  startContainer: (id: string) => Promise<void>;
  deleteContainer: (id: string) => Promise<void>;
  clearError: () => void;
}

export const useDeployStore = create<DeployState>((set) => ({
  containers: [],
  loading: false,
  deploying: false,
  error: null,

  fetchContainers: async () => {
    set({ loading: true, error: null });
    try {
      const containers = await deployService.listContainers();
      set({ containers, loading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : '获取容器列表失败',
        loading: false,
      });
    }
  },

  deploy: async (config: EchoKitConfig, port?: number) => {
    set({ deploying: true, error: null });
    try {
      const response = await deployService.deploy({ config, port });
      const containerInfo: ContainerInfo = {
        id: response.containerId,
        name: response.containerName,
        port: response.port,
        wsUrl: response.wsUrl,
        status: response.status,
        createdAt: new Date().toISOString(),
        health: response.health,
      };
      set((state) => ({
        containers: [...state.containers, containerInfo],
        deploying: false,
      }));
      return response;
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : '部署失败',
        deploying: false,
      });
      throw err;
    }
  },

  stopContainer: async (id: string) => {
    try {
      await deployService.stopContainer(id);
      set((state) => ({
        containers: state.containers.map((c) =>
          c.id === id ? { ...c, status: 'stopped' as const } : c
        ),
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : '停止容器失败',
      });
    }
  },

  startContainer: async (id: string) => {
    try {
      await deployService.startContainer(id);
      set((state) => ({
        containers: state.containers.map((c) =>
          c.id === id ? { ...c, status: 'running' as const } : c
        ),
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : '启动容器失败',
      });
    }
  },

  deleteContainer: async (id: string) => {
    try {
      await deployService.deleteContainer(id);
      set((state) => ({
        containers: state.containers.filter((c) => c.id !== id),
      }));
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : '删除容器失败',
      });
    }
  },

  clearError: () => set({ error: null }),
}));
