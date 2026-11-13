---
title: Volumes na nuvem
description: Conecte o Google Drive e o Google Fotos, e monte S3, Dropbox, OneDrive, Azure Blob e GCS como volumes.
order: 7
group: Usando o WingDrive
---

O WingDrive trata o armazenamento na nuvem como **volumes** nativos. Um serviço conectado aparece na barra lateral e aceita as mesmas operações de qualquer outro drive: navegar, copiar, mover, renomear, mandar para a lixeira e sincronizar.

## Serviços compatíveis

| Serviço | Como conectar |
| --- | --- |
| **Google Drive** e **Google Fotos** | Entrar com o Google no navegador. Várias contas por biblioteca. |
| Amazon S3 e armazenamento compatível com S3 (Cloudflare R2, MinIO, Wasabi, Backblaze B2, DigitalOcean Spaces) | Bucket, região e chaves de acesso |
| Dropbox, OneDrive | Cliente OAuth e refresh token |
| Azure Blob Storage | Contêiner, nome da conta e chave |
| Google Cloud Storage | Bucket e JSON da conta de serviço |

Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA e outros serviços estão planejados. O roadmap lista todos no épico de integrações.

## Conecte uma conta Google

<ol class="steps">
<li>Abra <strong>Configurações → Integrações</strong> (<kbd>Ctrl</kbd>+<kbd>,</kbd>).</li>
<li>Em <em>Contas Google</em>, deixe <strong>Google Drive</strong> e <strong>Google Fotos</strong> marcados, ou desmarque o que não quiser.</li>
<li>Clique em <strong>Conectar uma conta Google</strong>. O navegador abre a página de login do Google.</li>
<li>Permita o acesso. De volta ao WingDrive, o Drive da conta aparece na barra lateral como <code>Google Drive (voce@gmail.com)</code>.</li>
</ol>

Repita para adicionar outras contas. Cada uma mantém o próprio acesso ao Drive e ao Fotos.

<figure class="shot">
<a href="/img/docs/settings-integrations.webp"><img src="/img/docs/settings-integrations.webp" width="1400" height="900" loading="lazy" alt="Página Integrações das configurações, com contas Google, sincronização com a nuvem e cliente de login do Google"></a>
<figcaption>Configurações → Integrações: contas Google, sincronização com a nuvem e o cliente de login do Google.</figcaption>
</figure>

<div class="callout warn">

<p><strong>“O login precisa de um cliente OAuth do Google.”</strong> Versões que já trazem um cliente do Google entram direto. Se aparecer essa mensagem, siga os quatro passos da mesma página: crie um projeto no Google Cloud Console, ative as APIs Drive, Photos Library e Photos Picker, adicione suas contas como usuários de teste e crie um cliente OAuth do tipo <em>App para computador</em>. Cole o ID e o segredo em <strong>Cliente de login do Google</strong>.</p>

</div>

### O que dá para fazer no Drive

- **Navegar ao vivo.** As pastas vêm direto do Drive. Documentos, Planilhas e Apresentações Google aparecem como `.docx`, `.xlsx` e `.pptx` e abrem no navegador.
- **Editar nos seus apps.** Dê dois cliques em um arquivo do Drive para abrir uma cópia local. Cada vez que você salva, ela é enviada de volta. Se o arquivo mudou no Drive enquanto isso, sua versão é guardada como cópia de conflito.
- **Copiar, mover, renomear, criar pasta e mandar para a lixeira** funcionam entre pastas locais e o Drive, nos dois sentidos. Arquivos arrastados da área de trabalho são enviados.
- A **Visualização Rápida** baixa o arquivo para um cache antes. **Abrir no navegador** abre o item no drive.google.com.

### Mantenha uma pasta sincronizada

<ol class="steps">
<li>Em <strong>Configurações → Integrações</strong>, na seção <em>Sincronização com a nuvem</em>, clique em <strong>Sincronizar uma pasta</strong>.</li>
<li>Escolha a pasta deste dispositivo e a pasta do Drive que vai ficar igual a ela.</li>
<li>Escolha nos dois sentidos ou em um só, e sob demanda ou a cada N minutos.</li>
</ol>

### Google Fotos

Clique com o botão direito em fotos ou vídeos e escolha **Enviar para o Google Fotos**, se quiser em um álbum novo ou existente. **Importar do Google Fotos** abre o seletor do Google e baixa o que você escolher em qualidade original.

Desconectar uma conta apaga a autorização guardada aqui e revoga o acesso do WingDrive no Google.

### Pela CLI

```bash
wing integrations connect google
wing integrations list
wing integrations cloud-sync add Documentos ~/Documentos gdrive://voce@gmail.com/Backup --every 60
wing integrations photos upload <id-da-conta> ~/Imagens/Viagem --new-album "Viagem"
```

## Outros serviços

<ol class="steps">
<li>Na Visão geral, clique em <strong>Adicionar armazenamento</strong> e escolha <strong>Armazenamento na nuvem</strong>.</li>
<li>Escolha o provedor.</li>
<li>Preencha o bucket ou contêiner e as credenciais da tabela acima e confirme. O volume aparece em <em>Volumes</em>.</li>
</ol>

<div class="shots">

<figure class="shot">
<a href="/img/docs/add-storage.webp"><img src="/img/docs/add-storage.webp" width="560" height="500" loading="lazy" alt="Diálogo Adicionar armazenamento"></a>
<figcaption>Escolha <strong>Armazenamento na nuvem</strong>…</figcaption>
</figure>

<figure class="shot">
<a href="/img/docs/add-storage-cloud.webp"><img src="/img/docs/add-storage-cloud.webp" width="560" height="620" loading="lazy" alt="Seletor de provedores com S3, R2, MinIO, B2, Wasabi, Spaces, Drive, Dropbox, OneDrive e outros"></a>
<figcaption>…e depois o provedor.</figcaption>
</figure>

</div>

Para o Google Drive, use **Configurações → Integrações**, como descrito acima: o login é feito pelo navegador, sem pedir tokens.

Pela CLI, a configuração interativa orienta você:

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
