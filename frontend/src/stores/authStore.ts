import { create } from 'zustand';
import type {
  User,
  LoginRequest,
  RegisterRequest,
  UpdateUserRequest,
  ChangePasswordRequest,
} from '../types';
import { authService, tokenManager } from '../services/authApi';

interface AuthState {
  // 状态
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  isInitialized: boolean;
  error: string | null;

  // Actions
  initialize: () => Promise<void>;
  login: (data: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<void>;
  logout: () => Promise<void>;
  updateUser: (data: UpdateUserRequest) => Promise<void>;
  changePassword: (data: ChangePasswordRequest) => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  isAuthenticated: false,
  isLoading: false,
  isInitialized: false,
  error: null,

  // 初始化：检查本地 token 并获取用户信息
  initialize: async () => {
    const token = tokenManager.getToken();
    if (!token) {
      set({ isInitialized: true, isAuthenticated: false });
      return;
    }

    set({ isLoading: true });
    try {
      const user = await authService.getCurrentUser();
      set({
        user,
        isAuthenticated: true,
        isLoading: false,
        isInitialized: true,
      });
    } catch {
      // Token 无效，清除
      tokenManager.removeToken();
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        isInitialized: true,
      });
    }
  },

  // 登录
  login: async (data: LoginRequest) => {
    set({ isLoading: true, error: null });
    try {
      const response = await authService.login(data);
      set({
        user: response.user,
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : '登录失败';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  // 注册
  register: async (data: RegisterRequest) => {
    set({ isLoading: true, error: null });
    try {
      await authService.register(data);
      // 注册成功后自动登录
      await get().login({ email: data.email, password: data.password });
    } catch (err) {
      const message = err instanceof Error ? err.message : '注册失败';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  // 登出
  logout: async () => {
    // 如果已经在登出中或未认证，直接返回
    const state = get();
    if (!state.isAuthenticated && !state.user) {
      return;
    }

    set({ isLoading: true });
    try {
      await authService.logout();
    } catch {
      // 忽略登出 API 错误（可能 token 已过期）
    } finally {
      set({
        user: null,
        isAuthenticated: false,
        isLoading: false,
      });
    }
  },

  // 更新用户信息
  updateUser: async (data: UpdateUserRequest) => {
    set({ isLoading: true, error: null });
    try {
      const user = await authService.updateUser(data);
      set({ user, isLoading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : '更新用户信息失败';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  // 修改密码
  changePassword: async (data: ChangePasswordRequest) => {
    set({ isLoading: true, error: null });
    try {
      await authService.changePassword(data);
      set({ isLoading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : '修改密码失败';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  // 清除错误
  clearError: () => set({ error: null }),
}));

// 监听 401 事件，自动登出
if (typeof window !== 'undefined') {
  window.addEventListener('auth:logout', () => {
    useAuthStore.getState().logout();
  });
}
