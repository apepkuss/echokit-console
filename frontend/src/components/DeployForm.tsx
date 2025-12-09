import { useState } from 'react';
import { Form, Input, Select, Button, Collapse, message, Alert, Modal, Typography } from 'antd';
import { RocketOutlined } from '@ant-design/icons';
import type { EchoKitConfig, TTSPlatform, ASRPlatform, DeployResponse } from '../types';
import { useDeployStore } from '../stores';

const { TextArea } = Input;

const TTS_PLATFORMS: { value: TTSPlatform; label: string }[] = [
  { value: 'Openai', label: 'OpenAI' },
  { value: 'Groq', label: 'Groq' },
  { value: 'Elevenlabs', label: 'ElevenLabs' },
  { value: 'GSV', label: 'GSV (GPT-SoVITS)' },
  { value: 'Fish', label: 'Fish TTS' },
  { value: 'StreamGSV', label: 'StreamGSV' },
  { value: 'CosyVoice', label: 'CosyVoice (阿里百炼)' },
];

const ASR_PLATFORMS: { value: ASRPlatform; label: string }[] = [
  { value: 'Openai', label: 'OpenAI (Whisper)' },
  { value: 'Paraformer', label: 'Paraformer (阿里)' },
];

const LANGUAGES = [
  { value: 'auto', label: '自动检测' },
  { value: 'zh', label: '中文' },
  { value: 'en', label: '英文' },
];

interface DeployFormProps {
  onSuccess?: () => void;
}

// API 错误响应类型
interface ApiErrorResponse {
  error: string;
  message: string;
}

export function DeployForm({ onSuccess }: DeployFormProps) {
  const [form] = Form.useForm<EchoKitConfig>();
  const { deploy, deploying } = useDeployStore();
  const [deployResult, setDeployResult] = useState<DeployResponse | null>(null);
  const [logsModalVisible, setLogsModalVisible] = useState(false);
  const [deployError, setDeployError] = useState<string | null>(null);
  const [ttsPlatform, setTtsPlatform] = useState<TTSPlatform>('Openai');
  const [asrPlatform, setAsrPlatform] = useState<ASRPlatform>('Openai');

  const handleSubmit = async (values: EchoKitConfig) => {
    setDeployError(null);
    setDeployResult(null);

    try {
      const result = await deploy(values);
      setDeployResult(result);

      if (result.health.status === 'healthy') {
        message.success(`部署成功！WebSocket 地址: ${result.wsUrl}`);
        form.resetFields();
        setTtsPlatform('Openai');
        setAsrPlatform('Openai');
        onSuccess?.();
      } else {
        message.warning('容器已创建，但健康检查未通过');
      }
    } catch (err) {
      // 尝试从 axios 错误中提取详细信息
      let errorMessage = '部署失败';
      if (err && typeof err === 'object') {
        const axiosError = err as { response?: { data?: ApiErrorResponse }; message?: string };
        if (axiosError.response?.data?.message) {
          errorMessage = axiosError.response.data.message;
        } else if (axiosError.message) {
          errorMessage = axiosError.message;
        }
      }
      setDeployError(errorMessage);
      message.error('部署失败');
    }
  };

  const handlePlatformChange = (platform: TTSPlatform) => {
    setTtsPlatform(platform);
    // 清除之前平台的字段值，保留 platform
    // 使用 any 类型绕过联合类型的限制
    form.setFieldsValue({
      tts: {
        platform,
        apiKey: undefined,
        token: undefined,
        url: undefined,
        model: undefined,
        voice: undefined,
        speaker: undefined,
      } as never,
    });
  };

  const handleAsrPlatformChange = (platform: ASRPlatform) => {
    setAsrPlatform(platform);
    // 清除之前平台的字段值，保留 platform
    form.setFieldsValue({
      asr: {
        platform,
        apiKey: undefined,
        paraformerToken: undefined,
        model: undefined,
        lang: undefined,
        prompt: undefined,
        url: undefined,
      } as never,
    });
  };

  const renderDeployError = () => {
    if (!deployError) return null;

    return (
      <Alert
        type="error"
        message="部署失败"
        description={
          <pre
            style={{
              margin: 0,
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              fontSize: 12,
            }}
          >
            {deployError}
          </pre>
        }
        closable
        onClose={() => setDeployError(null)}
        style={{ marginBottom: 16 }}
      />
    );
  };

  const renderDeployResult = () => {
    if (!deployResult) return null;

    const { health, containerName, wsUrl } = deployResult;

    if (health.status === 'healthy') {
      return (
        <Alert
          type="success"
          message="部署成功"
          description={
            <div>
              <div>容器名称: {containerName}</div>
              <div>WebSocket 地址: {wsUrl}</div>
            </div>
          }
          closable
          onClose={() => setDeployResult(null)}
          style={{ marginBottom: 16 }}
        />
      );
    }

    // 根据状态确定错误类型和建议
    const getStatusInfo = () => {
      if (!health.containerRunning) {
        return {
          title: '容器启动失败',
          suggestion: '容器在启动后异常退出，请查看日志了解具体错误原因。',
        };
      }
      if (!health.httpReachable) {
        return {
          title: '服务未响应',
          suggestion: '容器正在运行，但服务未响应 HTTP 请求。可能是服务仍在初始化或配置错误。',
        };
      }
      return {
        title: '部署异常',
        suggestion: '请检查配置是否正确。',
      };
    };

    const statusInfo = getStatusInfo();

    return (
      <Alert
        type="error"
        message={statusInfo.title}
        description={
          <div style={{ lineHeight: 1.8 }}>
            <div style={{ marginBottom: 8 }}>
              <strong>容器名称:</strong> {containerName}
            </div>
            <div
              style={{
                background: '#fff2f0',
                border: '1px solid #ffccc7',
                borderRadius: 4,
                padding: '8px 12px',
                marginBottom: 8,
              }}
            >
              <div style={{ fontWeight: 500, marginBottom: 4 }}>错误信息:</div>
              <div style={{ color: '#cf1322' }}>{health.errorMessage || '未知错误'}</div>
            </div>
            <div style={{ fontSize: 12, color: '#666', marginBottom: 8 }}>
              💡 {statusInfo.suggestion}
            </div>
            <div style={{ display: 'flex', gap: 16, fontSize: 12 }}>
              <span>
                容器状态:{' '}
                <span style={{ color: health.containerRunning ? '#52c41a' : '#ff4d4f' }}>
                  {health.containerRunning ? '运行中' : '已停止'}
                </span>
              </span>
              <span>
                HTTP 响应:{' '}
                <span style={{ color: health.httpReachable ? '#52c41a' : '#ff4d4f' }}>
                  {health.httpReachable ? '正常' : '无响应'}
                </span>
              </span>
            </div>
            {health.logsTail && (
              <Button
                type="primary"
                danger
                size="small"
                style={{ marginTop: 12 }}
                onClick={() => setLogsModalVisible(true)}
              >
                查看详细日志
              </Button>
            )}
          </div>
        }
        closable
        onClose={() => setDeployResult(null)}
        style={{ marginBottom: 16 }}
      />
    );
  };

  // 根据平台渲染 TTS 配置项
  const renderTTSFields = () => {
    switch (ttsPlatform) {
      case 'Openai':
        return (
          <>
            <Form.Item
              name={['tts', 'apiKey']}
              label="API Key"
              rules={[{ required: true, message: '请输入 API Key' }]}
            >
              <Input.Password placeholder="sk-xxx" />
            </Form.Item>
            <Form.Item
              name={['tts', 'model']}
              label="模型"
              initialValue="gpt-4o-mini-tts"
              rules={[{ required: true, message: '请输入模型名称' }]}
            >
              <Input placeholder="gpt-4o-tts / gpt-4o-mini-tts" />
            </Form.Item>
            <Form.Item
              name={['tts', 'voice']}
              label="Voice (音色)"
              initialValue="alloy"
              rules={[{ required: true, message: '请输入音色' }]}
            >
              <Input placeholder="alloy / nova / echo / onyx / fable / shimmer" />
            </Form.Item>
            <Form.Item
              name={['tts', 'url']}
              label="API 端点"
              initialValue="https://api.openai.com/v1/audio/speech"
            >
              <Input placeholder="https://api.openai.com/v1/audio/speech" />
            </Form.Item>
          </>
        );

      case 'Groq':
        return (
          <>
            <Form.Item
              name={['tts', 'apiKey']}
              label="API Key"
              rules={[{ required: true, message: '请输入 API Key' }]}
            >
              <Input.Password placeholder="gsk_xxx" />
            </Form.Item>
            <Form.Item
              name={['tts', 'model']}
              label="模型"
              initialValue="playai-tts"
              rules={[{ required: true, message: '请输入模型名称' }]}
            >
              <Input placeholder="playai-tts" />
            </Form.Item>
            <Form.Item
              name={['tts', 'voice']}
              label="Voice (音色)"
              rules={[{ required: true, message: '请输入音色' }]}
            >
              <Input placeholder="Fritz-PlayAI / Arista-PlayAI" />
            </Form.Item>
            <Form.Item
              name={['tts', 'url']}
              label="API 端点"
              initialValue="https://api.groq.com/openai/v1/audio/speech"
            >
              <Input placeholder="https://api.groq.com/openai/v1/audio/speech" />
            </Form.Item>
          </>
        );

      case 'Elevenlabs':
        return (
          <>
            <Form.Item
              name={['tts', 'token']}
              label="Token"
              rules={[{ required: true, message: '请输入 Token' }]}
            >
              <Input.Password placeholder="ElevenLabs API Token" />
            </Form.Item>
            <Form.Item
              name={['tts', 'voice']}
              label="Voice (音色)"
              rules={[{ required: true, message: '请输入音色 ID' }]}
            >
              <Input placeholder="Voice ID" />
            </Form.Item>
            <Form.Item
              name={['tts', 'modelId']}
              label="Model ID"
              tooltip="可选，留空使用默认模型"
            >
              <Input placeholder="eleven_multilingual_v2 (可选)" />
            </Form.Item>
            <Form.Item
              name={['tts', 'languageCode']}
              label="Language Code"
              tooltip="可选，用于多语言模型指定输出语言"
            >
              <Input placeholder="zh / en (可选)" />
            </Form.Item>
          </>
        );

      case 'GSV':
        return (
          <>
            <Form.Item
              name={['tts', 'url']}
              label="API 端点"
              rules={[{ required: true, message: '请输入 API 端点' }]}
            >
              <Input placeholder="http://localhost:9094/v1/audio/speech" />
            </Form.Item>
            <Form.Item
              name={['tts', 'speaker']}
              label="Speaker (说话人)"
              rules={[{ required: true, message: '请输入说话人' }]}
            >
              <Input placeholder="cooper / default" />
            </Form.Item>
            <Form.Item
              name={['tts', 'apiKey']}
              label="API Key"
              tooltip="可选，用于需要认证的 GSV 服务"
            >
              <Input.Password placeholder="GSV API Key (可选)" />
            </Form.Item>
            <Form.Item
              name={['tts', 'timeoutSec']}
              label="超时时间 (秒)"
              tooltip="可选，请求超时时间"
            >
              <Input type="number" placeholder="30 (可选)" />
            </Form.Item>
          </>
        );

      case 'StreamGSV':
        return (
          <>
            <Form.Item
              name={['tts', 'url']}
              label="API 端点"
              rules={[{ required: true, message: '请输入 API 端点' }]}
            >
              <Input placeholder="http://localhost:9094/v1/audio/stream_speech" />
            </Form.Item>
            <Form.Item
              name={['tts', 'speaker']}
              label="Speaker (说话人)"
              rules={[{ required: true, message: '请输入说话人' }]}
            >
              <Input placeholder="cooper / default" />
            </Form.Item>
            <Form.Item
              name={['tts', 'apiKey']}
              label="API Key"
              tooltip="可选，用于需要认证的 StreamGSV 服务"
            >
              <Input.Password placeholder="StreamGSV API Key (可选)" />
            </Form.Item>
          </>
        );

      case 'Fish':
        return (
          <>
            <Form.Item
              name={['tts', 'apiKey']}
              label="API Key"
              rules={[{ required: true, message: '请输入 API Key' }]}
            >
              <Input.Password placeholder="Fish TTS API Key" />
            </Form.Item>
            <Form.Item
              name={['tts', 'speaker']}
              label="Speaker (说话人)"
              rules={[{ required: true, message: '请输入说话人' }]}
            >
              <Input placeholder="说话人名称" />
            </Form.Item>
          </>
        );

      case 'CosyVoice':
        return (
          <>
            <Form.Item
              name={['tts', 'token']}
              label="Token"
              rules={[{ required: true, message: '请输入 Token' }]}
            >
              <Input.Password placeholder="阿里百炼 API Key" />
            </Form.Item>
            <Form.Item
              name={['tts', 'speaker']}
              label="Speaker (说话人)"
              tooltip="可选，留空使用默认音色"
            >
              <Input placeholder="longhua_v2 / longyuan_v2 (可选)" />
            </Form.Item>
            <Form.Item
              name={['tts', 'version']}
              label="Version (版本)"
              tooltip="可选，指定 CosyVoice 版本"
            >
              <Input placeholder="v2 (可选)" />
            </Form.Item>
          </>
        );

      default:
        return null;
    }
  };

  // 根据平台渲染 ASR 配置项
  const renderASRFields = () => {
    switch (asrPlatform) {
      case 'Openai':
        return (
          <>
            <Form.Item
              name={['asr', 'apiKey']}
              label="API Key"
              rules={[{ required: true, message: '请输入 API Key' }]}
            >
              <Input.Password placeholder="sk-xxx" />
            </Form.Item>
            <Form.Item
              name={['asr', 'model']}
              label="模型"
              initialValue="whisper-1"
              rules={[{ required: true, message: '请输入模型名称' }]}
            >
              <Input placeholder="whisper-1" />
            </Form.Item>
            <Form.Item
              name={['asr', 'lang']}
              label="语言"
              initialValue="auto"
              rules={[{ required: true, message: '请选择语言' }]}
            >
              <Select options={LANGUAGES} />
            </Form.Item>
            <Form.Item
              name={['asr', 'prompt']}
              label="Prompt (提示词)"
              tooltip="用于引导 ASR 模型识别特定词汇，每行一个"
            >
              <TextArea
                rows={3}
                placeholder="Hello&#10;你好&#10;(noise)&#10;(bgm)&#10;(silence)"
              />
            </Form.Item>
            <Form.Item
              name={['asr', 'url']}
              label="API 端点"
              initialValue="https://api.openai.com/v1/audio/transcriptions"
            >
              <Input placeholder="https://api.openai.com/v1/audio/transcriptions" />
            </Form.Item>
          </>
        );

      case 'Paraformer':
        return (
          <>
            <Form.Item
              name={['asr', 'paraformerToken']}
              label="Paraformer Token"
              rules={[{ required: true, message: '请输入 Paraformer Token' }]}
            >
              <Input.Password placeholder="sk-xxx" />
            </Form.Item>
          </>
        );

      default:
        return null;
    }
  };

  const collapseItems = [
    {
      key: 'asr',
      label: 'ASR 语音识别配置',
      children: (
        <>
          <Form.Item name={['asr', 'platform']} label="平台" initialValue="Openai">
            <Select options={ASR_PLATFORMS} onChange={handleAsrPlatformChange} />
          </Form.Item>
          {renderASRFields()}
        </>
      ),
    },
    {
      key: 'llm',
      label: 'LLM 大语言模型配置',
      children: (
        <>
          <Form.Item
            name={['llm', 'url']}
            label="API 端点"
            initialValue="https://api.openai.com/v1/chat/completions"
            rules={[{ required: true, message: '请输入 LLM API 端点' }]}
          >
            <Input placeholder="https://api.openai.com/v1/chat/completions" />
          </Form.Item>
          <Form.Item
            name={['llm', 'apiKey']}
            label="API Key"
            rules={[{ required: true, message: '请输入 LLM API Key' }]}
          >
            <Input.Password placeholder="sk-xxx" />
          </Form.Item>
          <Form.Item
            name={['llm', 'model']}
            label="模型"
            rules={[{ required: true, message: '请输入模型名称' }]}
          >
            <Input placeholder="gpt-4o" />
          </Form.Item>
          <Form.Item
            name={['llm', 'systemPrompt']}
            label="系统提示词"
            initialValue="You are a helpful AI assistant."
          >
            <TextArea rows={4} placeholder="输入系统提示词" />
          </Form.Item>
        </>
      ),
    },
    {
      key: 'tts',
      label: 'TTS 语音合成配置',
      children: (
        <>
          <Form.Item name={['tts', 'platform']} label="平台" initialValue="Openai">
            <Select options={TTS_PLATFORMS} onChange={handlePlatformChange} />
          </Form.Item>
          {renderTTSFields()}
        </>
      ),
    },
  ];

  return (
    <div>
      <Typography.Title level={5} style={{ marginBottom: 16 }}>
        部署 EchoKit 服务器
      </Typography.Title>
      {renderDeployError()}
      {renderDeployResult()}
      <Form
        form={form}
        layout="vertical"
        onFinish={handleSubmit}
        autoComplete="off"
        size="small"
      >
        <Form.Item
          name="name"
          label="实例名称"
          rules={[{ required: true, message: '请输入实例名称' }]}
        >
          <Input placeholder="my-echokit" />
        </Form.Item>

        <Collapse
          items={collapseItems}
          defaultActiveKey={['tts']}
          size="small"
          style={{ marginBottom: 16 }}
        />

        <Form.Item style={{ marginBottom: 0 }}>
          <Button
            type="primary"
            htmlType="submit"
            icon={<RocketOutlined />}
            loading={deploying}
            block
          >
            {deploying ? '部署中...' : '部署'}
          </Button>
        </Form.Item>
      </Form>
      <Modal
        title="错误日志"
        open={logsModalVisible}
        onCancel={() => setLogsModalVisible(false)}
        footer={null}
        width={800}
      >
        <pre
          style={{
            maxHeight: 500,
            overflow: 'auto',
            backgroundColor: '#1e1e1e',
            color: '#d4d4d4',
            padding: 16,
            borderRadius: 4,
            fontSize: 12,
            fontFamily: 'Consolas, Monaco, "Courier New", monospace',
          }}
        >
          {deployResult?.health.logsTail || '暂无日志'}
        </pre>
      </Modal>
    </div>
  );
}
