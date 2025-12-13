import type { BluetoothConfig } from '../types/device';

// Web Bluetooth API 类型声明
declare global {
  interface Navigator {
    bluetooth: {
      requestDevice(options: RequestDeviceOptions): Promise<BluetoothDevice>;
    };
  }

  interface RequestDeviceOptions {
    filters?: Array<{ services?: string[]; name?: string; namePrefix?: string }>;
    optionalServices?: string[];
  }

  interface BluetoothDevice {
    id: string;
    name?: string;
    gatt?: BluetoothRemoteGATTServer;
  }

  interface BluetoothRemoteGATTServer {
    connected: boolean;
    connect(): Promise<BluetoothRemoteGATTServer>;
    disconnect(): void;
    getPrimaryService(service: string): Promise<BluetoothRemoteGATTService>;
  }

  interface BluetoothRemoteGATTService {
    getCharacteristic(characteristic: string): Promise<BluetoothRemoteGATTCharacteristic>;
  }

  interface BluetoothRemoteGATTCharacteristic {
    readValue(): Promise<DataView>;
    writeValue(value: BufferSource): Promise<void>;
  }
}

// BLE 特征值 UUID（来自 echokit/setup/index.html）
const BLE_SERVICE_UUID = '623fa3e2-631b-4f8f-a6e7-a7b09c03e7e0';
const BLE_CHARACTERISTIC_MAC_ADDRESS = 'a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d';
const BLE_CHARACTERISTIC_SSID = '1fda4d6e-2f14-42b0-96fa-453bed238375';
const BLE_CHARACTERISTIC_PASSWORD = 'a987ab18-a940-421a-a1d7-b94ee22bccbe';
const BLE_CHARACTERISTIC_SERVER_URL = 'cef520a9-bcb5-4fc6-87f7-82804eee2b20';
const BLE_CHARACTERISTIC_BACKGROUND_IMAGE = 'd1f3b2c4-5e6f-4a7b-8c9d-0e1f2a3b4c5d';
const BLE_CHARACTERISTIC_RESET = 'f0e1d2c3-b4a5-6789-0abc-def123456789';

// 分块传输参数
const CHUNK_SIZE = 512; // 每块 512 字节
const CHUNK_DELAY = 50; // 每块延迟 50ms

export class BluetoothService {
  private device: BluetoothDevice | null = null;
  private server: BluetoothRemoteGATTServer | null = null;
  private service: BluetoothRemoteGATTService | null = null;

  /**
   * 检查浏览器是否支持 Web Bluetooth API
   */
  static isSupported(): boolean {
    return 'bluetooth' in navigator;
  }

  /**
   * 连接到 EchoKit 设备
   */
  async connect(): Promise<void> {
    if (!BluetoothService.isSupported()) {
      throw new Error('当前浏览器不支持 Web Bluetooth API');
    }

    try {
      // 请求设备
      this.device = await navigator.bluetooth.requestDevice({
        filters: [{ services: [BLE_SERVICE_UUID] }],
        optionalServices: [BLE_SERVICE_UUID],
      });

      // 连接到 GATT 服务器
      if (!this.device.gatt) {
        throw new Error('设备不支持 GATT');
      }

      this.server = await this.device.gatt.connect();
      this.service = await this.server.getPrimaryService(BLE_SERVICE_UUID);

      console.log('已成功连接到设备:', this.device.name);
    } catch (error) {
      console.error('连接设备失败:', error);
      throw new Error('连接设备失败，请确保设备已开机并处于配对模式');
    }
  }

  /**
   * 断开设备连接
   */
  disconnect(): void {
    if (this.server?.connected) {
      this.server.disconnect();
    }
    this.device = null;
    this.server = null;
    this.service = null;
  }

  /**
   * 检查是否已连接
   */
  isConnected(): boolean {
    return this.server?.connected ?? false;
  }

  /**
   * 获取设备 MAC 地址
   * 尝试从 BLE 特征值读取 MAC 地址，如果失败则使用设备蓝牙 ID
   */
  async getMacAddress(): Promise<string> {
    if (!this.service) {
      throw new Error('请先连接设备');
    }

    try {
      // 尝试从 BLE 特征值读取 MAC 地址
      const characteristic = await this.service.getCharacteristic(BLE_CHARACTERISTIC_MAC_ADDRESS);
      const value = await characteristic.readValue();
      const decoder = new TextDecoder();
      const macAddress = decoder.decode(value);

      console.log('从设备读取的 MAC 地址:', macAddress);
      return macAddress;
    } catch (error) {
      // 如果读取失败，使用设备的 Bluetooth ID 作为后备方案
      console.warn('无法读取 MAC 地址特征值，使用设备 ID 作为标识符:', error);
      const deviceId = this.device?.id || `${this.device?.name}-${Date.now()}`;
      console.log('使用设备标识符:', deviceId);
      return deviceId;
    }
  }

  /**
   * 获取设备名称
   */
  getDeviceName(): string {
    if (!this.device) {
      throw new Error('请先连接设备');
    }
    return this.device.name || '未知设备';
  }

  /**
   * 写入配置到设备
   * @param config 配置数据
   * @param onProgress 进度回调 (0-1)
   */
  async writeConfig(
    config: BluetoothConfig,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    if (!this.service) {
      throw new Error('请先连接设备');
    }

    try {
      let completedSteps = 0;
      const totalSteps = config.backgroundImage ? 4 : 3;

      // 1. 写入 SSID
      await this.writeCharacteristic(BLE_CHARACTERISTIC_SSID, config.ssid);
      completedSteps++;
      onProgress?.(completedSteps / totalSteps);

      // 2. 写入密码
      await this.writeCharacteristic(BLE_CHARACTERISTIC_PASSWORD, config.password);
      completedSteps++;
      onProgress?.(completedSteps / totalSteps);

      // 3. 写入服务器 URL
      await this.writeCharacteristic(BLE_CHARACTERISTIC_SERVER_URL, config.serverUrl);
      completedSteps++;
      onProgress?.(completedSteps / totalSteps);

      // 4. 写入背景图片（如果有）
      if (config.backgroundImage) {
        await this.writeBackgroundImage(config.backgroundImage, (imgProgress) => {
          onProgress?.((completedSteps + imgProgress) / totalSteps);
        });
        completedSteps++;
        onProgress?.(completedSteps / totalSteps);
      }

      console.log('配置写入成功');
    } catch (error) {
      console.error('写入配置失败:', error);
      throw new Error('写入配置失败，请重试');
    }
  }

  /**
   * 写入字符串特征值
   */
  private async writeCharacteristic(uuid: string, value: string): Promise<void> {
    if (!this.service) {
      throw new Error('服务未初始化');
    }

    const characteristic = await this.service.getCharacteristic(uuid);
    const encoder = new TextEncoder();
    const data = encoder.encode(value);
    await characteristic.writeValue(data);
  }

  /**
   * 分块传输背景图片
   */
  private async writeBackgroundImage(
    file: File,
    onProgress?: (progress: number) => void
  ): Promise<void> {
    if (!this.service) {
      throw new Error('服务未初始化');
    }

    const characteristic = await this.service.getCharacteristic(
      BLE_CHARACTERISTIC_BACKGROUND_IMAGE
    );

    // 读取文件为 ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();
    const totalBytes = arrayBuffer.byteLength;
    const totalChunks = Math.ceil(totalBytes / CHUNK_SIZE);

    // 分块发送
    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const end = Math.min(start + CHUNK_SIZE, totalBytes);
      const chunk = arrayBuffer.slice(start, end);

      await characteristic.writeValue(chunk);

      // 更新进度
      onProgress?.((i + 1) / totalChunks);

      // 延迟以避免传输过快
      if (i < totalChunks - 1) {
        await new Promise(resolve => setTimeout(resolve, CHUNK_DELAY));
      }
    }
  }

  /**
   * 重启设备
   */
  async resetDevice(): Promise<void> {
    if (!this.service) {
      throw new Error('请先连接设备');
    }

    try {
      const characteristic = await this.service.getCharacteristic(BLE_CHARACTERISTIC_RESET);
      const encoder = new TextEncoder();
      await characteristic.writeValue(encoder.encode('RESET'));
      console.log('设备重启命令已发送');
    } catch (error) {
      console.error('重启设备失败:', error);
      throw new Error('重启设备失败');
    }
  }
}
