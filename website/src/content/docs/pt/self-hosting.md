---
title: Auto-hospedagem
description: Execute o wing-server em um NAS, servidor doméstico ou VPS, com Docker ou systemd.
order: 10
group: Avançado
---

`wing-server` incorpora o daemon completo do WingDrive, com processamento de mídia, e disponibiliza a interface web e a API RPC por HTTP. Seus dados permanecem no seu hardware enquanto você os acessa de qualquer navegador.

## Docker

Há imagens para x86_64 e ARM64, executadas por um usuário sem privilégios de root.

```bash
docker run -d \
  --name wingdrive \
  -p 8080:8080 \
  -p 7373:7373 \
  -v wingdrive-data:/data \
  -e WING_AUTH=admin:your-secure-password \
  ghcr.io/bernardopg/wingdrive/server:latest
```

A porta 8080 atende HTTP; a porta 7373 é usada para conexões ponto a ponto com seus dispositivos. Monte outros volumes para indexar armazenamento existente.

## Docker Compose

```yaml
services:
  wingdrive:
    image: ghcr.io/bernardopg/wingdrive/server:latest
    ports:
      - "8080:8080"
      - "7373:7373"
    volumes:
      - wingdrive-data:/data
      - /mnt/media:/media:ro
    environment:
      WING_AUTH: "admin:your-password"
      TZ: "America/Sao_Paulo"
    restart: unless-stopped

volumes:
  wingdrive-data:
```

## systemd

Crie `/etc/systemd/system/wingdrive.service`:

```ini
[Unit]
Description=WingDrive Server
After=network.target

[Service]
Type=simple
User=wingdrive
Group=wingdrive
Environment="DATA_DIR=/var/lib/wingdrive"
Environment="WING_AUTH=admin:your-secure-password"
ExecStart=/usr/local/bin/wingdrive-server --data-dir /var/lib/wingdrive
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

```bash
sudo useradd -r -s /bin/false wingdrive
sudo mkdir -p /var/lib/wingdrive
sudo chown wingdrive:wingdrive /var/lib/wingdrive
sudo systemctl enable --now wingdrive
```

## Configuração

| Configuração | Significado |
|---|---|
| `WING_AUTH` | Usuários da autenticação básica, `user1:pass1,user2:pass2`. `disabled` desativa a autenticação (não recomendado em uma rede). |
| `DATA_DIR` | Onde ficam bibliotecas, miniaturas e logs. Sempre defina em produção. |
| `PORT` / `--port` | Porta HTTP, 8080 por padrão. |

> O servidor se recusa a iniciar sem `WING_AUTH`, a menos que você desative a autenticação explicitamente. Isso evita expor suas bibliotecas acidentalmente.

## Proxy reverso

O servidor escuta em todas as interfaces. Para acesso pela internet, coloque-o atrás de um proxy reverso com TLS. Um site Nginx mínimo:

```nginx
server {
    listen 443 ssl http2;
    server_name files.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        client_max_body_size 0;
    }
}
```
