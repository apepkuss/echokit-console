import React, { useState } from 'react';
import {
  Modal,
  Steps,
  Button,
  Form,
  Input,
  Upload,
  Progress,
  Result,
  Alert,
  message,
} from 'antd';
import {
  WifiOutlined,
  CloudServerOutlined,
  PictureOutlined,
  CheckCircleOutlined,
  LoadingOutlined,
  SyncOutlined,
} from '@ant-design/icons';
import type { UploadFile } from 'antd/es/upload/interface';
import { BluetoothService } from '../services/bluetooth';
import { deviceService } from '../services/deviceApi';
import type { BluetoothConfig } from '../types/device';

interface RegisterDeviceModalProps {
  open: boolean;
  onCancel: () => void;
  onSuccess: () => void;
}

const RegisterDeviceModal: React.FC<RegisterDeviceModalProps> = ({
  open,
  onCancel,
  onSuccess,
}) => {
  const [form] = Form.useForm();
  const [currentStep, setCurrentStep] = useState(0);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [bluetoothService] = useState(() => new BluetoothService());
  const [deviceMac, setDeviceMac] = useState<string>('');
  const [deviceBluetoothName, setDeviceBluetoothName] = useState<string>('');
  const [fileList, setFileList] = useState<UploadFile[]>([]);

  // 步骤定义
  const steps = [
    {
      title: '连接设备',
      icon: <CloudServerOutlined />,
    },
    {
      title: '配置参数',
      icon: <WifiOutlined />,
    },
    {
      title: '写入配置',
      icon: <SyncOutlined />,
    },
    {
      title: '完成',
      icon: <CheckCircleOutlined />,
    },
  ];

  // 步骤 1: 连接设备
  const handleConnectDevice = async () => {
    if (!BluetoothService.isSupported()) {
      setErrorMessage('当前浏览器不支持 Web Bluetooth API，请使用 Chrome、Edge 或 Opera 浏览器');
      return;
    }

    setLoading(true);
    setErrorMessage(null);

    try {
      // 连接设备
      await bluetoothService.connect();

      // 读取设备信息
      const mac = await bluetoothService.getMacAddress();
      const bluetoothName = bluetoothService.getDeviceName();

      setDeviceMac(mac);
      setDeviceBluetoothName(bluetoothName);

      // 自动填充表单（使用蓝牙名称）
      form.setFieldsValue({
        deviceName: bluetoothName,
      });

      message.success('设备连接成功');
      setCurrentStep(1);
    } catch (error: any) {
      setErrorMessage(error.message || '连接设备失败，请重试');
      message.error('连接设备失败');
    } finally {
      setLoading(false);
    }
  };

  // 步骤 2: 验证并提交配置表单
  const handleConfigSubmit = async () => {
    try {
      const values = await form.validateFields();
      setCurrentStep(2);
      await handleWriteConfig(values);
    } catch (error) {
      console.error('表单验证失败:', error);
    }
  };

  // 步骤 3: 写入配置到设备
  const handleWriteConfig = async (values: any) => {
    setLoading(true);
    setProgress(0);
    setErrorMessage(null);

    try {
      // 1. 先检查设备是否已注册
      let isNewDevice = true;
      try {
        await deviceService.registerDevice({
          deviceId: deviceMac,
          name: values.deviceName,
          macAddress: deviceMac,
          boundContainerId: values.boundContainerId || 'official-dallas',
        });
        console.log('设备注册成功');
      } catch (error: any) {
        if (error.response?.status === 409) {
          // 设备已经注册过
          isNewDevice = false;
          console.log('设备已注册过');
          message.warning('设备已注册过，将跳过配置写入步骤');
          // 直接跳到完成步骤
          setCurrentStep(3);
          setLoading(false);
          return;
        } else {
          // 其他错误抛出
          throw error;
        }
      }

      // 2. 只有新设备才写入配置
      if (isNewDevice) {
        const config: BluetoothConfig = {
          ssid: values.wifiSsid,
          password: values.wifiPassword,
          serverUrl: values.serverUrl,
          backgroundImage: fileList.length > 0 ? (fileList[0].originFileObj as File) : undefined,
        };

        // 写入配置
        await bluetoothService.writeConfig(config, (p) => {
          setProgress(Math.round(p * 100));
        });

        // 重启设备
        await bluetoothService.resetDevice();

        // 断开连接
        bluetoothService.disconnect();

        message.success('配置写入成功，设备正在重启');
      }

      setCurrentStep(3);
    } catch (error: any) {
      setErrorMessage(error.message || '写入配置失败，请重试');
      message.error('写入配置失败');
    } finally {
      setLoading(false);
    }
  };

  // 处理取消
  const handleCancel = () => {
    if (bluetoothService.isConnected()) {
      bluetoothService.disconnect();
    }
    form.resetFields();
    setCurrentStep(0);
    setProgress(0);
    setErrorMessage(null);
    setDeviceMac('');
    setFileList([]);
    onCancel();
  };

  // 处理完成
  const handleFinish = () => {
    handleCancel();
    onSuccess();
  };

  // 渲染步骤内容
  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <CloudServerOutlined style={{ fontSize: 64, color: '#1890ff', marginBottom: 24 }} />
            <p style={{ fontSize: 16, marginBottom: 24 }}>
              请确保设备已开机并处于配对模式，然后点击下方按钮连接设备
            </p>
            {errorMessage && (
              <Alert message={errorMessage} type="error" showIcon style={{ marginBottom: 16 }} />
            )}
            <Button
              type="primary"
              size="large"
              loading={loading}
              onClick={handleConnectDevice}
              icon={<CloudServerOutlined />}
            >
              连接设备
            </Button>
          </div>
        );

      case 1:
        return (
          <div style={{ padding: '24px 0' }}>
            <Alert
              message={`设备已连接: ${deviceBluetoothName}`}
              type="success"
              showIcon
              style={{ marginBottom: 24 }}
            />
            <Form form={form} layout="vertical">
              <Form.Item
                label="设备名称"
                name="deviceName"
                rules={[{ required: true, message: '请输入设备名称' }]}
              >
                <Input placeholder="例如：客厅音箱" />
              </Form.Item>

              <Form.Item
                label="WiFi SSID"
                name="wifiSsid"
                rules={[{ required: true, message: '请输入 WiFi SSID' }]}
              >
                <Input placeholder="请输入 WiFi 名称" />
              </Form.Item>

              <Form.Item
                label="WiFi 密码"
                name="wifiPassword"
                rules={[{ required: true, message: '请输入 WiFi 密码' }]}
              >
                <Input.Password placeholder="请输入 WiFi 密码" />
              </Form.Item>

              <Form.Item
                label="Proxy 服务器地址"
                name="serverUrl"
                tooltip="设备将通过此地址连接到 EchoKit Proxy，Proxy 会根据设备绑定关系转发到对应的服务器"
                rules={[
                  { required: true, message: '请输入 Proxy 服务器地址' },
                  { type: 'url', message: '请输入有效的 URL' },
                ]}
                initialValue={import.meta.env.VITE_PROXY_WS_URL || 'wss://proxy.echokit.dev/ws'}
              >
                <Input placeholder="例如：wss://proxy.echokit.dev/ws" />
              </Form.Item>

              <Form.Item label="背景图片（可选）" name="backgroundImage">
                <Upload
                  listType="picture-card"
                  fileList={fileList}
                  beforeUpload={(file) => {
                    // 检查文件类型
                    const isImage = file.type.startsWith('image/');
                    if (!isImage) {
                      message.error('只能上传图片文件');
                      return false;
                    }
                    // 检查文件大小 (限制 2MB)
                    const isLt2M = file.size / 1024 / 1024 < 2;
                    if (!isLt2M) {
                      message.error('图片大小不能超过 2MB');
                      return false;
                    }
                    setFileList([file]);
                    return false; // 阻止自动上传
                  }}
                  onRemove={() => {
                    setFileList([]);
                  }}
                  maxCount={1}
                >
                  {fileList.length === 0 && (
                    <div>
                      <PictureOutlined />
                      <div style={{ marginTop: 8 }}>选择图片</div>
                    </div>
                  )}
                </Upload>
              </Form.Item>
            </Form>
          </div>
        );

      case 2:
        return (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <LoadingOutlined style={{ fontSize: 64, color: '#1890ff', marginBottom: 24 }} />
            <p style={{ fontSize: 16, marginBottom: 24 }}>正在写入配置到设备...</p>
            <Progress percent={progress} status="active" style={{ maxWidth: 400, margin: '0 auto' }} />
            {errorMessage && (
              <Alert message={errorMessage} type="error" showIcon style={{ marginTop: 24 }} />
            )}
          </div>
        );

      case 3:
        return (
          <Result
            status="success"
            title="设备注册成功！"
            subTitle="设备已成功注册并配置完成，设备正在重启中"
            extra={[
              <Button type="primary" key="finish" onClick={handleFinish}>
                完成
              </Button>,
            ]}
          />
        );

      default:
        return null;
    }
  };

  return (
    <Modal
      title="注册新设备"
      open={open}
      onCancel={handleCancel}
      width={700}
      footer={
        currentStep === 3
          ? null
          : [
              <Button key="cancel" onClick={handleCancel}>
                取消
              </Button>,
              currentStep === 1 && (
                <Button
                  key="submit"
                  type="primary"
                  loading={loading}
                  onClick={handleConfigSubmit}
                >
                  下一步
                </Button>
              ),
            ]
      }
    >
      <Steps current={currentStep} items={steps} style={{ marginBottom: 32 }} />
      {renderStepContent()}
    </Modal>
  );
};

export default RegisterDeviceModal;
