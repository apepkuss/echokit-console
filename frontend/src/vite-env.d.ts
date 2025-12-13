/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** API 后端地址 */
  readonly VITE_API_BASE_URL?: string;
  /** Proxy WebSocket URL（设备配网时使用） */
  readonly VITE_PROXY_WS_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
