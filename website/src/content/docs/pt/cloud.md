---
title: Volumes na nuvem
description: Conecte o Google Drive e o Google Fotos, e monte S3, Dropbox, OneDrive, Azure Blob e GCS como volumes.
order: 7
group: Usando o WingDrive
---

O WingDrive trata o armazenamento na nuvem como **volumes** nativos. Um serviço conectado aparece na barra lateral e aceita as mesmas operações de qualquer outra unidade: navegar, copiar, mover, renomear, mandar para a lixeira e sincronizar.

## Serviços compatíveis

| Serviço | Como conectar |
| --- | --- |
| **Google Drive** e **Google Fotos** | Entrar com o Google no navegador. Várias contas por biblioteca. |
| Amazon S3 e armazenamento compatível com S3 (Cloudflare R2, MinIO, Wasabi, Backblaze B2, DigitalOcean Spaces) | Bucket, região e chaves de acesso |
| Dropbox, OneDrive | Cliente OAuth e refresh token |
| Azure Blob Storage | Contêiner, nome da conta e chave |
| Google Cloud Storage | Bucket e JSON da conta de serviço |

Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA e outros serviços estão planejados. O roadmap lista todos no épico de integrações.

## Google Drive e Google Fotos

Abra **Configurações > Integrações** e escolha **Conectar uma conta Google**. O navegador abre a página de login do Google; depois que você permitir o acesso, o Drive da conta aparece na barra lateral como `Google Drive (seu@email)`. Repita para adicionar outras contas.

- As pastas do Drive são listadas na hora. Documentos, Planilhas e Apresentações Google aparecem como `.docx`, `.xlsx` e `.pptx` e abrem no navegador.
- Dê dois cliques em um arquivo do Drive para abrir uma cópia local no app de sempre. Cada vez que você salva, ela é enviada de volta. Se o arquivo mudou no Drive enquanto isso, sua versão é guardada como cópia de conflito.
- A **sincronização com a nuvem** mantém uma pasta do computador igual a uma pasta do Drive, nos dois sentidos ou em um só, sob demanda ou a cada alguns minutos.
- Clique com o botão direito em fotos ou vídeos e escolha **Enviar para o Google Fotos**. **Importar do Google Fotos** abre o seletor do Google e baixa o que você escolher em qualidade original.
- Desconectar uma conta revoga o acesso do WingDrive no Google.

Pela CLI:

```bash
wing integrations connect google
wing integrations list
wing integrations cloud-sync add Documentos ~/Documentos gdrive://voce@gmail.com/Backup --every 60
wing integrations photos upload <id-da-conta> ~/Imagens/Viagem --new-album "Viagem"
```

O login no Google precisa de um cliente OAuth. Versões que já trazem um entram direto; nas outras, crie um cliente do tipo "App para computador" no Google Cloud Console e cole-o em **Configurações > Integrações > Cliente de login do Google**.

## Outros serviços

No aplicativo desktop, use **Adicionar armazenamento** na barra lateral. Pela CLI, a configuração interativa orienta você:

```bash
wing cloud
```

Ou adicione um volume diretamente:

```bash
wing volume add-cloud --help
wing volume list
```

## Caminhos que você já conhece

Os caminhos na nuvem usam as mesmas URIs das ferramentas de cada provedor:

```text
s3://my-bucket/photos/vacation.jpg
gdrive://voce@gmail.com/Documentos/contrato.pdf
onedrive://Documents/budget.xlsx
dropbox://Apps/WingDrive/backup.zip
azblob://container/data/export.csv
gcs://bucket/logs/app.log
```

Copie um caminho do WingDrive e cole-o na AWS CLI, no `gsutil` ou em uma ferramenta semelhante.

## Identidade de conteúdo na nuvem

O WingDrive identifica arquivos pelo conteúdo com BLAKE3, onde quer que estejam. Para arquivos com mais de 100 KB, ele amostra apenas 58 KB (cabeçalho, quatro trechos igualmente espaçados e rodapé); assim, identificar um arquivo de 10 GB na nuvem transfere 58 KB, não 10 GB.

## Credenciais

Credenciais de nuvem e acessos do Google são criptografados com XChaCha20-Poly1305 usando a chave da sua biblioteca e guardados no banco de dados da biblioteca. Cada biblioteca mantém credenciais separadas. Remover um volume na nuvem ou desconectar uma conta apaga as credenciais.
