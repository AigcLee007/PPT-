# 宝塔面板 Docker 部署指南

本文按宝塔面板目录 `/www/wwwroot/ppt` 编写，适合把本项目部署到 Linux 云服务器。

## 1. 服务器准备

确保服务器已经具备以下条件：

- 已安装宝塔面板
- 已安装 Docker
- 已安装 Docker Compose 插件
- 已安装 Git
- 域名已经解析到你的云服务器

如果还没安装 Git，可以在服务器终端执行：

```bash
apt update && apt install -y git
```

如果是 CentOS：

```bash
yum install -y git
```

## 2. 拉取项目到宝塔目录

进入宝塔终端，执行：

```bash
cd /www/wwwroot
git clone https://github.com/AigcLee007/PPT-.git ppt
cd /www/wwwroot/ppt
```

如果目录已经存在且是空目录，也可以这样做：

```bash
cd /www/wwwroot/ppt
git init
git remote add origin https://github.com/AigcLee007/PPT-.git
git fetch origin
git checkout -b main origin/main
```

## 3. 创建环境配置

在项目根目录创建 `.env`：

```bash
cd /www/wwwroot/ppt
cp .env.example .env
```

然后编辑 `.env`：

```bash
nano .env
```

至少修改这些配置：

```env
AI_PROVIDER_FORMAT=openai

OPENAI_API_KEY=你的API_KEY
OPENAI_API_BASE=https://api.aittco.com

TEXT_MODEL=gpt-4o-mini
IMAGE_MODEL=gpt-image-1
IMAGE_CAPTION_MODEL=gpt-4o-mini

SECRET_KEY=请改成你自己的复杂字符串
FLASK_ENV=production

BACKEND_PORT=5000
FRONTEND_PORT=3000

CORS_ORIGINS=*
OUTPUT_LANGUAGE=zh
```

如果你实际使用的是 Gemini 格式中转，也可以改成：

```env
AI_PROVIDER_FORMAT=gemini
GOOGLE_API_KEY=你的API_KEY
GOOGLE_API_BASE=https://api.aittco.com
```

说明：

- `OPENAI_API_BASE` 我已经改成默认 `https://api.aittco.com`
- 如果你只走 OpenAI 兼容接口，优先用 `AI_PROVIDER_FORMAT=openai`
- `FRONTEND_PORT=3000` 表示前端容器映射到服务器 `3000` 端口，后面宝塔反向代理到它

## 4. 启动 Docker 容器

第一次部署执行：

```bash
cd /www/wwwroot/ppt
docker compose up -d --build
```

查看运行状态：

```bash
docker compose ps
```

查看日志：

```bash
docker compose logs -f
```

如果只看后端日志：

```bash
docker compose logs -f backend
```

如果只看前端日志：

```bash
docker compose logs -f frontend
```

## 5. 放行端口

在宝塔安全或服务器安全组中放行：

- `3000` 前端访问端口
- `80` 和 `443` 网站访问端口

如果你不希望外部直接访问 `3000`，可以只让宝塔反代使用它，同时在云服务器安全组里不开放 `3000` 公网访问。

## 6. 宝塔配置站点

在宝塔面板中：

1. 新建站点
2. 绑定你的域名，例如 `ppt.yourdomain.com`
3. PHP 版本选择“纯静态”或不启用 PHP
4. 创建完成后，进入站点设置

## 7. 配置反向代理到 Docker 前端

本项目的前端容器内部已经通过 Nginx 代理 `/api` 到后端容器，所以宝塔只需要把域名反代到前端容器端口即可。

在宝塔站点设置中添加反向代理：

- 代理名称：`ppt`
- 目标 URL：`http://127.0.0.1:3000`
- 发送域名：`$host`

如果你使用 HTTPS，再在宝塔里给该站点申请 SSL 证书，并开启强制 HTTPS。

## 8. 验证是否部署成功

浏览器访问你的域名后，应该能打开项目首页。

也可以在服务器执行：

```bash
curl http://127.0.0.1:3000
curl http://127.0.0.1:5000/health
```

如果后端健康检查返回成功，说明服务已正常启动。

## 9. 常用维护命令

更新代码并重建：

```bash
cd /www/wwwroot/ppt
git pull origin main
docker compose up -d --build
```

停止服务：

```bash
cd /www/wwwroot/ppt
docker compose down
```

重启服务：

```bash
cd /www/wwwroot/ppt
docker compose restart
```

查看容器：

```bash
docker ps
```

## 10. 数据目录说明

以下目录会在宿主机持久化：

- `/www/wwwroot/ppt/backend/instance`
- `/www/wwwroot/ppt/uploads`

建议你定期备份这两个目录。

## 11. 常见问题

### 1. 页面能打开，但生成失败

优先检查：

```bash
docker compose logs -f backend
```

通常是以下原因：

- `OPENAI_API_KEY` 没填对
- `OPENAI_API_BASE` 不可用
- 模型名称不支持

### 2. 域名打开 502

优先检查：

```bash
docker compose ps
docker compose logs -f frontend
```

再确认宝塔反代目标是不是：

```text
http://127.0.0.1:3000
```

### 3. 修改了 `.env` 但没有生效

执行：

```bash
cd /www/wwwroot/ppt
docker compose down
docker compose up -d --build
```

### 4. 拉取代码失败

如果你的服务器无法直接访问 GitHub，可以先在本地打包后上传到：

```text
/www/wwwroot/ppt
```

然后在该目录里执行：

```bash
docker compose up -d --build
```

