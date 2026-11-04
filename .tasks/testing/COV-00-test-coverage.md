---
id: COV-00
title: "Epic: Aumentar cobertura de testes do core, desktop e CLI"
status: In Progress
assignee: bernardopg
parent: DEV-000
priority: High
milestone: M3
tags: [epic, coverage, testing, core, desktop, cli]
last_updated: 2026-11-03
---

## Description

Aumentar a cobertura com testes de comportamento e de recuperação de falhas. O plano usa a
infraestrutura entregue na DEV-006 e prioriza perda de dados, contratos entre clientes e daemon
e regressões do uso diário no desktop. Entregas críticas entram no M2; integração completa e
gates finais fecham no M3. A divisão e os critérios de medição estão em [README.md](README.md).

## Implementation Steps

- [ ] [COV-01](COV-01-baseline-and-inventory.md): Medir baseline e inventariar lacunas por componente
- [ ] [COV-02](COV-02-contract-regression-tests.md): Adicionar a primeira suíte de regressões de contratos
- [ ] [COV-03](COV-03-core-file-operations.md): Cobrir operações de arquivos e prevenção de perda de dados
- [ ] [COV-04](COV-04-core-indexing-database.md): Cobrir banco, indexação e reconciliação do watcher
- [ ] [COV-05](COV-05-core-jobs-sync.md): Cobrir jobs, checkpoints e sincronização entre dispositivos
- [ ] [COV-06](COV-06-core-addressing-rpc.md): Cobrir endereçamento, validação e contratos RPC
- [ ] [COV-07](COV-07-desktop-native.md): Cobrir shell desktop, IPC e segurança do undo
- [ ] [COV-08](COV-08-desktop-react.md): Cobrir fluxos do Explorer e sidebar na interface React
- [ ] [COV-09](COV-09-cli-unit-contracts.md): Cobrir parsing, configuração e saída do CLI
- [ ] [COV-10](COV-10-cli-daemon-integration.md): Testar CLI com daemon isolado e filesystem real
- [ ] [COV-11](COV-11-coverage-ratchet.md): Aplicar metas progressivas e impedir regressão de cobertura
- [ ] [COV-12](COV-12-coverage-final-validation.md): Auditar metas, estabilidade e evidências do épico

## Acceptance Criteria

- [ ] Core com pelo menos 50% de linhas cobertas, desktop 75% e CLI 60% no relatório completo.
- [ ] Interface React compartilhada com pelo menos 60%, medida separadamente do shell desktop.
- [ ] Pelo menos 80% de cobertura nas linhas de produção alteradas, com erros e limites relevantes.
- [ ] Código não instrumentado listado; exclusões justificadas, sem remover arquivos para atingir metas.
- [ ] Todas as filhas Done com evidência, CI aprovado e merge no main conforme PLANNING.md.
- [ ] Limites de runtime e plataformas registrados; cobertura Linux não comprova macOS/Windows.
