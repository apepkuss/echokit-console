import { useState } from 'react';
import { Card, Form, Input, Button, Typography, message, Descriptions } from 'antd';
import { UserOutlined, LockOutlined, MailOutlined } from '@ant-design/icons';
import { useAuthStore } from '../stores';
import type { UpdateUserRequest, ChangePasswordRequest } from '../types';

const { Title, Text } = Typography;

export function Settings() {
  const { user, updateUser, changePassword, isLoading } = useAuthStore();
  const [profileForm] = Form.useForm();
  const [passwordForm] = Form.useForm();
  const [updatingProfile, setUpdatingProfile] = useState(false);
  const [updatingPassword, setUpdatingPassword] = useState(false);

  const handleUpdateProfile = async (values: UpdateUserRequest) => {
    setUpdatingProfile(true);
    try {
      await updateUser(values);
      message.success('个人信息已更新');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '更新失败，请稍后重试';
      message.error(errorMessage);
    } finally {
      setUpdatingProfile(false);
    }
  };

  const handleChangePassword = async (values: ChangePasswordRequest & { confirmPassword: string }) => {
    const { confirmPassword, ...passwordData } = values;
    setUpdatingPassword(true);
    try {
      await changePassword(passwordData);
      message.success('密码已修改');
      passwordForm.resetFields();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '修改密码失败，请检查当前密码是否正确';
      message.error(errorMessage);
    } finally {
      setUpdatingPassword(false);
    }
  };

  if (!user) {
    return null;
  }

  return (
    <div style={{ padding: 24, maxWidth: 800, margin: '0 auto' }}>
      <Title level={2} style={{ marginBottom: 24 }}>
        账户设置
      </Title>

      {/* 账户信息 */}
      <Card style={{ marginBottom: 24 }}>
        <Title level={4}>账户信息</Title>
        <Descriptions column={1} bordered>
          <Descriptions.Item label={<><MailOutlined /> 邮箱</>}>
            {user.email}
          </Descriptions.Item>
          <Descriptions.Item label="用户 ID">
            <Text copyable code>{user.id}</Text>
          </Descriptions.Item>
          <Descriptions.Item label="注册时间">
            {new Date(user.createdAt * 1000).toLocaleString()}
          </Descriptions.Item>
        </Descriptions>
      </Card>

      {/* 个人信息 */}
      <Card style={{ marginBottom: 24 }}>
        <Title level={4}>个人信息</Title>
        <Form
          form={profileForm}
          layout="vertical"
          onFinish={handleUpdateProfile}
          initialValues={{ name: user.name || '' }}
        >
          <Form.Item
            name="name"
            label="名称"
            rules={[
              { max: 50, message: '名称不能超过 50 个字符' },
            ]}
          >
            <Input
              prefix={<UserOutlined />}
              placeholder="请输入名称"
            />
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={updatingProfile || isLoading}
            >
              保存修改
            </Button>
          </Form.Item>
        </Form>
      </Card>

      {/* 修改密码 */}
      <Card>
        <Title level={4}>修改密码</Title>
        <Form
          form={passwordForm}
          layout="vertical"
          onFinish={handleChangePassword}
        >
          <Form.Item
            name="currentPassword"
            label="当前密码"
            rules={[
              { required: true, message: '请输入当前密码' },
            ]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder="请输入当前密码"
            />
          </Form.Item>

          <Form.Item
            name="newPassword"
            label="新密码"
            rules={[
              { required: true, message: '请输入新密码' },
              { min: 6, message: '密码至少需要 6 个字符' },
            ]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder="请输入新密码"
            />
          </Form.Item>

          <Form.Item
            name="confirmPassword"
            label="确认新密码"
            dependencies={['newPassword']}
            rules={[
              { required: true, message: '请确认新密码' },
              ({ getFieldValue }) => ({
                validator(_, value) {
                  if (!value || getFieldValue('newPassword') === value) {
                    return Promise.resolve();
                  }
                  return Promise.reject(new Error('两次输入的密码不一致'));
                },
              }),
            ]}
          >
            <Input.Password
              prefix={<LockOutlined />}
              placeholder="请再次输入新密码"
            />
          </Form.Item>

          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={updatingPassword || isLoading}
            >
              修改密码
            </Button>
          </Form.Item>
        </Form>
      </Card>
    </div>
  );
}

export default Settings;
