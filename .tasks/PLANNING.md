# Planejamento do WingDrive

Fonte única de verdade para o que fazer, em que ordem e com qual progresso.
Se algo não está aqui ou numa task de `.tasks/`, não está planejado.

## Método

**Scrumban solo**: cadência de Scrum (sprints curtos com meta, review e retro) com fluxo de
Kanban (limite de WIP, puxar a próxima task do topo do backlog). É o modelo mais usado para
times pequenos e devs solo porque mantém previsibilidade sem cerimônia de time grande.

- Sprint de **2 semanas**, com uma **meta** de uma frase.
- **WIP máximo: 2 tasks** `In Progress` dentro do sprint. Termina antes de começar.
- Roadmap no formato **Now / Next / Later** por milestone, sem datas fixas além do sprint atual.
- Priorização do backlog: valor para o uso como file manager desktop primeiro; risco de perda
  de dados acima de tudo.

## Hierarquia

```
Roadmap (ROADMAP.md)
└── Milestone  M1, M2, ...        resultado entregável (ex.: "File manager desktop Linux 1.0")
    └── Epic   TAURI-000, EXPL-000 task com tag `epic`; agrupa por área
        └── Task  TAURI-006, CLI-001 unidade de trabalho com critérios de aceite
Sprint (sprints/SNN.md)           fatia de tempo; puxa tasks de um milestone
Backlog                           tasks `To Do` de um milestone sem `sprint`
```

Cada task registra a hierarquia no front matter:

```yaml
parent: TAURI-000   # epic
milestone: M1       # milestone do roadmap
sprint: S01         # só quando comprometida num sprint
```

## Convenção de nomes

Nada no projeto se chama "Spacedrive" nem usa prefixo `sd`/`sd-`. Use `wingdrive` ou `wing`
(crates, binários, pacotes, scripts, funções, tipos, variáveis de ambiente, textos). Exceção:
atribuição legal ao upstream. Ver `FORK-003`.

## Definition of Ready (entra no sprint)

- Critérios de aceite verificáveis escritos na task.
- `milestone` definido e dependências resolvidas.

## Definition of Done (sai do sprint)

- Todos os critérios de aceite marcados `[x]` com evidência (teste, comando ou validação em runtime).
- `cargo fmt`, `cargo clippy -D warnings` e `bun run typecheck` limpos para o que mudou.
- Task com `status: Done` e `last_updated` atualizado; sprint file atualizado.
- Commit no `main` com mensagem Conventional Commits.

## Rituais

| Quando | O quê | Onde registrar |
|---|---|---|
| Início do sprint | Planning: meta + tasks puxadas do backlog | `sprints/SNN.md`, campo `sprint` nas tasks |
| A cada sessão | Atualizar checkboxes e status das tasks tocadas | task file + log do sprint |
| Fim do sprint | Review (o que entregou) + Retro (o que muda) | seção final de `sprints/SNN.md` |
| Fim do sprint | Tasks não concluídas voltam ao backlog ou vão ao próximo sprint | campo `sprint` |
| Fim do milestone | Atualizar `ROADMAP.md` (Now/Next/Later) | `ROADMAP.md` |

## Comandos

```bash
cargo run --bin task-validator -- list --milestone M1            # escopo do milestone
cargo run --bin task-validator -- list --sprint S01              # quadro do sprint
cargo run --bin task-validator -- list --milestone M1 --status "To Do"   # backlog do milestone
cargo run --bin task-validator -- validate                       # schema das tasks staged
```

Arquivos de planejamento (`PLANNING.md`, `ROADMAP.md`, `sprints/SNN.md`) não têm hífen no
nome, então o validator os ignora como tasks.
