# Revisão do app Tauri do WingDrive como file manager

Data: 2026-09-03
Escopo: `apps/tauri`, `packages/interface/src/routes/explorer`, `core/src/ops/files`
Ambiente: Linux, `GDK_BACKEND=x11`, daemon `sd-daemon` em `127.0.0.1:6969`, biblioteca `My Library` com 0 locations e 2 volumes.

## Resumo

O app subiu e conectou no daemon, mas só depois de corrigir um bug bloqueante de configuração. A navegação, seleção, teclado e views estão bem construídas e virtualizadas. Os problemas reais estão na camada de *feedback de estado*: operações de arquivo não atualizam a listagem, e o app assina eventos do filesystem que ninguém consome. Para uso como file manager no dia a dia, isso é o que mais dói.

---

## P0 — Bloqueantes

### 1. O app não iniciava: `plugins.updater` ausente

`apps/tauri/src-tauri/src/main.rs:1972` registra `tauri_plugin_updater`, mas `tauri.conf.json` não tinha bloco `plugins`. Resultado:

```
thread 'main' panicked at apps/tauri/src-tauri/src/main.rs:2297:10:
error while running tauri application: PluginInitialization("updater",
  "Error deserializing 'plugins.updater' ...: invalid type: null, expected struct Config")
```

Corrigido nesta sessão adicionando um bloco mínimo em `apps/tauri/src-tauri/tauri.conf.json`:

```json
"plugins": { "updater": { "endpoints": [], "pubkey": "" } }
```

Ação pendente: definir endpoints e `pubkey` reais para release, ou registrar o plugin condicionalmente (`#[cfg(desktop)]` + feature flag) para que a ausência de config não derrube o app.

### 2. Copiar / mover / colar não atualiza a listagem

`packages/interface/src/hooks/useRefetchFileListings.ts` existe e funciona, mas só é usado em dois lugares:

- `routes/explorer/hooks/useDeleteFiles.ts`
- `routes/explorer/hooks/useDuplicateFiles.ts`

`components/modals/FileOperationModal.tsx:handleSubmit` faz `await copyFiles.mutateAsync(...)` e fecha o diálogo imediatamente. A mutation só **enfileira um job**; não há `useWaitForJob` nem `refetchListings`. Consequência: depois de colar, arrastar para uma pasta ou importar do sistema, o arquivo não aparece.

`useDeleteFiles` já resolveu esse problema corretamente (espera o job, checa `failed_count`, refaz o fetch). O caminho de cópia precisa do mesmo tratamento.

Mesma lacuna em:
- `routes/explorer/hooks/useEmptySpaceContextMenu.ts` — "New Folder" não refaz o fetch.
- `routes/explorer/SelectionContext.tsx:saveRename` — rename não refaz o fetch nem mostra toast de erro (só `console.error` + `throw`).

### 3. Eventos de filesystem são assinados mas ninguém escuta

`packages/ts-client/src/event-filter.ts` inclui na assinatura padrão: `EntryCreated`, `EntryModified`, `EntryDeleted`, `EntryMoved`, `FsRawChange`, `FileOperationCompleted`, `FilesModified`, `IndexingCompleted`.

Nenhum arquivo em `packages/interface/src` ou `packages/ts-client/src/hooks` referencia esses eventos. Verificado por grep: zero ocorrências fora do próprio filtro.

O explorer nunca reage a mudanças externas. Crie um arquivo pelo terminal com o app aberto e ele não aparece. Some com um arquivo e a linha continua lá.

### 4. Pasta aparece vazia na primeira visita

`core/src/ops/files/query/directory_listing.rs:750-760`, caminho efêmero (pasta não indexada):

```rust
if cache.is_indexing(&local_path) {
    return Ok(DirectoryListingOutput { files: Vec::new(), total_count: 0, has_more: false });
}
```

A query dispara um indexer em background e retorna vazio. Como o frontend não escuta `IndexingCompleted` (item 3) e não tem `refetchInterval`, a pasta fica vazia até o usuário sair e voltar. É a primeira coisa que um usuário novo vê ao navegar fora de uma location.

---

## P1 — Semântica incorreta de file manager

### 5. Arrastar arquivo para pasta copia em vez de mover

`packages/interface/src/components/DndProvider.tsx:344`

```rust
// Determine operation based on modifier keys
// For now default to copy (user can choose in modal)
const operation = "copy";
```

O comentário diz que lê teclas modificadoras; o código não lê. Todo file manager desktop **move** dentro do mesmo volume e **copia** entre volumes. Copiar por padrão dentro do mesmo disco é o comportamento errado e gera duplicatas silenciosas.

Correção: comparar o volume de origem/destino, usar `move` no mesmo volume, e ler `shiftKey` (forçar move) / `altKey` ou `ctrlKey` (forçar copy) do evento do dnd-kit.

### 6. Soltar em pasta só funciona no Grid view

`useDroppable` aparece apenas em `routes/explorer/views/GridView/FileCard.tsx:111`. `ListView`, `ColumnView` e `MediaView` não registram drop targets, e o `Breadcrumb` também não (sem "mover para a pasta pai"). Trocar de view muda o que o app consegue fazer.

### 7. Não existe ordenação ascendente/descendente

`core/src/ops/files/query/directory_listing.rs:44` define `DirectorySortBy { Name, Modified, Size, Type }` — sem campo de direção. A ordem é fixa por variante ("newest first", "largest first"). Clicar duas vezes no cabeçalho da coluna não inverte nada porque o backend não expõe isso.

Correção: adicionar `sort_direction: SortDirection` ao input e propagar para o `ORDER BY`.

### 8. Clipboard não conversa com o sistema

`packages/interface/src/hooks/useClipboard.ts` é um store zustand isolado. Copiar no WingDrive e colar no Nautilus/Finder não funciona, e vice-versa. `tauri_plugin_clipboard_manager` já está registrado em `main.rs:1967` mas não é usado para paths de arquivo.

### 9. Atalhos padrão ausentes

`packages/interface/src/util/keybinds/registry.ts` não tem:
- `explorer.refresh` (F5 / Cmd+R)
- `explorer.newFolder` (Shift+Cmd+N)
- `explorer.toggleHiddenFiles` (Cmd+Shift+.)

Sem refresh manual e sem atualização por evento (item 3), não há como forçar a listagem a se atualizar.

---

## P2 — Bugs de implementação

### 10. Hooks dentro de `.map()` no FileOperationModal

`packages/interface/src/components/modals/FileOperationModal.tsx:57-64`

```tsx
const sourcePaths = props.sources.slice(0, 3).map(...).filter(...);
const sourceFileQueries = sourcePaths.map(path =>
    useLibraryQuery({ type: "files.by_path", input: { path } }, { enabled: !!path })
);
```

Chamada de hook dentro de `map` com contagem variável. Viola as Rules of Hooks. Se o número de sources mudar entre renders (ex.: seleção de 1 para 3 arquivos com o diálogo montado), React quebra ou reordena estado entre queries. Deve virar uma query em lote única.

### 11. `hasSameSourceDest` compara peras com maçãs

`FileOperationModal.tsx:82-88` compara `source.Physical.path === destination.Physical.path`. Source é o **arquivo**, destination é a **pasta**. A comparação nunca é verdadeira no caso real (colar na própria pasta de origem). A proteção útil — colar dentro da pasta de onde veio, ou mover pasta para dentro de si mesma — não acontece aqui.

Nota positiva: o backend cobre o caso perigoso. `core/src/ops/files/copy/safety.rs:recursive_copy_errors` rejeita destino dentro da origem, inclusive através de symlink, com testes. O problema é só de UX: o usuário abre o diálogo, confirma, e o job falha depois.

### 12. `reveal_file` no Windows está quebrado

`apps/tauri/src-tauri/src/files.rs:137`

```rust
std::process::Command::new("explorer").arg("/select,").arg(path)
```

`explorer.exe` exige `/select,<path>` como **um único argumento**. Assim, ele ignora e abre Documentos.

No Linux (`files.rs:147`) o reveal só faz `xdg-open` da pasta pai, sem selecionar o arquivo. Aceitável como fallback, mas vale usar a interface D-Bus `org.freedesktop.FileManager1.ShowItems` quando disponível.

Em ambos: `.spawn()?.wait()?` síncrono dentro de `async fn` bloqueia a thread do runtime. Usar `tokio::process::Command` ou `spawn_blocking`.

### 13. Duplo clique não trata Symlink

`routes/explorer/views/GridView/FileCard.tsx:81`

```tsx
if (file.kind === "File" && "Physical" in file.sd_path) { ... }
```

`EntryKind` (`core/src/domain/file.rs:22`) tem `File | Directory | Symlink`. Duplo clique em symlink não faz nada, sem feedback.

### 14. `alert()` em vez de toast

`useEmptySpaceContextMenu.ts:31` usa `alert()`. O próprio `platform.ts:56` documenta que diálogos nativos do WebView são inconfiáveis no Windows. O resto do app usa `toast` de `@wingdrive/primitives`.

### 15. Churn de assinaturas de eventos

No log de runtime (`/tmp/wd-tauri2.log`), a mesma assinatura de eventos de job é recriada repetidamente durante navegação normal:

```
subscription_id=26, 27, 28, 32, 33, 41, 42, 43, 46, 47 ...
```

15 assinaturas idênticas de `["JobQueued","JobStarted","JobProgress",...]` em poucos minutos. Indica dependências instáveis de `useEffect` causando subscribe/unsubscribe em loop. Cada uma abre uma conexão TCP com o daemon.

### 16. `console.log` de produção no caminho do clipboard

`useClipboard.ts`, `useExplorerKeyboard.ts:112-130`, `useEmptySpaceContextMenu.ts:44-56` e `DndProvider.tsx` fazem `console.groupCollapsed` com dump JSON completo dos `SdPath` a cada copy/cut/paste. Ruído e custo de serialização em operação quente.

---

## O que está bom

- Views virtualizadas com `@tanstack/react-virtual` em Grid, List, Column e Media. Diretórios grandes não travam a UI.
- `directory_listing` com `limit: null` retorna tudo sem truncar silenciosamente (verificado em `directory_listing.rs:240`).
- Proteção contra cópia recursiva no core, com cobertura de symlink e testes (`copy/safety.rs`).
- Delete com trash real por padrão (`delete/job.rs:212`), diálogo de confirmação estilizado, espera de job e relatório de `failed_count`.
- Histórico de navegação por aba com back/forward corretos (`explorer/context.tsx:104-161`).
- Import externo com rejeição explícita e toast em vez de engolir o drop (`useExternalFileDrop.ts` + `resolveExternalDrop.ts`, com teste).
- Seleção sincronizada com TabManager via ref, evitando updater durante render no React 19 (`SelectionContext.tsx:88-107`).

---

## Ordem sugerida

1. `plugins.updater` definitivo (P0-1) — já mitigado, falta decidir a forma final.
2. Consumir `FsRawChange` / `EntryCreated` / `EntryDeleted` / `EntryMoved` / `IndexingCompleted` num hook único que invalida as query keys de listagem. Resolve os itens 2, 3 e 4 de uma vez e torna o `useRefetchFileListings` manual desnecessário.
3. `move` como padrão no drag interno, com detecção de volume e teclas modificadoras.
4. Drop targets em List, Column, Media e Breadcrumb.
5. Direção de ordenação no `DirectoryListingInput`.
6. Corrigir os hooks em `.map()` do FileOperationModal.
7. `reveal_file` no Windows e I/O não bloqueante.
8. Atalhos de refresh, nova pasta e arquivos ocultos.
