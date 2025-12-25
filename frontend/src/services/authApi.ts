import axios from 'axios';
import type {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  User,
  UpdateUserRequest,
  ChangePasswordRequest,
} from '../types';

const TOKEN_KEY = 'echokit_token';

// 创建认证专用的 axios 实例
const authApi = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 30000,
});

// 请求拦截器：添加 Authorization header
authApi.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem(TOKEN_KEY);
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器：处理 401 错误和错误信息
authApi.interceptors.response.use(
  (response) => response,
  (error) => {
    // 提取后端返回的错误信息
    if (error.response?.data?.error) {
      error.message = error.response.data.error;
    } else if (error.response?.data?.message) {
      error.message = error.response.data.message;
    }

    if (error.response?.status === 401) {
      // 登录接口的 401 不触发自动登出，让调用方处理
      const isLoginRequest = error.config?.url?.includes('/auth/login');
      const isLogoutRequest = error.config?.url?.includes('/auth/logout');

      if (!isLoginRequest && !isLogoutRequest) {
        // Token 过期或无效，清除本地存储
        localStorage.removeItem(TOKEN_KEY);
        // 触发自定义事件，通知应用需要重新登录
        window.dispatchEvent(new CustomEvent('auth:logout'));
      }
    }
    return Promise.reject(error);
  }
);

// Token 管理
export const tokenManager = {
  getToken: (): string | null => {
    return localStorage.getItem(TOKEN_KEY);
  },

  setToken: (token: string): void => {
    localStorage.setItem(TOKEN_KEY, token);
  },

  removeToken: (): void => {
    localStorage.removeItem(TOKEN_KEY);
  },
};

// 认证 API
export const authService = {
  // 用户注册
  register: async (data: RegisterRequest): Promise<RegisterResponse> => {
    const response = await authApi.post<RegisterResponse>('/auth/register', data);
    return response.data;
  },

  // 用户登录
  login: async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await authApi.post<LoginResponse>('/auth/login', data);
    // 保存 token
    tokenManager.setToken(response.data.token);
    return response.data;
  },

  // 用户登出
  logout: async (): Promise<void> => {
    try {
      await authApi.post('/auth/logout');
    } finally {
      tokenManager.removeToken();
    }
  },

  // 获取当前用户信息
  getCurrentUser: async (): Promise<User> => {
    const response = await authApi.get<User>('/auth/me');
    return response.data;
  },

  // 更新用户信息
  updateUser: async (data: UpdateUserRequest): Promise<User> => {
    const response = await authApi.put<User>('/auth/me', data);
    return response.data;
  },

  // 修改密码
  changePassword: async (data: ChangePasswordRequest): Promise<void> => {
    await authApi.put('/auth/password', data);
  },
};

export default authApi;
