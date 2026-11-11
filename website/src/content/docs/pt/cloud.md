---
title: Volumes na nuvem
description: Monte S3, Google Drive, Dropbox, OneDrive, Azure Blob, GCS e outros como volumes.
order: 7
group: Usando o WingDrive
---

O WingDrive trata o armazenamento na nuvem como **volumes** nativos. Conecte um serviço e ele se comportará como qualquer outra unidade: indexada, pesquisável e navegável, com as mesmas operações de arquivo.

## Serviços compatíveis

O suporte à nuvem é baseado no [OpenDAL](https://opendal.apache.org/), que abrange mais de 40 serviços:

- **Armazenamento de objetos**: Amazon S3, Azure Blob, Google Cloud Storage, MinIO, Backblaze B2, Cloudflare R2
- **Nuvem para consumidores**: Google Drive, Dropbox, OneDrive e outros
- **Auto-hospedados e empresariais**: WebDAV, Nextcloud, Seafile e outros

## Conecte um serviço

No aplicativo desktop, use **Add Storage** na barra lateral. Pela CLI, a configuração interativa orienta você:

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
gdrive://My Drive/Documents/contract.pdf
onedrive://Documents/budget.xlsx
dropbox://Apps/WingDrive/backup.zip
azblob://container/data/export.csv
gcs://bucket/logs/app.log
```

Copie um caminho do WingDrive e cole-o na AWS CLI, no `gsutil` ou em uma ferramenta semelhante.

## Navegação offline

Depois que um volume na nuvem é indexado, você pode pesquisá-lo, navegar pelas miniaturas e ler metadados offline. Apenas a leitura ou gravação do conteúdo dos arquivos exige conexão.

## Identidade de conteúdo na nuvem

O WingDrive identifica arquivos pelo conteúdo com BLAKE3, onde quer que estejam. Para arquivos com mais de 100 KB, ele amostra apenas 58 KB (cabeçalho, quatro trechos igualmente espaçados e rodapé); assim, identificar um arquivo de 10 GB na nuvem transfere 58 KB, não 10 GB. Uma foto no Google Drive e sua cópia no notebook compartilham a mesma identidade.

## Credenciais

As credenciais de nuvem são criptografadas com XChaCha20-Poly1305 usando a chave da sua biblioteca e armazenadas no chaveiro do sistema operacional (Secret Service no Linux, Credential Manager no Windows, Keychain no macOS). Cada biblioteca mantém credenciais separadas. Remover um volume na nuvem apaga suas credenciais.
