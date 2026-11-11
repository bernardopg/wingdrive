---
title: O que é o WingDrive?
description: Um gerenciador de arquivos local-first que indexa todas as suas unidades em uma biblioteca.
order: 1
group: Início
---

O WingDrive é um gerenciador de arquivos para desktop com um daemon em segundo plano que indexa suas pastas, unidades externas e volumes na nuvem em uma única **biblioteca**. Os arquivos permanecem onde estão. O índice permite navegar, pesquisar e reconhecer arquivos rapidamente entre seus dispositivos.

Não há conta, telemetria nem servidor central. A indexação, as miniaturas e a pesquisa são executadas na sua máquina.

> **Alpha.** Linux é a plataforma compatível, e o objetivo atual (marco M2) é substituir Dolphin, Nautilus ou Yazi como seu gerenciador de arquivos padrão. Os instaladores para Windows são distribuídos em versão beta desde a 3.0. No macOS, a compilação é feita a partir do código-fonte.

## O que você encontra

- **Um Explorer rápido** com visualizações em grade, lista, colunas, mídia e tamanho, Quick Preview, modo de painel duplo (F3) e terminal acoplado (F4).
- **Pesquisa** com filtros por tipo, extensão, tamanho, data e arquivos ocultos, com contagens para todos os resultados.
- **Operações seguras com arquivos**: verificação prévia de cópia, cópias atômicas, desfazer cópia, movimentação e envio à lixeira, compactação e extração, somas de verificação.
- **Identidade de conteúdo**: cada arquivo recebe um hash BLAKE3, para que o mesmo arquivo em dois dispositivos seja reconhecido como um só.
- **Dispositivos**: emparelhe suas máquinas via Iroh/QUIC para sincronizar a biblioteca e enviar arquivos com o Wingdrop.
- **Volumes na nuvem** por meio do OpenDAL: S3, Google Drive, Dropbox, OneDrive, Azure Blob, GCS e outros.
- **CLI `wing` e `wing-server`** para scripts e máquinas sem interface gráfica, no mesmo daemon.

## Conceitos principais

**Biblioteca.** Seu banco de dados pessoal de informações sobre arquivos: metadados, etiquetas, dispositivos e configurações de sincronização. Fica na sua máquina e pode sincronizar os dados de organização com dispositivos emparelhados.

**Localização.** Uma pasta que o WingDrive monitora e indexa. Adicionar uma localização nunca move nem copia arquivos.

**Volume.** Um dispositivo de armazenamento: um disco interno, uma unidade removível ou um serviço de nuvem montado como volume.

**Sistema de arquivos virtual (VDFS).** Uma visualização única dos arquivos em todos os dispositivos e na nuvem. Cada arquivo mantém sua localização física, mas aparece em uma única interface pesquisável.

**WingPath.** Um endereço universal de arquivo. Um caminho pode ser *físico* (um arquivo em um dispositivo), *na nuvem* (`s3://bucket/photo.jpg`) ou *por conteúdo* (encontra o arquivo pelo seu conteúdo, onde quer que haja uma cópia).

## Próximos passos

- [Instalar o WingDrive](/pt/docs/installation/)
- [Primeira execução](/pt/docs/first-run/)
- [Usar o Explorer](/pt/docs/explorer/)
