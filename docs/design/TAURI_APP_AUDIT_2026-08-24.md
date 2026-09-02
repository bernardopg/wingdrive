# Auditoria do app Tauri

Data: 24 de agosto de 2026

## Resultado executivo

O app Tauri é um produto funcional em desenvolvimento, não um mockup completo. O shell nativo, a conexão com o daemon, o Explorer, o Overview, as estatísticas por tipo, Sources, Redundancy, Settings, Inspector, jobs, tabs e várias janelas auxiliares usam código e operações reais.

O app ainda não pode ser classificado como perfeitamente implementado. A auditoria encontrou rotas placeholder, ações sem persistência, integrações opcionais que simulam sucesso, diferenças entre consultas do backend e ausência de uma matriz de testes do aplicativo empacotado. A validação ao vivo cobriu Linux em modo de desenvolvimento. Ela não prova o mesmo comportamento no macOS, Windows ou bundle de produção.

## Escopo e evidência

Foram verificados:

- `apps/tauri`, incluindo rotas React, plataforma Tauri, comandos Rust, janelas e servidor HTTP local.
- `packages/interface`, incluindo Explorer, Overview, Inspector, Quick Preview, Sources, Redundancy, Settings, Spacebot e jobs.
- Tipos gerados em `packages/ts-client`.
- Backlog oficial em `.tasks` e o índice `TODO`.
- Aplicativo nativo Linux conectado ao daemon e a uma biblioteca real.

Checks executados:

- `bun run typecheck`: passou.
- Build Vite de produção do frontend Tauri: passou.
- `cargo check --manifest-path apps/tauri/src-tauri/Cargo.toml`: passou.
- `cargo run -p task-validator -- validate`: passou.
- Aplicativo `target/debug/WingDrive`: iniciou, abriu janela nativa e exibiu dados do daemon.

Evidência observada na janela nativa:

- Dispositivo `PC-689c341c` e volumes `Root` e `ssd-extra`.
- Overview com dados reais de volumes e capacidade.
- File Kinds inicialmente mostrou duas imagens e um vídeo, mas a análise do banco confirmou que eram `content_identities` órfãs, sem arquivos em `entries`.
- Sources com estado vazio e catálogo real de adapters.
- Redundancy com resposta real do backend e estado vazio coerente.
- Settings com nome, slug e diretório de dados reais.
- Explorer com arquivos reais, Inspector, tabs e comandos nativos de menu.

## Matriz de implementação

| Área | Estado | Evidência | Lacuna principal |
| --- | --- | --- | --- |
| Shell Tauri e daemon | Implementado e validado no Linux dev | Janela iniciou e exibiu dados via RPC | Falta bundle e matriz multiplataforma |
| Estado de biblioteca entre janelas | Implementado | Estado persistido e eventos `library-changed` | Cobertura automatizada ausente |
| Overview | Implementado e validado no runtime Linux | Queries reais de biblioteca, devices, volumes, locations e jobs renderizaram com o daemon ativo | Jobs remotos não são reativos |
| Explorer básico | Implementado | Diretório real, seleção, inspector e menus renderizados | Teste destrutivo de operações não executado |
| Grid | Implementado | Virtualização, thumbnails, seleção, teclado e drag interno no código | Meta de 10 mil itens não medida |
| List | Implementado | TanStack Table, virtualização, sort e seleção no código | Multi-sort e resize precisam de verificação |
| Column e Media | Implementados, parcialmente validados | Componentes usam listagens e seleção reais | Scroll por tab e matriz de codecs ausentes |
| Search básico | Implementado | `search.files`, debounce e views compartilhadas | Filtros avançados, histórico, salvos e semântico faltam |
| Recents | Implementado e validado no runtime Linux, UX parcial | Query real ordenada por `IndexedAt` e rota abriram com o daemon ativo | Estado vazio pouco informativo |
| File Kinds | Implementado e validado no runtime Linux sem arquivos | Estatísticas e busca usam o `kind_id` canônico de arquivos navegáveis; a rota exibiu contagens zero coerentes | Falta validar a abertura de um cartão não vazio no Explorer filtrado |
| Favorites | Implementado e validado no runtime Linux | `metadata.set_favorite` persiste e sincroniza metadata; Inspector e Quick Preview usam a action; rota filtra `favorite=true` e reage por invalidação; dois clientes receberam o ResourceEvent e o favorito sobreviveu ao restart do daemon | Sync com um segundo peer ainda não foi exercitado |
| Sources | Implementado e validado no runtime Linux sem registros | List, adapters, config, create, sync e delete usam RPC real; a rota exibiu empty state e ação Add Source | Nenhuma source configurada para validar create, sync e delete |
| Busca de Sources | Corrigido | Agora filtra nome, adapter e tipo carregados | Busca é local, adequada ao conjunto já carregado |
| Redundancy | Implementado e validado no runtime Linux sem conteúdo replicado | `redundancy.summary`, filtros estruturados e estado sem volumes indexados renderizaram | Dados reais replicados não estavam disponíveis |
| Tags | Implementado para entradas indexadas | Queries e mutations reais | Entradas efêmeras continuam sem suporte completo |
| Inspector | Implementado, com lacunas | Metadados, sidecars, tags, Favorite persistente e jobs reais | History possui dados dummy |
| Quick Preview | Implementado, validação parcial | Controller, renderers e integração do Inspector existem | Vídeo e áudio Linux têm mudanças locais ainda não validadas em bundle |
| File opening | Implementado | Backend por plataforma e UI `Open With` existem | Matriz por extensão e sistema pendente |
| File operations | Implementado, não validado destrutivamente | Copy, move, delete, rename, duplicate e create folder usam actions/jobs | Undo e validação antecipada de conflito faltam |
| Job Manager | Implementado e validado no runtime Linux | Popover, tela completa e histórico real de nove jobs renderizaram | Ordenação, detalhes e pop-out completo pendentes |
| Settings | Parcialmente implementado, janela validada no runtime Linux | A janela nativa separada abriu e General exibiu device, slug e versão reais | Updater, sync, encryption e persistência total não provados |
| Multi-window | Parcialmente implementado | Settings abriu como segunda janela nativa; Inspector, Quick Preview, jobs, Spacedrop e overlays existem | Explorer independente, geometria persistente, drag entre janelas e demais janelas não foram exercitados |
| Spacedrop | Implementado no código, não exercitado | Janela e protocolo estão conectados ao backend | Teste entre dois peers não executado |
| Updater | Exemplo ou integração parcial | Documentação e exemplo existem | Fluxo de release real não foi validado |
| Spacebot | Indisponibilidade explícita sem dependência privada | A rota informa que o Spacebot está indisponível e todas as mutations do stub rejeitam | Contrato com o runtime privado não foi executado porque o pacote não existe neste checkout |
| Memories, Autonomy e Schedule | Não expostos sem runtime real | Rotas placeholder foram removidas | Implementação depende do runtime privado |
| Importação externa por drop | Não implementada | Handler anterior descartava os arquivos | Precisa de destino, conflito, job e progresso |

## Bugs e riscos priorizados

### Alta prioridade

1. Spacebot não simula mais sucesso sem runtime real; falta validar o contrato privado.

   A indisponibilidade é explícita e as sete mutations do stub rejeitam. As rotas placeholder de Memories, Autonomy e Schedule não são mais expostas. O teste de contrato real continua bloqueado porque `/home/bitter/development/spacebot/packages/api-client` não existe neste sistema. Tarefa em progresso: `TAURI-004`.

2. Favorites persistente foi implementado e validado com registros reais.

   A action valida o entry, persiste `user_metadata.favorite`, grava Insert ou Update no sync log e emite ResourceEvent. Directory listing e Search hidratam `File.favorite`. A rota global refaz a query ao receber mudanças, evitando manter arquivos que deixaram de ser favoritos. Inspector e Quick Preview usam a action real. Uma fixture temporária confirmou add/remove, busca filtrada, persistência após restart e ResourceEvents em dois clientes; a location, o arquivo e os registros foram removidos após o teste. Tarefa concluída: `TAURI-002`.

3. File Kinds e Search discordavam. Corrigido.

   A causa era a estatística contar três identidades órfãs enquanto a biblioteca não tinha arquivos persistentes. Estatística, busca vazia em modo Fast e navegação agora compartilham o `content_identity.kind_id`. Testes SQLite cobrem identidades órfãs, entradas sem identidade e extensões divergentes. Tarefa concluída: `TAURI-003`.

4. Não existe regressão nativa empacotada.

   Typecheck e build Vite não provam IPC, daemon, codecs, dialogs ou janelas. Tarefa: `TAURI-006`.

### Média prioridade

1. O drop de arquivos externos era absorvido e descartado.

   O handler impedia o comportamento nativo, contava os arquivos e apenas escrevia no console. Ele foi removido. A implementação real está em `TAURI-005`.

2. Coleções geradas usam tipos de tuple incorretos.

   `sources.list` e `sources.list_items` são gerados como `[SourceInfo]` e `[SourceItem]`. Isso descreve exatamente um item, não uma lista variável. Tarefa: `TAURI-007`.

3. Ações visíveis sem backend.

   Install Adapter e Delete Sidecar não faziam nada. Elas foram ocultadas. Devem voltar somente com actions reais.

4. Busca de Sources era visualmente editável, mas ignorava o valor.

   O campo usava `value=""` e callbacks vazios. Agora filtra a lista real.

5. Quick Look do menu era um no-op.

   A ação agora usa o mesmo `openQuickPreview` do atalho de teclado e do Inspector.

### Baixa prioridade ou dívida controlada

- Recents vazio não explica se não há registros ou se a consulta falhou.
- O Overview consulta jobs remotos sem assinatura reativa dedicada.
- Scroll de Grid, List e Media não é restaurado por tab.
- O bundle principal do frontend permanece grande. O build gerou chunks acima de 500 kB.
- MapLibre emite aviso de BigInt para o target configurado. O build conclui, mas browsers antigos podem divergir.

## Mockups e código sem uso removidos

Foram removidos do Overview:

- `mockData.ts`, com devices, volumes e projetos artificiais.
- `ProjectCards.tsx`, que gerava métricas aleatórias.
- `ContentBreakdown.tsx`, que mostrava categorias fixas.

Esses componentes não eram renderizados pela rota atual. O Overview ativo já usa dados reais.

## Correções aplicadas

- Quick Look abre Quick Preview.
- Sources search filtra registros reais.
- File Kinds agora conta somente arquivos navegáveis e abre cartões não vazios no Explorer filtrado.
- O Explorer parou de engolir drops externos sem executar operação.
- Favorite persistente foi implementado no Inspector, Quick Preview, Search e rota Favorites.
- Delete Sidecar sem mutation foi removido.
- Install Adapter sem implementação foi removido.
- Mockups mortos do Overview foram removidos.
- Tasks de Grid, List, Search, Settings e Multi-window foram reconciliadas com o código atual.
- Foi criado o epic `TAURI-000` com sete tarefas verificáveis.
- O `TODO` foi reduzido a um índice priorizado. O histórico concluído continua no Git.

## Definição de pronto recomendada

O app Tauri só deve ser chamado de pronto quando:

1. Nenhum controle visível for no-op ou sucesso simulado.
2. Typecheck, Rust check, task validation e build passarem.
3. O bundle instalar, iniciar, conectar ao daemon e abrir uma biblioteca real.
4. Explorer, busca, preview, operações, jobs e janelas auxiliares passarem no aplicativo empacotado.
5. O mesmo smoke test passar no Linux, macOS e Windows.
6. Favoritos, Spacebot e filtros por tipo tiverem contratos reais ou ficarem explicitamente indisponíveis.

## Limites desta auditoria

- Nenhuma operação destrutiva de arquivo ou reset de dados foi executada.
- Não havia segundo peer para validar sync e Spacedrop.
- Não havia source configurada para validar sync e paginação de adapter.
- O ambiente validado foi Linux Wayland/XWayland em modo dev.
- Mudanças locais preexistentes em streaming de vídeo e áudio foram preservadas. Elas passaram typecheck e Rust check, mas não foram declaradas prontas sem teste do bundle e reprodução real.
