# EchoKit Console

> [!WARNING]
> 当前版本的 EchoKit Console 仍在开发中，功能和界面可能会有较大变动，建议仅用于测试和评估目的。

## 快速开始 (Docker Compose 部署)

<p align="center">
  <a href="https://www.youtube.com/watch?v=aebEQSI092s">
    <img
      src="https://img.youtube.com/vi/aebEQSI092s/hqdefault.jpg"
      alt="EchoKit Console Demo"
      width="600"
    />
  </a>
</p>

<p align="center">
  <em>点击观看 EchoKit Console Demo: Deployment, Device Registration & Custom Server Setup</em>
</p>

- Step 1: 部署服务

  参考 [EchoKit Console Docker Compose 部署指南](docs/docker-compose-deployment.md) 部署 EchoKit Console 服务，并访问 EchoKit Console 创建用户账号。

- Step 2: 烧录设备

  由于当前使用的设备固件是开发版，因此需要通过命令行，手动烧录固件。

  - 克隆固件仓库：

    ```bash
    git clone -b feat-xiaozhi git@github.com:second-state/echokit_box.git
    ```

  - 编译固件：

    ```bash
    cd echokit_box/

    cargo build --release
    ```

  - 烧录固件：

    ```bash
    # 清除当前设备中的固件
    espflash erase-flash

    # 烧录新固件
    espflash flash --monitor --flash-size 16mb echokit
    ```

  - 参考文档：[烧录固件](https://echokit.dev/docs/hardware/flash-firmware/)

- Step 3: 设备配网

  在 `https://echokit.dev/setup/` 页面配网时，`EchoKit Server WebSocket URL` 一栏填写 EchoKit Proxy 服务的地址。 例如：`docker-compose.yml` 中 `proxy` 配置的 `EXTERNAL_HOST` 和 `EXTERNAL_PORT` 分别为 `192.168.0.104` 和 `10086`，则填写：`ws://192.168.0.104:10086/ws`。

- Step 4: 注册新设备

  使用用户账号登录 EchoKit Console，进入“设备管理”页面，点击“注册新设备”，填写设备激活码，完成设备注册。

- Step 5: 创建自定义 EchoKit Server 容器

  使用用户账号登录 EchoKit Console，进入“服务器”页面，点击左侧边栏“+”，填写 ASR、LLM、TTS 等服务信息，然后点击底部的【部署】按钮，系统会自动创建并启动自定义 EchoKit Server 容器。

- Step 6: 连接设备到自定义 EchoKit Server

  使用用户账号登录 EchoKit Console，进入“设备管理”页面。在设备列表中，在相关设备的【EchoKit 服务器】下拉列表中，选择刚刚创建的自定义 EchoKit Server，稍等片刻，设备会自动连接到新的 EchoKit Server。
