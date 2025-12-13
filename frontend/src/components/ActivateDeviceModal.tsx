import { useState } from 'react';
import {
  Modal,
  Input,
  Button,
  Form,
  Result,
  message,
  Typography,
  Space,
  Divider,
} from 'antd';
import {
  MobileOutlined,
  CheckCircleOutlined,
  NumberOutlined,
} from '@ant-design/icons';
import { deviceService } from '../services/deviceApi';

const { Title, Paragraph, Text } = Typography;

interface ActivateDeviceModalProps {
  open: boolean;
  onCancel: () => void;
  onSuccess: () => void;
}

export default function ActivateDeviceModal({
  open,
  onCancel,
  onSuccess,
}: ActivateDeviceModalProps) {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [boundDevice, setBoundDevice] = useState<{ id: string; name: string } | null>(null);

  const handleSubmit = async (values: { code: string; deviceName?: string }) => {
    setLoading(true);
    try {
      const result = await deviceService.confirmActivation({
        code: values.code.trim(),
        deviceName: values.deviceName?.trim(),
      });

      setBoundDevice({
        id: result.deviceId,
        name: values.deviceName || `EchoKit-${result.deviceId.slice(-5)}`,
      });
      setSuccess(true);
      message.success('激活码已确认');
    } catch (error: unknown) {
      const err = error as { response?: { data?: { message?: string } } };
      const errorMsg = err.response?.data?.message || '激活失败，请检查激活码是否正确';
      message.error(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    // 如果激活成功，关闭时触发刷新
    if (success) {
      onSuccess();
    }
    // 重置状态
    setSuccess(false);
    setBoundDevice(null);
    form.resetFields();
    onCancel();
  };

  const handleBindAnother = () => {
    setSuccess(false);
    setBoundDevice(null);
    form.resetFields();
  };

  return (
    <Modal
      title={null}
      open={open}
      onCancel={handleClose}
      footer={null}
      width={480}
      destroyOnClose
    >
      {success && boundDevice ? (
        <Result
          status="success"
          icon={<CheckCircleOutlined style={{ color: '#52c41a' }} />}
          title="激活码已确认"
          subTitle={
            <Space direction="vertical" size="small">
              <Text>设备 <Text strong>{boundDevice.name}</Text> 正在完成绑定</Text>
              <Text type="secondary">请等待设备屏幕显示绑定成功</Text>
            </Space>
          }
          extra={
            <Space>
              <Button type="primary" onClick={handleClose}>
                完成
              </Button>
              <Button onClick={handleBindAnother}>
                绑定另一台设备
              </Button>
            </Space>
          }
        />
      ) : (
        <Space direction="vertical" size="large" style={{ width: '100%' }}>
          {/* 标题 */}
          <div style={{ textAlign: 'center' }}>
            <MobileOutlined style={{ fontSize: 48, color: '#1890ff' }} />
            <Title level={3} style={{ marginTop: 16, marginBottom: 8 }}>
              激活新设备
            </Title>
            <Paragraph type="secondary">
              请查看设备屏幕上显示的 6 位激活码
            </Paragraph>
          </div>

          <Divider style={{ margin: '12px 0' }} />

          {/* 表单 */}
          <Form
            form={form}
            layout="vertical"
            onFinish={handleSubmit}
            requiredMark={false}
          >
            <Form.Item
              name="code"
              label={
                <Space>
                  <NumberOutlined />
                  <span>激活码</span>
                </Space>
              }
              rules={[
                { required: true, message: '请输入激活码' },
                { len: 6, message: '激活码为 6 位数字' },
                { pattern: /^\d{6}$/, message: '激活码只能包含数字' },
              ]}
            >
              <Input
                placeholder="输入 6 位激活码"
                maxLength={6}
                size="large"
                style={{
                  textAlign: 'center',
                  fontSize: 28,
                  letterSpacing: 12,
                  fontFamily: 'monospace',
                  fontWeight: 'bold',
                }}
                autoComplete="off"
              />
            </Form.Item>

            <Form.Item
              name="deviceName"
              label="设备名称（可选）"
              tooltip="为设备起一个便于识别的名称"
            >
              <Input
                placeholder="例如：客厅音箱、书房助手"
                size="large"
              />
            </Form.Item>

            <Form.Item style={{ marginBottom: 0, marginTop: 24 }}>
              <Button
                type="primary"
                htmlType="submit"
                loading={loading}
                block
                size="large"
              >
                确认绑定
              </Button>
            </Form.Item>
          </Form>

          {/* 帮助信息 */}
          <Divider style={{ margin: '12px 0' }} />
          <div style={{ textAlign: 'center' }}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              激活码有效期为 5 分钟，过期后请在设备上重新获取
            </Text>
          </div>
        </Space>
      )}
    </Modal>
  );
}
