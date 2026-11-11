# Plano de cobertura

Épico [COV-00](COV-00-test-coverage.md), ligado à DEV-000. Segue
[PLANNING.md](../PLANNING.md): roadmap, milestone, épico, task e sprint de duas semanas.
A DEV-006 já entrega runners, LCOV, badges e publicação; este plano amplia os testes.

## Metas de linhas

| Componente do relatório | Referência no main | Primeira etapa | Conclusão |
|---|---|---|---|
| Core (`core/`) | 19,6% (15.875/81.068) | 30% | 50% |
| Desktop (`apps/tauri/`) | 45,2% (2.702/5.981) | 60% | 75% |
| CLI (`apps/cli/`) | 0,4% (20/5.128) | 25% | 60% |
| Interface (`packages/interface/`) | 39,6% (2.189/5.527) | 50% | 60% |

Referência verificada em 2026-11-03, gerada às 17:58 UTC no SHA
`1064b6b8b6e94f5b7040e3c112e784d5c0e57271`. Fonte: `summary.json` do branch
[badges](https://github.com/bernardopg/wingdrive/blob/badges/summary.json).
São metas, não resultados já obtidos neste branch.

As porcentagens contam linhas instrumentadas, conforme [COVERAGE.md](../../COVERAGE.md).
Código Rust não compilado e TypeScript não importado podem ficar fora do denominador.
A COV-01 deve listar essas lacunas e manter o escopo comparável antes/depois. Não reduzir
exclusões ou escolher só arquivos fáceis para elevar a média. Linhas novas/alteradas de produção
terão piso de 80% na COV-11; branches de falha relevantes exigem assertivas mesmo sem métrica
de branch disponível. O número de testes não substitui medição de cobertura.

## Ordem e capacidade

1. COV-01 estabelece a referência e o inventário; COV-02 entrega a primeira suíte de contratos.
2. COV-03, COV-06, COV-07 e COV-09 protegem arquivos, endereçamento, shell e CLI no M2.
3. COV-04 e COV-08 ampliam indexação e fluxos React no M2.
4. COV-05 e COV-10 cobrem recuperação/sync e CLI com daemon no M3.
5. COV-11 ativa os pisos após as suites passarem; COV-12 verifica o conjunto e fecha o épico.

COV-01 a COV-04, COV-06, COV-07 e COV-09 estão In Progress (#343, #344, #358, #359 e #362; a
COV-07 já mede 81,5%); bugs achados pelas suítes ficam em [BUGS.md](BUGS.md). As demais ficam no backlog sem `sprint`.
Não acrescentar compromisso ao S04/S05 já ocupado: puxar no máximo duas tasks por vez para o
sprint com capacidade, preencher `sprint` e atualizar o quadro e o log juntos. Não inventar
S06 nem datas antes do planning. Um teste implementado neste branch só vira Done após cumprir
critérios e a Definition of Done, incluindo merge no main.

## Evidência por entrega

Registrar SHA, ambiente/plataforma, comandos, contagens cobertas/instrumentadas por arquivo e
componente, LCOV/summary e duração. Todos os testes novos entram no runner normal e no de
cobertura. `just ci-local` deve passar antes de qualquer push.

Usar diretórios temporários, SQLite local descartável e processos/endpoints exclusivos.
Não alterar HOME/XDG do processo principal nem o daemon do autor. Serviços reais só entram
em integração isolada; nenhuma credencial externa é necessária. React roda com mocks isolados
por processo, como o CI atual. Testes nativos e validação de UI são evidências distintas.

```bash
scripts/coverage-rust.sh coverage/rust/rust.lcov
scripts/coverage-frontend.sh coverage/frontend
python3 scripts/coverage-report.py coverage/site coverage/rust/rust.lcov $(find coverage/frontend -name lcov.info)
```

O relatório completo é a referência das metas. Relatórios focados servem para diagnosticar
arquivos; não comprovam os pisos de componente.

Para o CLI, `scripts/coverage-cli-production.py` lê um LCOV e conta só as linhas de produção de
`apps/cli/`, sem os módulos `#[cfg(test)]` e as matrizes `args/matrix.rs` que o relatório completo
conta como código. É a medida usada nas metas da COV-09:

```bash
cargo llvm-cov --locked -p wing-cli --bin wing --lcov --output-path target/cov-cli/cli.lcov
python3 scripts/coverage-cli-production.py target/cov-cli/cli.lcov
``` Falha de um runner deixa aquele componente
sem evidência atual, mesmo que um badge anterior ainda exista.
