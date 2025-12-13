// 用户信息
export interface User {
  id: string;
  email: string;
  name: string | null;
  createdAt: number;
  updatedAt: number | null;
}

// 登录请求
export interface LoginRequest {
  email: string;
  password: string;
}

// 登录响应
export interface LoginResponse {
  token: string;
  user: User;
}

// 注册请求
export interface RegisterRequest {
  email: string;
  password: string;
  name?: string;
}

// 注册响应
export interface RegisterResponse {
  user: User;
}

// 更新用户请求
export interface UpdateUserRequest {
  name?: string;
}

// 修改密码请求
export interface ChangePasswordRequest {
  currentPassword: string;
  newPassword: string;
}

// 认证状态
export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}
