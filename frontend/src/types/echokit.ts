// EchoKit Server 配置类型定义

export type ASRPlatform = 'Openai' | 'Paraformer';

// 各平台 ASR 配置
export interface OpenaiASRConfig {
  platform: 'Openai';
  apiKey: string;
  model: string;
  lang: string;
  prompt?: string;
  url?: string;
}

export interface ParaformerASRConfig {
  platform: 'Paraformer';
  paraformerToken: string;
}

export type ASRConfig = OpenaiASRConfig | ParaformerASRConfig;

export interface LLMConfig {
  url: string;
  apiKey: string;
  model: string;
  systemPrompt: string;
  history?: number;
}

export type TTSPlatform =
  | 'Openai'
  | 'Groq'
  | 'Elevenlabs'
  | 'GSV'
  | 'Fish'
  | 'StreamGSV'
  | 'CosyVoice';

// 各平台 TTS 配置
export interface OpenaiTTSConfig {
  platform: 'Openai';
  apiKey: string;
  model: string;
  voice: string;
  url?: string;
}

export interface GroqTTSConfig {
  platform: 'Groq';
  apiKey: string;
  model: string;
  voice: string;
  url?: string;
}

export interface ElevenlabsTTSConfig {
  platform: 'Elevenlabs';
  token: string;
  voice: string;
  modelId?: string;
  languageCode?: string;
}

export interface GSVTTSConfig {
  platform: 'GSV';
  url: string;
  speaker: string;
  apiKey?: string;
  timeoutSec?: number;
}

export interface StreamGSVTTSConfig {
  platform: 'StreamGSV';
  url: string;
  speaker: string;
  apiKey?: string;
}

export interface FishTTSConfig {
  platform: 'Fish';
  apiKey: string;
  speaker: string;
}

export interface CosyVoiceTTSConfig {
  platform: 'CosyVoice';
  token: string;
  speaker?: string;
  version?: string;
}

export type TTSConfig =
  | OpenaiTTSConfig
  | GroqTTSConfig
  | ElevenlabsTTSConfig
  | GSVTTSConfig
  | StreamGSVTTSConfig
  | FishTTSConfig
  | CosyVoiceTTSConfig;

export interface EchoKitConfig {
  name: string;
  asr: ASRConfig;
  llm: LLMConfig;
  tts: TTSConfig;
}

export interface DeployRequest {
  config: EchoKitConfig;
  port?: number;
}

export type HealthStatus = 'healthy' | 'unhealthy' | 'unknown';

export interface HealthCheckResult {
  status: HealthStatus;
  httpReachable: boolean;
  containerRunning: boolean;
  errorMessage?: string;
  logsTail?: string;
}

export interface DeployResponse {
  containerId: string;
  containerName: string;
  port: number;
  wsUrl: string;
  status: ContainerStatus;
  health: HealthCheckResult;
}

export type ContainerStatus = 'running' | 'stopped' | 'error' | 'creating' | 'starting';

export interface ContainerInfo {
  id: string;
  name: string;
  port: number;
  wsUrl: string;
  status: ContainerStatus;
  createdAt: string;
  health?: HealthCheckResult;
}
