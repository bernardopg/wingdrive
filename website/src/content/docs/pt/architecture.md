---
title: Arquitetura
description: Como o daemon, os clientes, a biblioteca e a rede se integram.
order: 11
group: Avançado
---

O WingDrive segue uma arquitetura cliente-daemon. Um único processo `wing-daemon` gerencia todo o estado. Cada cliente é uma interface leve que se comunica com ele por JSON-RPC 2.0.

```text
Desktop app (Tauri + React) ┐
wing CLI                    ├─ JSON-RPC ─▶ wing-daemon ─▶ actions / queries ─▶ services ─▶ SQLite
wing-server + web UI        ┘
```

O aplicativo móvel é a exceção: ele incorpora o mesmo núcleo Rust como biblioteca, em vez de se comunicar com um daemon.

## A pilha

| Camada | Tecnologia |
|---|---|
| Núcleo | Rust, Tokio |
| Banco de dados | SQLite com SeaORM |
| Rede | Iroh sobre QUIC |
| Identidade de conteúdo | BLAKE3 |
| Armazenamento na nuvem | OpenDAL |
| Desktop | Tauri 2 e React 19 |

## Dentro do núcleo

O núcleo separa **substantivos** de **verbos**:

- **Domínio**: arquivos, dispositivos, volumes, localizações, etiquetas e endereçamento.
- **Operações**: cada verbo disponível ao usuário é uma *ação* (escrita) ou uma *consulta* (leitura), registrada em tempo de compilação. A mesma operação atende ao aplicativo desktop, à CLI e à web.
- **Tarefas**: operações longas persistem seu progresso e continuam após uma reinicialização.
- **Serviços**: subsistemas de longa duração, como rede, sincronização, monitoramento do sistema de arquivos e monitoramento de volumes.

## Indexação

Adicionar uma localização inicia uma tarefa de indexação que percorre a pasta, registra as entradas na biblioteca e, dependendo do modo, calcula a identidade de conteúdo e dados de mídia, como miniaturas. Um monitor mantém o índice atualizado conforme os arquivos mudam.

A identidade de conteúdo usa BLAKE3. Arquivos pequenos são processados integralmente; arquivos com mais de 100 KB são amostrados (58 KB no total), o que mantém a indexação na nuvem rápida e previsível.

## Endereçamento

Cada arquivo tem um **WingPath**:

- **Físico**: um dispositivo e um caminho nele.
- **Nuvem**: um serviço, um bucket ou unidade e um caminho.
- **Conteúdo**: um ID de conteúdo que aponta para qualquer cópia disponível.

É isso que permite copiar e mover da mesma forma entre um notebook, uma unidade externa e um bucket.

## Privacidade desde a concepção

A indexação, as miniaturas e a pesquisa são executadas localmente. O tráfego entre dispositivos emparelhados é criptografado. As credenciais de nuvem são criptografadas em repouso no chaveiro do sistema operacional. Nenhum dado é enviado de volta ao projeto.

## Saiba mais

O repositório contém a documentação técnica completa: [docs/ no GitHub](https://github.com/bernardopg/wingdrive/tree/main/docs).
