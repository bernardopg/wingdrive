---
title: Primeira execução
description: Crie uma biblioteca, adicione suas primeiras localizações e conheça a interface.
order: 3
group: Início
---

O WingDrive é executado como um serviço em segundo plano (o **daemon**), com uma interface separada. Fechar a janela não interrompe a indexação nem a sincronização e, com *Keep running in the background* ativado, o WingDrive permanece na bandeja para que as pastas abram imediatamente.

## 1. Crie uma biblioteca

Na primeira inicialização, o assistente de configuração pede um nome para a biblioteca, como "Pessoal" ou "Trabalho". Você pode criar mais bibliotecas depois e alternar entre elas.

A biblioteca é um banco de dados local. Fazer backup de todo o seu sistema de organização é tão simples quanto copiar a pasta da biblioteca.

## 2. Adicione localizações

Uma localização é uma pasta monitorada pelo WingDrive. Clique em **+** ao lado de *Locations* na barra lateral ou acesse **Settings → Locations** e escolha uma pasta. Comece com algo fácil de gerenciar, como Documentos ou Imagens.

Modos de indexação:

- **Shallow**: apenas nomes e metadados básicos. É o mais rápido.
- **Content**: adiciona a identidade de conteúdo (BLAKE3), permitindo reconhecer duplicatas e cópias.
- **Deep**: adiciona análise de mídia, como miniaturas. Recomendado para pastas de fotos e vídeos.

Acompanhe o progresso no gerenciador de tarefas, acessível pelo indicador de status. Unidades removíveis conectadas aparecem automaticamente como volumes.

## 3. Conheça a interface

- **Barra lateral**: visão geral, recentes, favoritos, localizações, volumes, dispositivos e etiquetas. Personalize os grupos de acordo com sua forma de trabalhar.
- **Explorer**: a visualização de arquivos, com abas e modos de grade, lista, colunas, mídia e tamanho.
- **Inspector**: metadados, etiquetas e detalhes técnicos da seleção (Ctrl+I).
- **Barra de pesquisa**: encontra arquivos em todas as localizações indexadas.
- **Paleta de comandos**: todas as ações pelo nome (Ctrl+Shift+P).

## 4. Defina como padrão

No Linux, o WingDrive pode se tornar o gerenciador de arquivos padrão, para que `xdg-open` em uma pasta e "Show in folder" em outros aplicativos a abram nele. Consulte [Integração com Linux](/pt/docs/linux-integration/).

## A seguir

- [Usar o Explorer](/pt/docs/explorer/)
- [Operações com arquivos e como desfazê-las](/pt/docs/file-operations/)
