---
title: Primeira execução
description: Crie uma biblioteca, adicione suas primeiras pastas e conheça a interface.
order: 3
group: Início
---

O WingDrive roda um serviço em segundo plano (o **daemon**) e uma janela separada. Fechar a janela não interrompe a indexação nem a sincronização. Com *Keep running in the background* ativado, o WingDrive fica na bandeja e as pastas abrem na hora.

Esta página leva uns cinco minutos. No fim você tem uma biblioteca com as primeiras pastas indexadas.

<figure class="shot">
<a href="/img/shots/WingOverview-1600.webp"><img src="/img/shots/WingOverview-1600.webp" srcset="/img/shots/WingOverview-800.webp 800w, /img/shots/WingOverview-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="A página Visão geral com totais de armazenamento, dispositivos, localizações e volumes"></a>
<figcaption>A Visão geral depois de indexar as primeiras pastas: armazenamento, arquivos indexados, dispositivos, localizações e volumes.</figcaption>
</figure>

## 1. Crie uma biblioteca

<ol class="steps">
<li>Abra o WingDrive. Na primeira vez, o assistente de configuração abre.</li>
<li>Dê um nome à biblioteca, por exemplo <em>Pessoal</em> ou <em>Trabalho</em>.</li>
<li>Confirme. A barra lateral aparece com <strong>Visão geral</strong>, <strong>Recentes</strong> e <strong>Favoritos</strong>.</li>
</ol>

A biblioteca é um banco de dados local sobre os seus arquivos: nomes, tamanhos, identidade de conteúdo, etiquetas e dispositivos. Os arquivos nunca são movidos para dentro dela. Para fazer backup de tudo que você organizou, copie a pasta da biblioteca. Dá para criar outras bibliotecas depois e alternar entre elas pelo menu ao lado do título da página.

## 2. Adicione suas pastas

Uma **localização** é uma pasta que o WingDrive observa e indexa. Adicionar uma nunca move nem copia arquivos.

<ol class="steps">
<li>Na Visão geral, clique em <strong>Adicionar armazenamento</strong>, no canto superior direito. Também dá para clicar no <strong>+</strong> ao lado de <em>Localizações</em> na barra lateral.</li>
<li>Escolha <strong>Pasta local</strong> e selecione uma pasta. Comece por algo pequeno, como Documentos ou Imagens.</li>
<li>Escolha o modo de indexação e confirme. A pasta aparece em <em>Localizações</em> e a indexação começa.</li>
</ol>

<figure class="shot">
<a href="/img/docs/add-storage.webp"><img src="/img/docs/add-storage.webp" width="560" height="500" loading="lazy" alt="Diálogo Adicionar armazenamento com Pasta local, Armazenamento na nuvem e Unidade externa"></a>
<figcaption><strong>Adicionar armazenamento</strong> inclui uma pasta local, um serviço na nuvem ou uma unidade externa.</figcaption>
</figure>

| Modo de indexação | O que lê | Use para |
|---|---|---|
| **Shallow** | Nomes e metadados básicos | Pastas enormes que você só navega |
| **Content** | Acrescenta a identidade de conteúdo BLAKE3 | A maioria das pastas: duplicatas e cópias são reconhecidas |
| **Deep** | Acrescenta análise de mídia e miniaturas | Pastas de fotos e vídeos |

Acompanhe o progresso no gerenciador de tarefas (o ícone de sincronização no rodapé da barra lateral). Drives removíveis aparecem sozinhos em *Volumes* quando você os conecta.

<div class="callout tip">

<p><strong>Dica.</strong> As regras de indexação ficam em <strong>Configurações → Indexador</strong>: pular arquivos de sistema, pastas Git, pastas de build como <code>node_modules</code> e arquivos ocultos, ou respeitar o <code>.gitignore</code>. Elas valem para as localizações adicionadas depois da mudança.</p>

</div>

<figure class="shot">
<a href="/img/docs/settings-indexer.webp"><img src="/img/docs/settings-indexer.webp" width="1400" height="640" loading="lazy" alt="Configurações do indexador com chaves de exclusão e filtros"></a>
<figcaption>Configurações → Indexador decide o que as novas localizações ignoram.</figcaption>
</figure>

## 3. Conheça a interface

<figure class="shot">
<a href="/img/shots/WingGridView-1600.webp"><img src="/img/shots/WingGridView-1600.webp" srcset="/img/shots/WingGridView-800.webp 800w, /img/shots/WingGridView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Visualização em grade com o Inspetor aberto em uma foto"></a>
<figcaption>O Explorer em grade. A barra lateral fica à esquerda e o Inspetor à direita.</figcaption>
</figure>

- **Barra lateral**: Visão geral, Recentes, Favoritos, Tipos de arquivo, localizações, volumes, dispositivos e etiquetas. Arraste os itens para reordenar. O botão de paleta no rodapé adiciona grupos.
- **Explorer**: a visualização de arquivos, com abas e as visualizações em grade, lista, colunas, mídia e tamanho.
- **Inspetor** (<kbd>Ctrl</kbd>+<kbd>I</kbd>): detalhes, datas, caminho, permissões, SHA-256 e etiquetas da seleção.
- **Pesquisa** (a lupa na barra superior): encontra arquivos em todas as localizações indexadas.
- **Paleta de comandos** (<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd>): todas as ações pelo nome.
- **Configurações** (<kbd>Ctrl</kbd>+<kbd>,</kbd> ou a engrenagem no rodapé da barra lateral).

## 4. Defina como padrão

No Linux, o WingDrive pode abrir pastas pelo `xdg-open`, atender o "Mostrar na pasta" de outros apps e escolher arquivos para uploads no navegador. Veja [Integração com o Linux](/pt/docs/linux-integration/).

## A seguir

- [Use o Explorer](/pt/docs/explorer/)
- [Operações de arquivo e desfazer](/pt/docs/file-operations/)
- [Conecte o Google Drive e outras nuvens](/pt/docs/cloud/)
