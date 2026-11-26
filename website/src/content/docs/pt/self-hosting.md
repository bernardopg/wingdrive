---
title: Auto-hospedagem
description: Execute o wing-server em um NAS, servidor doméstico ou VPS, com Docker ou systemd.
order: 10
group: Avançado
---

`wing-server` incorpora o daemon completo do WingDrive, com processamento de mídia, e disponibiliza a interface web e a API RPC por HTTP. Seus dados permanecem no seu hardware enquanto você os acessa de qualquer navegador.

## Docker

Há imagens para x86_64 e ARM64, executadas por um usuário sem privilégios de root e com verificação de saúde própria. O arquivo compose do repositório é a instalação inteira:

```bash
curl -O https://raw.githubusercontent.com/bernardopg/wingdrive/main/apps/server/docker-compose.yml
WING_AUTH='eu:uma-senha-longa' docker compose up -d
```

Abra `http://<servidor>:8080` e entre com essas credenciais. Sem `WING_AUTH` o arquivo não inicia, então não existe senha padrão. Para indexar armazenamento existente, descomente a montagem no arquivo e adicione a pasta como local na interface web. Atualize com `docker compose pull && docker compose up -d`.

O contêiner usa a rede do host: outros dispositivos encontram o servidor para o pareamento por mDNS, e a sincronização usa uma porta UDP escolhida na inicialização, então publicar portas não basta. A rede do host só existe no Linux.

Sem compose:

```bash
docker run -d --name wingdrive --network host --restart unless-stopped \
  -v wingdrive-data:/data -e WING_AUTH='eu:uma-senha-longa' \
  ghcr.io/bernardopg/wingdrive/server:latest
```

## Arquivo da versão

Sem um runtime de contêineres, use o arquivo `wing-server-<versão>-linux-x86_64` da [página de download](/pt/download/) (`.tar.gz`, `.tar.xz` ou `.zip`). Ele traz `bin/wing-server`, a CLI `bin/wing` e as bibliotecas FFmpeg e libheif que eles carregam em `lib/wingdrive/`, e exige glibc 2.39 ou mais recente.

```bash
VERSION=3.8.1
sudo tar -xJf wing-server-$VERSION-linux-x86_64.tar.xz -C /opt
sudo ln -sfn /opt/wing-server-$VERSION-linux-x86_64 /opt/wingdrive-server
```

Mantenha `bin/` e `lib/` juntos. Para atualizar, extraia o novo arquivo e mova o link `/opt/wingdrive-server`.

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
ExecStart=/opt/wingdrive-server/bin/wing-server --data-dir /var/lib/wingdrive
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
