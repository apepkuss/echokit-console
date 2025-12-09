// 设备状态
export type DeviceStatus = 'online' | 'offline' | 'unknown';

// 设备信息
export interface Device {
  deviceId: string;          // 设备唯一标识（MAC 地址）
  name: string;               // 设备名称（用户友好）
  macAddress: string;        // WiFi MAC 地址
  boundContainerId?: string; // 绑定的 EchoKit Server 容器 ID
  createdAt: number;         // 创建时间（Unix 时间戳）
  lastConnectedAt?: number; // 最后连接时间
  status: DeviceStatus;       // 连接状态
}

// 设备注册请求
export interface RegisterDeviceRequest {
  deviceId: string;
  name: string;
  macAddress: string;
  boundContainerId?: string; // 可选：初始绑定的 Server
}

// 蓝牙配置数据（用于写入设备）
export interface BluetoothConfig {
  ssid: string;
  password: string;
  serverUrl: string;
  backgroundImage?: File;
}
