---
title: O Explorer
description: Visualizações, pesquisa, Quick Preview, abas, painel duplo, terminal e atalhos de teclado.
order: 4
group: Usando o WingDrive
---

O Explorer é onde você passa o dia. Ele foi criado para ser rápido com o teclado e agradável de ler.

## Visualizações

| Visualização | Ideal para |
|---|---|
| **Grade** | Pastas mistas, com miniaturas e o Inspector ao lado |
| **Lista** | Código e documentos: colunas ordenáveis, pastas expansíveis e indicadores Git |
| **Colunas** | Navegar por árvores de pastas profundas |
| **Mídia** | Pastas de fotos e vídeos |
| **Tamanho** | Encontrar o que ocupa espaço |

<div class="shots">

<figure class="shot">
<a href="/img/shots/WingListView-1600.webp"><img src="/img/shots/WingListView-1600.webp" srcset="/img/shots/WingListView-800.webp 800w, /img/shots/WingListView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Visualização em lista com indicadores Git e o terminal acoplado"></a>
<figcaption>Lista: colunas ordenáveis, pastas expansíveis, indicadores Git e o terminal (F4).</figcaption>
</figure>

<figure class="shot">
<a href="/img/shots/WingColumnView-1600.webp"><img src="/img/shots/WingColumnView-1600.webp" srcset="/img/shots/WingColumnView-800.webp 800w, /img/shots/WingColumnView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Visualização em colunas"></a>
<figcaption>Colunas: navegue por árvores profundas.</figcaption>
</figure>

</div>

<div class="shots">

<figure class="shot">
<a href="/img/shots/WingMediaView-1600.webp"><img src="/img/shots/WingMediaView-1600.webp" srcset="/img/shots/WingMediaView-800.webp 800w, /img/shots/WingMediaView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Visualização de mídia de uma pasta de fotos"></a>
<figcaption>Mídia: pastas de fotos e vídeos.</figcaption>
</figure>

<figure class="shot">
<a href="/img/shots/WingSizeView-1600.webp"><img src="/img/shots/WingSizeView-1600.webp" srcset="/img/shots/WingSizeView-800.webp 800w, /img/shots/WingSizeView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Visualização por tamanho"></a>
<figcaption>Tamanho: veja o que ocupa espaço.</figcaption>
</figure>

</div>

Agrupe uma pasta em seções por tipo ou data, mostre ou oculte arquivos ocultos (Ctrl+H) e filtre a pasta atual (Ctrl+Shift+F).

## Quick Preview

Pressione **Space** em um arquivo para visualizá-lo sem abrir outro aplicativo: imagens, vídeo, áudio, PDF, texto e código.

## Pesquisa

A barra de pesquisa busca em todas as localizações indexadas. Refine os resultados por **tipo**, **extensão**, **tamanho**, **data** e **arquivos ocultos**. As contagens abrangem todos os resultados, não apenas a primeira página.

<figure class="shot">
<a href="/img/shots/WingSearch-1600.webp"><img src="/img/shots/WingSearch-1600.webp" srcset="/img/shots/WingSearch-800.webp 800w, /img/shots/WingSearch-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Resultados de pesquisa com filtros de tipo, extensão, tamanho e data"></a>
<figcaption>Pesquisa com filtros. As contagens cobrem todos os resultados.</figcaption>
</figure>

## Abas, painel duplo e terminal

- **Abas**: Ctrl+T abre uma, Ctrl+W fecha e Ctrl+Shift+T reabre a última.
- **Painel duplo**: F3 divide o Explorer em dois para copiar e comparar lado a lado.
- **Terminal acoplado**: F4 mostra um terminal abaixo da visualização que acompanha a pasta atual. Shift+F4 abre seu próprio terminal nessa pasta. Escolha o comando em **Settings**; se estiver vazio, usa `xdg-terminal-exec`, `$TERMINAL` ou um terminal instalado.

## Git

Em um repositório, arquivos e pastas exibem indicadores dos estados modificado, adicionado, não rastreado e ignorado, de modo que a visualização em lista também funciona como um `git status` rápido.

## Etiquetas

As etiquetas organizam sem mover arquivos. Pressione Alt+G para entrar no modo de etiquetas; depois, 1–9 e 0 ativam ou desativam suas dez primeiras etiquetas na seleção. Escape sai do modo de etiquetas.

## Atalhos de teclado

No macOS, leia Ctrl como Cmd.

| Ação | Atalho |
|---|---|
| Paleta de comandos | Ctrl+Shift+P |
| Configurações | Ctrl+, |
| Quick Preview | Space |
| Renomear | F2 |
| Abrir seleção | Enter |
| Inspector | Ctrl+I |
| Focar na barra de caminho | Ctrl+L |
| Voltar / avançar / pasta superior | Ctrl+← / Ctrl+→ / Ctrl+↑ |
| Copiar / recortar / colar | Ctrl+C / Ctrl+X / Ctrl+V |
| Duplicar | Ctrl+D |
| Desfazer | Ctrl+Z |
| Mover para a lixeira / excluir permanentemente | Delete / Shift+Delete |
| Nova pasta / novo arquivo | Ctrl+Shift+N / Ctrl+Alt+N |
| Selecionar tudo / inverter / por padrão | Ctrl+A / Ctrl+Shift+I / Ctrl+S |
| Atualizar | F5 |
| Arquivos ocultos | Ctrl+H |
| Painel duplo | F3 |
| Terminal | F4 |
| Nova janela | Ctrl+N |
