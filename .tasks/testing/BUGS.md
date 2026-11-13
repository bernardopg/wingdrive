# Bugs encontrados na ampliação de cobertura

## Undo da lixeira não encontra itens em caminhos com diretório pai por symlink

- Estado: corrigido em 2026-11-12. Reproduzido com `TMPDIR`, `HOME` e `XDG_DATA_HOME` apontando para um diretório por symlink (`file_undo_test`: 3 itens "no longer in the trash"); com a correção os 18 testes passam no mesmo ambiente.
- Local: `core/src/ops/trash/mod.rs`, `newest_ids` e `select`; journal em `core/src/ops/files/delete/job.rs`.
- Reprodução no Linux: execute `file_undo_test` com HOME e XDG temporários cujo caminho passa pelo symlink `target` desta worktree. `undoing_a_trash_restores_the_items` falha com `it is no longer in the trash` para os dois itens livres. Com os mesmos diretórios pelo caminho físico, os 18 testes passam.
- A biblioteca `trash` canonicaliza o pai antes de mover o item, enquanto a busca do journal compara o caminho registrado literalmente com o caminho da lixeira. Não foi demonstrada perda dos itens, que permanecem na lixeira.
- Correção: `trash_location` reproduz o caminho que a lixeira grava (pai canonicalizado mais o nome do item, sem resolver o próprio item). `trashed_ids` e a restauração por caminho comparam por essa forma e devolvem o id sob o caminho do journal. A restauração continua recusando quando há algo no caminho original (checagem do undo) e a biblioteca `trash` recusa colisão no destino físico, então nada é sobrescrito.
- Testes: `ops::trash::tests::trash_location_resolves_a_symlinked_parent_only` e `items_trashed_through_a_symlinked_folder_are_found_and_restored`.
## Hard link TAR segue link simbólico existente no destino

- Estado: corrigido; testes de regressão passaram em 2026-11-12 (`cargo test -p wing-core --lib archive::`).
- Local: `core/src/ops/archive/engine.rs`, `extract_tar`.
- Reprodução: extraia um TAR contendo hard link `copied` para `source` em um destino onde `source` é um link simbólico para um arquivo externo. A implementação anterior abria o alvo com `File::open` e copiava seu conteúdo para `copied`.
- Impacto: conteúdo legível pelo processo pode ser copiado para o destino da extração. A reprodução foi identificada por análise do código e o teste de regressão cobre o caso.
- Correção: rejeitar origens que não sejam arquivos regulares, abrir antes de criar o destino e usar `O_NOFOLLOW` em Unix. A verificação dos diretórios ancestrais ainda depende de verificações por caminho e não garante proteção contra alterações concorrentes do diretório de extração; isso não foi testado.


## Arquivos antigos no cache de arquivos compactados não são removidos

- Estado: corrigido; testes de regressão passaram em 2026-11-12.
- Local: `core/src/ops/archive/browse.rs`, `prune`.
- Reprodução: coloque um arquivo comum antigo na raiz do cache e execute `prune`. A função tentava removê-lo como diretório e deixava a entrada no disco.
- Impacto: entradas comuns antigas permaneciam no cache e geravam avisos repetidos. Não foi identificado um fluxo normal que crie essas entradas.
- Correção: escolher a operação de remoção pelo tipo da entrada, sem seguir links simbólicos ao decidir como removê-la.


## Avisos do Clippy nos testes de menus de serviço

- Estado: corrigido localmente; `cargo clippy --locked -p file-opening-linux --all-targets -- -D warnings` passou.
- Local: `apps/tauri/crates/file-opening-linux/src/service_menus.rs`.
- Quatro testes clonando valores para construir slices unitários geravam `cloned_ref_to_slice_refs`. Eles agora usam `std::slice::from_ref`, sem alterar o comportamento testado.

## Vazamento de memória ao obter nome de sidecar

- Estado: corrigido localmente. `file_name` retorna `Cow`: empresta nomes físicos e de nuvem e possui a string gerada para sidecar. O teste de endereçamento cobre as duas formas; não houve medição de memória em execução prolongada.
- Local: `core/src/domain/addressing.rs`, `WingPath::file_name`, variante `Sidecar`.
- Reprodução: crie um `WingPath::sidecar` e chame `file_name()` repetidamente. Cada chamada executa `format!(...).into_boxed_str()` e `Box::leak`, inclusive quando o resultado é descartado.
- Resultado: cada chamada mantém a alocação pelo restante do processo. Chamadas repetidas podem aumentar a memória do daemon sem limite.
- Correção: o retorno agora é `Option<Cow<'_, str>>`, e os chamadores internos foram ajustados. Essa é uma mudança na API Rust para consumidores externos.

## Entrada inválida pode causar panic no caminho de sidecar do desktop

- Estado: corrigido localmente e testado chamando a função de comando diretamente, sem iniciar o runtime Tauri.
- Local: `apps/tauri/src-tauri/src/files.rs`, `get_sidecar_path`.
- Reprodução: após resolver uma biblioteca existente, invoque o comando com `content_uuid` menor que quatro bytes, por exemplo `"a"`, ou com caracteres multibyte que cruzem os índices 2 ou 4.
- Resultado: os slices `content_uuid[0..2]` e `[2..4]` causam panic em vez de retornar erro; o comportamento observado no processo hospedeiro depende de como a chamada Tauri trata esse panic.
- Correção: validar e normalizar o UUID antes de procurar a biblioteca. O teste cobre strings curtas, Unicode, UUID inválido e normalização de UUID válido.

## Junção de caminhos de nuvem duplica separador

- Estado: corrigido localmente; teste `cloud_join_does_not_duplicate_separators` em `core/tests/addressing_contract_test.rs` aprovado.
- Local: `WingPath::join` em `core/src/domain/addressing.rs`.
- Reprodução: junte `s3://photos/2026/` com `/image.jpg`. A saída é `s3://photos/2026//image.jpg`, em vez de `s3://photos/2026/image.jpg`.
- Impacto: a chave resultante pode identificar outro objeto no armazenamento de nuvem. Não foi verificado um fluxo de interface que produza essa combinação.
- Correção: remover uma barra no encontro dos componentes e tratar o caminho vazio da raiz. Barras adicionais na chave são preservadas.

## Validador de cor aceita caracteres não hexadecimais

- Estado: corrigido localmente; teste `space_color_rejects_non_hex_characters` em `core/tests/space_contract_test.rs` aprovado.
- Local: `Space::validate_color` em `core/src/domain/space.rs`, usado pelas ações de criação e atualização de espaços.
- Reprodução: `Space::validate_color("#zzzzzz")` retorna `true`; a função verifica apenas o prefixo e o tamanho.
- Impacto: as ações podem aceitar uma cor que não representa um valor hexadecimal válido. O resultado visual não foi verificado.
- Correção: exigir seis dígitos hexadecimais ASCII após o prefixo `#`.

## Limite de nome de tag conta bytes como caracteres

- Estado: corrigido localmente; teste `tag_name_limit_counts_unicode_characters` em `core/tests/tag_validation_contract_test.rs` aprovado.
- Local: `TagValidator::validate_tag_name` em `core/src/ops/tags/validation/tag_validator.rs`.
- Reprodução: um nome com 128 caracteres `é` é rejeitado por exceder 255, pois ocupa 256 bytes em UTF-8. A mensagem informa um limite de 255 caracteres e a função aceita Unicode.
- Impacto: nomes com caracteres multibyte atingem o limite antes dos nomes ASCII. O uso efetivo desse validador no fluxo de criação ainda não foi demonstrado.
- Correção: contar até 255 valores escalares Unicode, em linha com a mensagem de caracteres. A entrada da ação de criação também foi corrigida, incluindo o limite de namespace e a validação hexadecimal da cor. O teste cobre 128, 255 e 256 caracteres multibyte; grafemas compostos podem conter mais de um valor escalar.

## Criação de tag atribuía a biblioteca como dispositivo criador

- Estado: corrigido localmente; reproduzido e validado em `test_tag_creation_and_application_with_events`.
- Local: `core/src/ops/tags/create/action.rs`, `CreateTagAction::execute`.
- Antes: a ação preenchia `created_by_device` e a origem das aplicações iniciais com `library.id()`, em vez do identificador do dispositivo.
- Correção: obter o ID do `device_manager` do contexto e retornar erro caso não esteja disponível. O teste integrado verifica o ID persistido contra o dispositivo do fixture.
- Limite: a alteração não migra registros já gravados com ID incorreto.

## Busca não encontrava uma tag somente pelo alias

- Estado: corrigido localmente; reproduzido e validado em `searches_aliases_and_respects_namespace_and_type_filters`.
- Local: `core/src/ops/tags/manager.rs`, `TagManager::search_tags`.
- Antes: quando a busca textual no nome não retornava IDs, a função retornava antes de consultar os aliases. Uma tag chamada `Project` com alias `uniquealias` não aparecia ao buscar `uniquealias`.
- Correção: prosseguir com a busca por aliases mesmo quando a busca inicial está vazia. O teste também verifica namespace, tipo e resultado ausente.

## Aviso de incompatibilidade futura em dependência Rust

- Estado: confirmado na compilação instrumentada, sem falha de build atual.
- Dependência: `proc-macro-error2` 2.0.1, `src/lib.rs:494` no pacote da dependência.
- O compilador avisa que o reexport público de `proc_macro`, declarado privado, será rejeitado por uma versão futura do Rust (`E0365`, issue rust-lang/rust#127909).
- Avaliar atualização ou correção da dependência e conferir a árvore de dependências antes de alterar o lockfile. O aviso não identifica um erro de código próprio do WingDrive.

## Harness removia diretórios de teste com nomes fixos

- Estado: corrigido localmente; os testes de preflight, incluindo isolamento, passaram com `HOME` e `XDG_CONFIG_HOME` temporários.
- Local: `core/tests/helpers/test_data.rs`, `TestDataDir::with_mode`.
- Reprodução anterior no Linux: duas instâncias com o mesmo nome usavam o mesmo diretório em `/tmp` ou `HOME`, e a nova instância removia o caminho antes de criá-lo.
- Impacto: execuções simultâneas podiam apagar os dados umas das outras, assim como qualquer conteúdo preexistente naquele caminho.
- Correção: cada instância usa um UUID e cria seu próprio diretório de forma exclusiva, sem remover um diretório anterior. O teste `test_data_directories_are_unique_and_clean_up_independently` verifica a separação e a limpeza independente.

## Servidor de mídia do desktop anunciava bytes inexistentes

- Estado: corrigido; testes em `apps/tauri/src-tauri/src/server/tests/mod.rs` aprovados.
- Local: `apps/tauri/src-tauri/src/server.rs`, `serve_file` e `parse_range`.
- Reprodução anterior: um arquivo vazio sem `Range` respondia `Content-Length: 1` sem corpo; `Range: bytes=0-` no mesmo arquivo recebia `206` em vez de `416`; `Range: bytes=6-2` num arquivo não vazio recebia `206` com intervalo invertido.
- Correção: arquivo vazio anuncia comprimento zero; intervalo sobre arquivo vazio retorna `416` com `Content-Range: bytes */0`; intervalo invertido é ignorado e recebe a resposta completa, como os demais intervalos malformados.
- Limite: um caso passa pelo roteador HTTP real em loopback com token e intervalo; os demais chamam os handlers diretamente. Não houve validação no player da interface.

## Rótulo de montagem GVFS decodificava escape incompleto

- Estado: corrigido; testes em `apps/tauri/src-tauri/src/gvfs/tests/mod.rs` aprovados.
- Local: `apps/tauri/src-tauri/src/gvfs.rs`, `percent_decode`.
- Reprodução anterior: `user%A` virava `user` seguido de quebra de linha, pois a função aceitava um único dígito hexadecimal após `%`.
- Correção: decodificar só escapes com dois dígitos hexadecimais ASCII e preservar os demais caracteres. UTF-8 em escapes válidos continua funcionando.
- Limite: sem montagem GVFS real nem execução de `gio`.

## Instâncias nomeadas distintas compartilhavam o nome D-Bus

- Estado: corrigido; teste `valid_instance_names_do_not_collide_on_the_session_bus` aprovado.
- Local: `apps/tauri/src-tauri/src/launch.rs`, `single_instance_id`.
- Reprodução anterior: `work-a`, `work_a`, `work_2da`, `café` e `cafê`, todos aceitos por `WINGDRIVE_INSTANCE`, geravam só três IDs. Uma abertura podia ir para a instância errada.
- Correção: cada byte não alfanumérico vira `_` seguido de dois dígitos hexadecimais, inclusive o próprio `_`. A instância padrão e nomes só com letras e números ASCII mantêm o ID. Um nome Unicode de 64 bytes continua um nome D-Bus válido.
- Compatibilidade: instâncias com hífen, sublinhado ou Unicode mudam de nome D-Bus. Uma instância de versão anterior já aberta não recebe a abertura de uma versão nova com o mesmo nome. Não houve teste entre versões.

## URI de lançamento aceitava escape percentual com sinal

- Estado: corrigido; teste `file_uri_rejects_signed_percent_escapes` aprovado.
- Local: `apps/tauri/src-tauri/src/launch.rs`, `percent_decode`.
- Reprodução anterior: `file:///tmp/%+A` era aceito como caminho com quebra de linha, pois `u8::from_str_radix` aceita o sinal `+`.
- Correção: rejeitar escape que não tenha exatamente dois dígitos hexadecimais ASCII.

## Caminho de sidecar do desktop aceitava segmentos com `..`

- Estado: corrigido na revisão; teste `sidecar_segments_that_could_leave_the_content_folder_return_an_error` aprovado.
- Local: `apps/tauri/src-tauri/src/files.rs`, `get_sidecar_path`.
- Reprodução anterior: com UUID válido, `variant = "../../etc/passwd"` ou `kind = "../thumb"` gerava um caminho fora da pasta do conteúdo. O servidor de mídia já rejeitava esses segmentos; o comando IPC não.
- Correção: o comando usa a mesma validação do servidor (`valid_sidecar_segments`) antes de montar o caminho.
- Limite: o comando só devolve o caminho; não foi verificado se algum fluxo da interface lê o arquivo resultante.

## Cópia e movimentação de pastas seguiam links simbólicos

- Estado: corrigido; os quatro casos de `core/tests/copy_symlink_test.rs` falhavam antes da correção e passam depois.
- Local: `core/src/ops/files/copy/strategy.rs`, `copy_file_streaming` (percurso da pasta) e `get_path_size`, usado por `LocalMoveStrategy`.
- Reprodução anterior: uma pasta com `self -> .`, um link para outra pasta e um link quebrado. Cópia com `CopyMethod::Streaming` ou `Atomic` terminava sem falhas, mas `dest/source/self` virava uma pasta real com cópias aninhadas da própria pasta até a resolução de links falhar, e o link quebrado sumia sem erro. A movimentação `Atomic` da mesma pasta falhava com `No such file or directory`, pois o cálculo de tamanho seguia o link quebrado.
- Impacto: a cópia gravava dados que não estavam na seleção (o conteúdo do alvo de cada link) e repetia a pasta dentro dela mesma. O job informava sucesso.
- Correção: no Unix, o percurso recria cada link com o mesmo alvo em vez de segui-lo, e o tamanho usa `symlink_metadata`. A movimentação em streaming continua apagando a origem sem tocar no alvo dos links.
- Limite: Windows mantém o comportamento anterior, sem teste. Links no topo da seleção (o link selecionado em si) não mudaram. `FileCopyJob::get_path_size` e `count_files_in_path` ainda seguem links para estimar progresso; ver Melhorias sugeridas.

## Exclusão permanente falhava em link simbólico quebrado

- Estado: corrigido; `permanent_delete_removes_links_without_following_them` em `core/tests/delete_strategy_test.rs` falhava antes da correção e passa depois.
- Local: `core/src/ops/files/delete/strategy.rs`, `LocalDeleteStrategy::permanent_delete` e `get_path_size`.
- Reprodução anterior: `permanent_delete` em um link que aponta para um caminho inexistente retornava `No such file or directory (os error 2)` e o link ficava no disco.
- Correção: ler o próprio link com `symlink_metadata` e removê-lo com `remove_file`. O tamanho liberado de um link passa a ser o do link, não o do alvo. O teste confirma que os alvos de links para arquivo e pasta continuam intactos.
- Limite: o modo lixeira (`trash::delete`) não foi testado com links quebrados.

## Revelar arquivo no Linux enviava URI sem codificação

- Estado: corrigido; `reveal_uri_round_trips_paths_with_reserved_characters` em `apps/tauri/src-tauri/src/files/tests/mod.rs` falhava antes da correção e passa depois.
- Local: `apps/tauri/src-tauri/src/files.rs`, `reveal_path` no Linux, agora via `reveal_uri`.
- Reprodução anterior: o comando montava `file://{caminho}` sem codificar. `/tmp/100%20off.txt` virava `/tmp/100 off.txt` ao decodificar, `notes#1.txt` perdia `#1.txt` como fragmento e `what?.txt` virava consulta.
- Impacto: "Mostrar na pasta" pedia ao gerenciador de arquivos um item diferente ou inexistente. O fallback abre a pasta pai só quando a chamada D-Bus falha.
- Correção: gerar a URI com `Url::from_file_path`, que codifica o caminho como o lado receptor (`filemanager1.rs`) decodifica.
- Limite: sem chamada D-Bus real nem gerenciador de arquivos no teste.

## Entrada de autostart não escapava o caminho do executável

- Estado: corrigido; `autostart_entry_escapes_launcher_paths_for_the_exec_key` em `apps/tauri/src-tauri/src/background.rs` falhava antes da correção e passa depois.
- Local: `apps/tauri/src-tauri/src/background.rs`, `autostart_entry`.
- Reprodução anterior: com o executável em `/opt/100%d/a\b"c$d`e/WingDrive`, a linha `Exec` saía com `%d` sem escape, `\` duplicado só uma vez e `$` e `` ` `` sem barra. Pela Desktop Entry Specification, `%d` é um código de campo removido na expansão, e um `\` literal entre aspas exige quatro barras no arquivo.
- Impacto: "Iniciar com o sistema" gravava um comando que apontava para outro caminho quando o AppImage ou o executável ficava num caminho com esses caracteres.
- Correção: aplicar a regra de aspas do `Exec` (`"`, `` ` ``, `$` e `\` com barra), depois duplicar as barras pelo escape de string e trocar `%` por `%%`.
- Limite: o teste compara com a especificação; não houve sessão gráfica executando a entrada.

## Configuração do app tratava mudança de caixa como alteração

- Estado: corrigido; `same_value_in_another_case_is_not_a_change` e `validation_rejects_each_invalid_field` em `core/tests/app_config_action_test.rs` falhavam antes da correção e passam depois.
- Local: `core/src/ops/config/app/update.rs`, `UpdateAppConfigAction::validate` e `execute`.
- Reprodução anterior: com `log_level` salvo como `warn` e tema `nord`, enviar `WARN` e `Nord` respondia `Updated: log_level, theme` com `requires_restart = true`, gravava o arquivo e emitia `ConfigChanged`, embora o valor salvo continuasse igual. Enviar `wingbot_base_url = "/"` (então `spacebot_base_url`) passava na validação e salvava uma URL vazia, pois a barra final é removida depois.
- Correção: comparar nível de log e tema já em minúsculas, como são gravados, e validar a URL do Spacebot depois de remover as barras finais.
- Limite: o teste chama a ação direto com o contexto do harness, sem RPC nem interface.

## Job de cópia substituía item criado depois da confirmação

- Estado: corrigido; `copy_without_a_policy_keeps_a_file_that_appeared_after_the_check` em `core/tests/copy_conflict_test.rs` falhava antes da correção e passa depois.
- Local: `core/src/ops/files/copy/job.rs`, decisão de conflito do `FileCopyJob`.
- Reprodução anterior: um job sem política de conflito e com `overwrite = false` copiava `report.txt` sobre um `report.txt` já existente no destino, com `Streaming` e `Atomic`. O arquivo original virava o novo conteúdo e o job contava a cópia como bem-sucedida.
- Impacto: a ação de cópia pergunta sobre conflitos antes de enfileirar o job, mas o job roda depois. Dois jobs na fila para a mesma pasta, ou outro programa criando o mesmo nome nesse intervalo, levavam à substituição sem pergunta. O campo `CopyOptions::overwrite` não era lido pelo job.
- Correção: um conflito sem política só é substituído com `overwrite = true`; caso contrário, aquele item falha com `Destination already exists` e os demais seguem. A mesma entrada no disco, comparada sem seguir links, não conta como conflito, para manter renomeações só de maiúsculas em volumes que ignoram caixa.
- Limite: um job retomado depois de interrupção, com uma pasta parcialmente copiada no destino e sem política, agora falha nessa pasta em vez de completá-la por cima. Não houve teste de retomada nem de volume que ignora caixa.

## Renomear para um nome existente movia o item para dentro da pasta

- Estado: corrigido; `rename_onto_an_existing_folder_does_not_move_into_it`, `rename_onto_an_existing_file_fails_and_keeps_both` e `renaming_a_link_to_its_target_name_keeps_the_target` em `core/tests/copy_conflict_test.rs`.
- Local: `core/src/ops/files/copy/job.rs` (destino do `MoveMode::Rename`) e `core/src/ops/files/rename/action.rs`, que não verifica se o nome novo já existe.
- Reprodução anterior: renomear `notes.txt` para `docs`, sendo `docs` uma pasta existente, movia o arquivo para `docs/notes.txt` e reportava `moved_count = 1`. Renomear `draft.txt` para `final.txt` existente resolvia o destino como o próprio `draft.txt`, não fazia nada e também reportava sucesso.
- Correção: a renomeação usa o caminho pedido como destino exato, sem a regra de soltar dentro de pasta; com a correção do conflito acima, um nome ocupado falha e mantém os dois itens.
- Limite: a ação de renomear continua sem validar o nome antes de criar o job; o erro aparece no resultado do job.

## Testes do task-system dependiam da ordem de execução

- Estado: corrigido; `cargo test -p wing-task-system --test integration_test` passou três vezes seguidas com 10 testes.
- Local: `crates/task-system/tests/integration_test.rs`, `many_pauses_test`.
- Reprodução anterior: na execução de `scripts/coverage-rust.sh` de 2026-11-06, 9 dos 10 testes falharam. `many_pauses_test` instalava um subscriber global de tracing; quando rodava antes dos testes com `#[traced_test]`, o primeiro deles parava com `a global default trace dispatcher has already been set` e os demais com `Once instance has previously been poisoned`.
- Correção: `many_pauses_test` também usa `#[traced_test]`, sem alterar `RUST_LOG` nem instalar subscriber global.
- Limite: só afeta o runner de cobertura, que testa os crates compartilhados; o CI normal não roda essa suíte.

## Verificação de checksum da cópia nunca detectava erro

- Estado: corrigido; `written_file_check_reads_the_destination_back` em `core/src/ops/files/copy/strategy.rs`.
- Local: `copy_single_file_with_offset`, usado pela cópia em streaming com `verify_checksum`.
- Antes: os dois hashers recebiam o mesmo bloco lido da origem, então a comparação era sempre igual.
- Correção: depois do `sync_all`, `verify_written_file` relê o destino do disco e compara com o hash da origem. Diferença remove a cópia e retorna `InvalidData`.
- Limite: o teste corrompe o destino diretamente; não simula falha de hardware nem cache do sistema de arquivos.

## Estimativa de progresso da cópia seguia links

- Estado: corrigido; `totals_count_inner_links_once_without_following_them` em `core/src/ops/files/copy/job.rs`.
- Local: `FileCopyJob::get_path_size` e `count_files_in_path`, agora em `tree_totals`.
- Antes: um link para `/` dentro da seleção percorria o disco inteiro e um link para uma pasta ancestral não terminava, antes mesmo de a cópia começar.
- Correção: o caminho selecionado continua sendo seguido; links dentro dele contam como um item do próprio tamanho, como a cópia os recria.

## Limite de nome na renomeação usava a unidade errada

- Estado: corrigido; `length_limit_uses_the_platform_unit` em `core/src/ops/files/rename/validation.rs`.
- Antes: contava bytes em todas as plataformas e a mensagem dizia caracteres. No Windows, cujo limite é de 255 unidades UTF-16, 128 `é` eram recusados.
- Correção: bytes no Linux e no macOS, unidades UTF-16 no Windows; a mensagem nomeia a unidade.
- Limite: o caso do Windows só roda no CI do Windows.

## Nome alternativo perdia bytes fora de UTF-8

- Estado: corrigido; `free_name_keeps_bytes_that_are_not_utf8` em `core/src/ops/files/plan.rs`.
- Local: `next_free_name`, usado pela política "manter os dois".
- Antes: `to_string_lossy` trocava esses bytes por `U+FFFD`, então `caf\xff.txt` virava `caf� (1).txt`.
- Correção: o nome é dividido nos bytes do `OsStr`, junto ao `.` ASCII, e o número entra sem converter o restante.

## Renomear para um nome ocupado só falhava dentro do job

- Estado: corrigido; `rename_action_refuses_a_taken_name_before_queueing_a_job` em `core/tests/copy_conflict_test.rs`.
- Local: `FileRenameAction::validate`.
- Antes: a ação aceitava o nome e o erro só aparecia no resultado do job.
- Correção: a validação recusa um nome já usado por arquivo, pasta ou link, com `field = "new_name"`. Uma renomeação só de maiúsculas que encontra o próprio item continua aceita.

## Descoberta de servidores ignorava `Host=nome` no ssh config

- Estado: corrigido; caso novo em `ssh_config_lists_concrete_hosts` (`apps/tauri/src-tauri/src/network_discovery.rs`).
- Antes: só `Host nome` era lido; `Host=nome` e `Host = nome`, aceitos pelo OpenSSH, eram ignorados.

## Teste do daemon do desktop ignorava `CARGO_TARGET_DIR`

- Estado: corrigido em `start_daemon_integration_tests::workspace_daemon_binary`.
- Antes: procurava `wing-daemon` só em `<workspace>/target`, e falhava num worktree que compila em outro diretório.

## Testes de lixeira usavam a lixeira real

- Estado: corrigido em `core/tests/delete_strategy_test.rs`; caso novo `trash_moves_a_dangling_link_into_the_isolated_trash`.
- Antes: `trash::delete` real num arquivo de `/tmp` deixava itens em `/tmp/.Trash-<uid>` ou na lixeira do usuário.
- Correção: `XDG_DATA_HOME` aponta para uma pasta temporária, e os arquivos ficam no mesmo volume. O caso novo confirma que um link quebrado vai para a lixeira como link, sem seguir o alvo.

## Achados do teste real em 2026-11-06

Encontrados ao rodar as correções num daemon isolado e no app Tauri de verdade. Os testes unitários anteriores chamavam a estratégia ou o job direto e não passavam por estes caminhos.

### Ação de cópia falhava numa pasta com link quebrado

- Estado: corrigido; `copy_action_queues_a_folder_with_a_dangling_link` em `core/tests/copy_symlink_test.rs`.
- Local: `core/src/ops/files/copy/action.rs`, `calculate_totals`.
- Reprodução: `files.copy` pelo RPC numa pasta com um link quebrado respondia `Failed to read metadata: No such file or directory` antes de criar o job. A correção dos links no job nunca era alcançada pela interface nem pelo CLI.
- Correção: a ação usa a mesma contagem do job (`tree_totals`), que não segue links internos e conta um link quebrado selecionado como um item.

### Job de exclusão dizia que um link quebrado não existia

- Estado: corrigido; `delete_job_removes_a_selected_dangling_link` em `core/tests/copy_symlink_test.rs`.
- Local: `core/src/ops/files/delete/job.rs`, `validate_targets`.
- Reprodução: `files.delete` permanente num link quebrado terminava com `Target does not exist`, embora `permanent_delete` já soubesse removê-lo.
- Correção: verificar a existência com `symlink_metadata`.

### Caminho de sidecar ignorava a instância do app

- Estado: corrigido; `sidecar_path_uses_the_given_data_directory` em `apps/tauri/src-tauri/src/files/tests/mod.rs`.
- Local: `apps/tauri/src-tauri/src/files.rs`, `get_sidecar_path`.
- Reprodução: com `WINGDRIVE_INSTANCE` ou `WINGDRIVE_DATA_DIR`, o comando procurava bibliotecas no diretório padrão e respondia `Failed to read libraries directory`.
- Correção: usar o diretório de dados do daemon da janela (`DaemonState::data_dir`), o mesmo do servidor de mídia.

### "Mostrar na pasta" nunca selecionava o arquivo no Linux

- Estado: corrigido; `show_items_argument_is_a_gvariant_string_array`, e validado com um serviço `FileManager1` no barramento de teste.
- Local: `apps/tauri/src-tauri/src/files.rs`, `reveal_path`.
- Reprodução: o `gdbus call` recebia a URI como texto solto, mas `ShowItems` espera um array (`as`). O `gdbus` recusava com `Erro ao analisar o parâmetro 1 do tipo "as"` e o app sempre caía no fallback de abrir a pasta pai.
- Correção: passar `['<uri>']` em texto GVariant, com `\` e `'` escapados.

### "Mostrar na pasta" ficava pendente com o fallback

- Estado: corrigido; `reveal_fallback_does_not_wait_for_the_folder_opener` falhava antes (`reveal waited 30s`) e passa depois.
- Reprodução: sem `FileManager1`, o `xdg-open` abriu o `kitty` e só retornou quando a janela fechou; o comando ficou pendente por minutos.
- Correção: o fallback roda em segundo plano e só registra falhas no log.

### Autostart rejeitado com `%` no caminho

- Estado: corrigido; `autostart_entry_escapes_launcher_paths_for_the_exec_key` atualizado.
- Reprodução: a entrada seguia a especificação, mas o GLib (`gio launch`, autostart do GNOME) e o `systemd-xdg-autostart-generator`, que este sistema usa, procuram o programa antes de converter `%%` em `%` e recusavam a entrada.
- Correção: com `%` no caminho, o `Exec` começa por `/usr/bin/env`. Validado executando a entrada gerada pelo app com `gio launch` e como unit gerada pelo systemd.
- Limite: o gerador do systemd interpreta `\`, `$` e `` ` `` de forma diferente da especificação. Caminhos com esses caracteres funcionam no GLib, não no systemd.

## Achados da suíte de falhas de cópia em 2026-11-10

`core/tests/copy_failure_test.rs` roda o job de cópia contra cancelamento, falha no meio e itens fora do comum. Seis dos doze casos iniciais falharam; a checagem pelo CLI num daemon isolado achou o sétimo item.

### Cancelar uma sobrescrita destruía o arquivo anterior

- Estado: corrigido; `cancelled_overwrite_keeps_the_previous_file`.
- Local: `core/src/ops/files/copy/strategy.rs`, `copy_single_file_with_offset` e `FastCopyStrategy`.
- Reprodução anterior: copiar `stream.bin` sobre um `stream.bin` existente com `overwrite = true` e cancelar no meio. `File::create` truncava o destino antes de ler a origem, e o cancelamento apagava o parcial. Sobrava nada: nem o conteúdo antigo, nem o novo.
- Correção: cada arquivo é escrito como `.wing-part-<id>` na pasta de destino e renomeado ao terminar. Erro ou cancelamento remove o temporário. A cópia `Atomic` (`std::fs::copy`) segue o mesmo caminho.
- Limite: um daemon morto no meio da cópia deixa um `.wing-part-*` na pasta. Não há limpeza automática.

### Cópia cancelada terminava como concluída com uma falha

- Estado: corrigido; `cancelled_copy_leaves_no_partial_file`.
- Local: `core/src/ops/files/copy/job.rs`, tratamento do erro da estratégia.
- Reprodução anterior: cancelar um job durante um arquivo grande. A estratégia consumia o pedido de interrupção e devolvia um erro comum; o job contava o item como falha e seguia para os próximos itens do lote. O resultado era `Completed` com `failed_count = 1`. Pausar tinha o mesmo efeito.
- Correção: o erro da estratégia carrega a marca de interrupção (`is_interruption`) e o job devolve `JobError::Interrupted`, que o executor trata como cancelado ou pausado.

### `preserve_timestamps` não era lido

- Estado: corrigido; `copies_keep_content_and_modification_time` e `timestamps_are_new_when_the_option_is_off`.
- Reprodução anterior: toda cópia saía com a data do momento da cópia, com a opção ligada (o padrão) ou desligada. Só a movimentação por rename mantinha a data.
- Correção: depois de cada item copiado, o job aplica a data de modificação da origem a arquivos e pastas do destino. Falha nessa etapa só vai para o log.
- Limite: data de acesso e de criação não são copiadas; links ficam com a data em que foram recriados. No Windows, pastas podem manter a data nova.

### Link selecionado era seguido, e link quebrado derrubava o job

- Estado: corrigido no Unix; `a_selected_link_is_copied_and_moved_as_a_link` e `copy_action_accepts_a_selected_dangling_link`.
- Local: `strategy.rs` (as três estratégias locais), `job.rs` (`collect_file_metadata`, `tree_totals`) e `action.rs` (validação das origens e checagem de conflito).
- Reprodução anterior: selecionar um link quebrado junto com outros arquivos. A ação respondia `Source file does not exist`; chamado direto, o job inteiro falhava com `Failed to read metadata`, sem copiar os demais itens. Um link para pasta copiava a pasta inteira, e a movimentação em streaming copiava o alvo e apagava só o link.
- Correção: o link selecionado é o item, como já eram os links dentro de pastas. É recriado com o mesmo alvo, e mover ou renomear age sobre o link.
- Mudança de comportamento: copiar um link para arquivo agora gera um link, não uma cópia do conteúdo. Isso reverte a escolha registrada acima em "Estimativa de progresso da cópia seguia links" ("o caminho selecionado continua sendo seguido"). Windows continua seguindo links.

### Pipe, socket ou dispositivo dentro de uma pasta sumia na movimentação

- Estado: corrigido; `folder_with_a_special_file_is_refused_and_kept`.
- Reprodução anterior: mover em streaming uma pasta com um pipe nomeado. O percurso ignorava o que não fosse arquivo, pasta ou link, a cópia terminava sem erro e o job apagava a pasta de origem com o pipe dentro.
- Correção: a pasta falha com `pipes, sockets and device files are not supported`, antes de copiar conteúdo de arquivo. A origem fica intacta.
- Limite: a cópia (sem mover) da mesma pasta também falha, em vez de pular o item.

### Cópia que falhava na leitura deixava arquivo truncado

- Estado: corrigido junto com a escrita em nome temporário; `copy_that_fails_while_reading_leaves_no_partial_file`.
- Reprodução anterior: copiar `/proc/self/mem`, que abre e falha na primeira leitura, deixava um `mem` vazio no destino.

## URIs de sidecar não reconheciam todos os tipos gerados

- Estado: corrigido localmente; regressão `every_sidecar_kind_and_format_round_trips_through_uri` em `core/tests/addressing_contract_test.rs`.
- Local: `core/src/domain/addressing.rs`, `WingPath::from_uri`.
- Reprodução anterior: gere uma URI de `Thumbstrip` ou `GaussianSplat` com `to_uri` e tente lê-la com `from_uri`. Os diretórios `thumbstrips` e `gaussian_splats` eram recusados com `InvalidSidecarKind`.
- Correção: reconhecer os dois diretórios que os tipos públicos já geram. O teste percorre os sete tipos e os seis formatos, com variante Unicode contendo ponto.
- Limite: valida somente o endereçamento, sem gerar ou resolver arquivos de sidecar.

## JSON de sidecar recusava o próprio formato MessagePack

- Estado: corrigido localmente; regressão `every_sidecar_kind_and_format_round_trips_through_json` em `core/tests/addressing_contract_test.rs`.
- Local: `core/src/ops/sidecar/types.rs`, `SidecarFormat::try_from`, usado pelo desserializador de `WingPath`.
- Reprodução anterior: serialize um `WingPath::Sidecar` com `SidecarFormat::MessagePack` e desserialize o JSON. O valor público `message_pack` falhava com `Invalid sidecar format`.
- Correção: aceitar `message_pack` junto dos aliases existentes `messagepack` e `msgpack`, preservando a representação pública.
- Limite: não altera o formato dos arquivos MessagePack nem os tipos gerados dos clientes.

## Caminhos do provedor genérico de nuvem não faziam ida e volta

- Estado: corrigido localmente; regressão `every_cloud_service_preserves_keys_through_uri_and_json` em `core/tests/addressing_contract_test.rs`.
- Local: `core/src/volume/backend/mod.rs`, `CloudServiceType::from_scheme`.
- Reprodução anterior: construa um `WingPath::cloud` com `CloudServiceType::Other`. Sua URI `cloud://photos/key` retornava `UnknownScheme`, e seu JSON com `service = "cloud"` também era recusado.
- Correção: reconhecer o esquema `cloud` correspondente à variante pública `Other`. O teste verifica os dez provedores, raiz vazia e chaves Unicode com caracteres reservados.
- Limite: nenhum backend de nuvem foi acessado; a correção cobre a identidade dos caminhos.

## Avaliação pública ignorava rejeição por diretórios filhos

- Estado: corrigido localmente; regressão `child_directory_rejection_reaches_the_public_evaluator` em `core/tests/indexing_rules_test.rs`.
- Local: `core/src/ops/indexing/rules.rs`, `IndexerRuler::evaluate_path`.
- Reprodução anterior: configure `RejectIfChildrenDirectoriesArePresent` para `cache`, crie esse subdiretório e avalie a pasta pai. `apply_all` indicava rejeição, mas `evaluate_path` retornava `Accept` porque passava `is_dir = false` à decisão final.
- Correção: usar `metadata.is_dir()` na avaliação. O teste também troca o diretório filho por um arquivo com o mesmo nome e exige aceitação.
- Limite: exercita a API pública das regras; não verifica a reconciliação do watcher nem a remoção de registros já indexados.

## Erro de leitura de diretório aparecia como erro de MessagePack

- Estado: corrigido localmente; regressão `child_rules_distinguish_files_directories_and_missing_paths` em `core/tests/indexing_rules_test.rs`.
- Local: `core/src/ops/indexing/rules.rs`, `accept_dir_for_its_children` e `reject_dir_for_its_children`.
- Reprodução anterior: obtenha metadados de uma pasta e aplique uma regra de diretórios filhos a um caminho inexistente com esses metadados. A falha de `read_dir` virava `RuleParametersRMPDecode(TypeMismatch(...))`, ocultando o erro de filesystem.
- Correção: propagar `Error::Io` com o erro original nas duas regras.
- Limite: o teste usa caminho inexistente; não simula falhas de permissão nem erros durante a iteração de uma pasta já aberta.

## Teste de autostart alterava o XDG global do runner

- Estado: corrigido; o teste existente continua verificando criação, remoção e remoção repetida da entrada.
- Local: `apps/tauri/src-tauri/src/background.rs`, `autostart_file_is_written_and_removed` e `write_autostart`.
- Reprodução anterior: o teste chamava `set_var("XDG_CONFIG_HOME", ...)` no processo compartilhado pelos testes e não restaurava o valor. Outros testes podiam observar o diretório temporário ou um caminho já removido.
- Correção: executar o teste existente num subprocesso com HOME/XDG temporários, sem alterar o ambiente do runner. Separar a gravação da entrada da resolução do ambiente permite que os casos novos verifiquem erros de criação e remoção sem apagar dados existentes.
- Limite: sem sessão gráfica nem execução da entrada de autostart. Não foi observado acesso à configuração real do usuário durante a execução inicial.

## Percentual escapado no Exec descartava os arquivos selecionados

- Estado: corrigido; `escaped_percent_file_codes_remain_literal_and_still_append_files` falhava antes da correção e passou depois. A suíte de `file-opening-linux` terminou com 28 testes unitários e 1 doctest aprovados.
- Local: `apps/tauri/crates/file-opening-linux/src/desktop_entry.rs`, `command_lines`.
- Reprodução anterior: com `Exec=viewer --label=%%f %%u` e os arquivos `/first.txt` e `/second.txt`, a função produzia dois comandos `viewer --label=%f %u`, ambos sem os arquivos. A busca por substring confundia o percentual literal com os códigos `%f` e `%u`.
- Correção: consumir cada par de percentual e código antes de decidir se a entrada aceita um arquivo por comando. Sem código real de arquivo, os dois caminhos são anexados a um único comando. Outro caso verifica que `%%%f` ainda expande o percentual literal seguido do código real.
- Limite: o teste verifica os argumentos gerados, sem executar um aplicativo gráfico ou consultar associações reais do usuário.

## Servidor de mídia respondia 403 para pasta no Windows

- Estado: corrigido; `file_stream_rejects_missing_files_directories_and_relative_paths` falhou no job `Rust Workspace (Windows)` do PR #359 (`403` em vez de `400`) e define o comportamento esperado.
- Local: `apps/tauri/src-tauri/src/server.rs`, `serve_file`.
- Reprodução anterior: pedir uma pasta ao `/file`. No Unix a pasta abre e a checagem `is_dir` responde `400`; no Windows `File::open` falha com acesso negado, que virava `403`.
- Correção: um erro de abertura numa pasta responde `400` nas duas plataformas.
- Limite: conferido só pelo CI do Windows; sem máquina Windows à mão.

## CLI: busca com `--wing-path` inválido pesquisava a biblioteca inteira

- Estado: corrigido; `search_scope_errors_are_rejected_by_the_actual_command_tree` e os testes de `domains/search/args.rs` cobrem endereço inválido e válido.
- Local: `apps/cli/src/domains/search/args.rs`, `FileSearchArgs`.
- Reprodução anterior: `wing search files report --wing-path content://invalid` era aceito. A conversão imprimia um aviso e usava `SearchScope::Library`, ampliando a busca em vez de restringi-la ao caminho pedido.
- Correção: o Clap interpreta o valor com `WingPath::from_uri` e rejeita o endereço que não converte. Um endereço válido continua virando `SearchScope::Path`.

## CLI: busca aceitava `--wing-path` e `--location` juntos

- Estado: corrigido; a rejeição é verificada no parser de busca e na árvore real de comandos.
- Local: `apps/cli/src/domains/search/args.rs`, `FileSearchArgs`.
- Reprodução anterior: com as duas opções, a conversão usava o caminho e descartava a location sem avisar.
- Correção: `conflicts_with = "location"` faz o parser recusar a combinação e nomear as duas opções.

## CLI: indexação tratava endereço inválido como caminho local

- Estado: corrigido; `malformed_and_nonlocal_addresses_are_rejected_by_all_index_commands` cobre endereço inválido, esquema desconhecido, nuvem e outro dispositivo nos três comandos.
- Local: `apps/cli/src/domains/index/args.rs`, `index start`, `index quick-scan` e `index browse`.
- Reprodução anterior: um valor que falhava em `WingPath::from_uri`, como `content://invalid`, caía em `WingPath::local` e chegava ao core como o caminho literal `content://invalid`.
- Correção: os três comandos usam `local_index_path`, que retorna erro para endereço malformado e para endereço que não é local. Caminhos sem esquema continuam aceitos.

## CLI: checagem de conflito da cópia perdia conflitos reais

- Estado: corrigido; três dos quatro testes de `domains::file::conflict_tests` falhavam antes da correção.
- Local: `apps/cli/src/domains/file/mod.rs`, `check_for_simple_conflicts`.
- Reprodução anterior: com duas origens e conflito só na segunda, a função retornava `false`, pois olhava apenas a primeira. Um link simbólico quebrado no destino também retornava `false`, porque `metadata` segue o link. Com destino em outro dispositivo, um arquivo local de mesmo caminho podia gerar um falso conflito.
- Impacto: o CLI decide por esse resultado se pede confirmação antes de sobrescrever. O efeito no job do daemon não foi testado.
- Correção: verificar todas as origens locais, usar `symlink_metadata` para enxergar o próprio link e ignorar o disco local quando o destino não é deste dispositivo. Erros de acesso diferentes de `NotFound` são propagados.
- Limite: a checagem continua local e sujeita a mudanças no filesystem entre a consulta e a execução do job.

## CLI: Wingdrop enviava caminhos relativos ao daemon

- Estado: corrigido; `wingdrop_relative_path_is_resolved_before_sending` falhava antes da correção.
- Local: `apps/cli/src/domains/network/args.rs`, `From<WingdropSendArgs> for WingdropSendInput`.
- Reprodução anterior: `WingPath::from_uri` aceita um caminho sem esquema como local e o mantinha relativo. O daemon, que tem outro diretório de trabalho, resolveria `relative-file` em outra pasta. Não houve transferência real.
- Correção: o Wingdrop usa o mesmo `local_path` dos comandos de arquivo, que resolve o caminho a partir do diretório do CLI e mantém `local://<dispositivo>/...` como endereço remoto. O teste cobre os dois casos.

## CLI: filtro `--status` de jobs aceitava valor desconhecido

- Estado: corrigido; `job_list_rejects_unknown_status` e `job_monitor_rejects_unknown_status` falhavam antes da correção.
- Local: `apps/cli/src/domains/job/args.rs`, `JobListArgs` e `JobMonitorArgs`.
- Reprodução anterior: `wing job list --status runnnning` passava pelo parser. A conversão para `JobStatus` falhava e virava `None`, o mesmo que listar sem filtro.
- Correção: um `value_parser` recusa o valor e lista os seis status aceitos. Outro teste confirma a conversão dos seis status válidos e a ausência de filtro sem a opção.

## CLI: `job pause`, `resume` e `cancel` saíam com sucesso quando falhavam

- Estado: corrigido; os três testes `failed_*_returns_error` em `apps/cli/src/tests/rpc.rs` falhavam antes da correção.
- Local: `apps/cli/src/domains/job/mod.rs`.
- Reprodução anterior: com resposta `success: false` do daemon, os comandos imprimiam a falha e retornavam `Ok(())`. Scripts recebiam código de saída 0.
- Correção: os três comandos retornam erro quando `success` é falso. No formato JSON a resposta estruturada continua sendo impressa antes do erro.
- Limite: respostas RPC sintéticas em loopback, sem daemon real.

## CLI: `wing logs` ignorava `--data-dir` e `--instance`

- Estado: corrigido; `log_path_uses_the_selected_data_directory` falhava antes da correção.
- Local: `apps/cli/src/domains/logs/mod.rs`, `get_daemon_log_path`, e `apps/cli/src/context.rs`.
- Reprodução anterior: a função usava `wing_core::config::default_data_dir()` e devolvia o log da instalação padrão, mesmo com outro diretório de dados ou outra instância selecionada.
- Correção: `Context` guarda o diretório do daemon (`daemon_data_dir`), já resolvido para a instância nomeada, e a busca de logs parte dele. Os testes usam diretórios temporários e não leem logs reais.
- Limite: o fluxo de streaming (`logs follow`) não foi executado contra um daemon real.

## CLI: `--since` rejeitava o formato mostrado na própria ajuda

- Estado: corrigido; `accepts_documented_naive_timestamp_as_utc` falhava antes da correção.
- Local: `apps/cli/src/domains/sync/mod.rs`, `parse_time_filter`.
- Reprodução anterior: `2025-12-03 10:00:00`, exemplo da ajuda de `wing sync events --since`, retornava erro porque `DateTime::parse_from_str` exige fuso.
- Correção: data sem fuso é lida como UTC, e a ajuda de `sync events` e `sync metrics` diz isso. Datas RFC 3339 mantêm o offset informado.

## CLI: duração extrema em `--since` causava panic

- Estado: corrigido; a regressão foi reproduzida sem daemon.
- Local: `apps/cli/src/domains/sync/mod.rs`, `parse_duration` e `parse_time_filter`.
- Reprodução anterior: `--since '9223372036854775807 weeks ago'` estourava a multiplicação em build de debug.
- Correção: multiplicação, construção da `Duration` e subtração da data usam operações verificadas; valor fora do intervalo vira erro.

## CLI: exportação SQL não escapava apóstrofo no tipo de modelo

- Estado: corrigido; `sql_export_escapes_quotes_in_model_types` falhava antes da correção.
- Local: `apps/cli/src/domains/sync/mod.rs`, `format_events_sql`.
- Reprodução anterior: um modelo chamado `name'with-quote` era interpolado num literal SQL sem duplicar o apóstrofo, gerando SQL inválido.
- Correção: duplicar o apóstrofo, como já era feito para resumo e detalhes. O teste verifica o texto gerado, sem executar SQL.

## CLI: relatório de `index verify` causava panic com caminho Unicode longo

- Estado: corrigido; `index_verify_renders_long_unicode_paths_without_panicking` falhava antes da correção.
- Local: `apps/cli/src/domains/index/mod.rs`, listas de entradas ausentes e obsoletas.
- Reprodução anterior: um caminho com 60 caracteres `é` era cortado num índice de byte fora da fronteira UTF-8.
- Correção: `report_path` abrevia por caracteres. Outro teste cobre o limite de truncamento.

## CLI: `wing status` causava panic com `node_id` Unicode longo

- Estado: corrigido; regressão com resposta RPC sintética.
- Local: `apps/cli/src/main.rs`, tabela de rede do `status`.
- Reprodução anterior: um `node_id` com 26 caracteres `é` era cortado no byte 47, fora da fronteira UTF-8.
- Correção: abreviar por caracteres.
- Limite: não foi demonstrado que um daemon real produza esse identificador.

## CLI: `wing library info <id>` consultava a biblioteca selecionada

- Estado: corrigido; o teste RPC sintético falhava ao comparar o ID enviado.
- Local: `apps/cli/src/domains/library/mod.rs`, `library info`.
- Reprodução anterior: com uma biblioteca selecionada, o comando enviava ao daemon o ID selecionado em vez do informado.
- Correção: usar o ID explícito no contexto da consulta; sem argumento, vale a biblioteca selecionada.

## CLI: monitor de eventos causava panic com ID de job curto

- Estado: corrigido; `short_job_id_is_rendered_without_panicking` falhava antes da correção.
- Local: `apps/cli/src/domains/events/mod.rs`, `summarize_event`, e `apps/cli/src/domains/logs/mod.rs`.
- Reprodução anterior: `&job_id[..8]` causava panic com ID menor que oito bytes ou com caractere multibyte cruzando o oitavo byte.
- Correção: `short_job_id` limita por caracteres. O streaming de logs recebeu o mesmo tratamento, sem teste com stream real.

## CLI: monitor de eventos omitia o campo alterado em `ConfigChanged`

- Estado: corrigido; `config_change_summary_includes_changed_field` falhava antes da correção.
- Local: `apps/cli/src/domains/events/mod.rs`, `summarize_event`.
- Reprodução anterior: um braço anterior do `match` respondia só `Configuration changed` e tornava inalcançável o braço que incluía o nome do campo.
- Correção: remover o braço duplicado.

## Desktop: "Open Library..." tratava a recusa do daemon como sucesso

- Estado: corrigido; `opening_a_library_returns_the_daemon_answer_or_its_error` em `apps/tauri/src-tauri/src/main/tests/events.rs`.
- Local: `apps/tauri/src-tauri/src/main.rs`, item de menu `open-library` (agora `handle_menu_event` e `open_library_request`).
- Reprodução anterior: escolher uma pasta sem biblioteca. O daemon responde `{"Error": {"ValidationError": ...}}`; o handler aceitava qualquer JSON, emitia `library-opened` com o erro e não mostrava o diálogo de falha. Um caminho de seleção que não era local virava `.`, a pasta de trabalho do processo.
- Correção: `open_library_request` devolve o conteúdo de `JsonOk` e transforma `Error` e resposta vazia em mensagem; o menu mostra o diálogo de erro nesses casos e recusa seleção não local. O seletor deixa de abrir na pasta de trabalho do processo.
- Limite: o diálogo nativo não roda no `MockRuntime`; os testes cobrem a requisição e as respostas com um listener TCP.

## Desktop: ID da biblioteca entrava sem escape no script injetado nas janelas

- Estado: corrigido; `injected_globals_are_json_string_literals`.
- Local: `apps/tauri/src-tauri/src/main.rs`, `set_library_id` e `set_current_library_id`.
- Reprodução anterior: um ID com aspas, como `x"; alert(1); "`, fechava o literal de `window.__WINGDRIVE_LIBRARY_ID__` e o resto rodava como script em todas as janelas.
- Correção: `globals_script` escreve a URL e o ID como literais JSON.
- Limite: o ID vem do próprio frontend; não há caminho conhecido para um valor externo chegar aqui.

## Desktop: assinatura padrão do Tauri não recebia ofertas do Wingdrop

- Estado: corrigido; `default_subscription_matches_the_typescript_client` lê `packages/ts-client/src/event-filter.ts` e compara as listas.
- Local: `get_default_event_subscription` em `apps/tauri/src-tauri/src/main.rs`.
- Reprodução anterior: a lista Rust, que diz espelhar `DEFAULT_EVENT_SUBSCRIPTION`, não tinha `WingdropOffered` e `WingdropUpdated`. Uma assinatura sem `eventTypes` não recebia ofertas. Os comentários dos dois lados diziam que `JobProgress` ficava de fora, mas as duas listas o incluem e o gerenciador de jobs o consome.
- Correção: os dois eventos entram na lista Rust e os comentários passam a descrever o que é excluído (`LogMessage`, `IndexingProgress`).
- Limite: o ts-client sempre envia `eventTypes`; o padrão Rust só vale para chamadas sem a lista.

## Desktop: rotas de janela não codificavam IDs vindos do frontend

- Estado: corrigido; `window_routes_encode_identifiers_from_the_frontend` em `apps/tauri/src-tauri/src/windows/tests/mod.rs`.
- Local: `WingDriveWindow::create` em `apps/tauri/src-tauri/src/windows.rs`.
- Reprodução anterior: um `context_id` ou `file_id` com `/` ou `:`, válidos num label de janela, entrava cru na rota (`/media-viewer/x/y`, `/contextmenu?context=a/b:c`) e mudava o caminho lido pelo frontend. A rota do Explorer já codificava; as outras não.
- Correção: settings, inspector, media viewer, quick preview, drag overlay e context menu passam o ID por `percent_encode`.

## Desktop: overlay de voz ficava escondido sem informação de monitor

- Estado: corrigido; `voice_overlay_toggle_closes_an_open_overlay`.
- Local: `WingDriveWindow::VoiceOverlay` em `apps/tauri/src-tauri/src/windows.rs`.
- Reprodução anterior: a janela era criada invisível e, se `position_overlay_window` falhasse (sem monitor atual), o comando retornava erro com a janela já criada. Ela não aparecia, e o próximo atalho só a fechava.
- Correção: a falha de posicionamento vira aviso no log e a janela é mostrada onde o gerenciador de janelas a colocar. `resize_overlay_window` continua retornando o erro.

## Desktop: descoberta mDNS perdia servidores numa rede movimentada

- Estado: corrigido; `browsing_parses_what_avahi_printed_even_when_it_is_stopped` em `apps/tauri/src-tauri/src/network_discovery.rs`.
- Local: `browse_mdns` (agora `browse_with`).
- Reprodução anterior: a função esperava o `avahi-browse` terminar e só depois lia o stdout. Uma listagem maior que o buffer do pipe (64 KiB no Linux) bloqueava o processo, que só parava ao ser morto no limite de tempo; o diálogo esperava o limite inteiro e recebia a listagem truncada. O teste gera 2.000 linhas (~150 KiB).
- Correção: uma thread lê o stdout enquanto o processo roda.
- Limite: verificado com um script no lugar do `avahi-browse`, sem rede mDNS real.

## Core: listagem de mídia nunca indicava resultados além do limite

- Estado: corrigido; `media_listing_reports_more_results_than_the_limit` em `core/tests/file_queries_test.rs`.
- Local: `core/src/ops/files/query/media_listing.rs`, `files.media_listing`.
- Reprodução anterior: com `limit: 3` numa pasta com quatro imagens, a consulta usava `LIMIT 3` e depois comparava `total_count > limit` com as mesmas três linhas, então `has_more` era sempre `false`. `total_count` também nunca passava do limite.
- Correção: a consulta busca uma linha além do limite, devolve só as `limit` primeiras e marca `has_more` quando a linha extra existe.
- Limite: a visão de mídia do Explorer pede `limit: 10000` e não lê `has_more`; o efeito visível é uma pasta com mais de 10 mil mídias truncada sem aviso.

## Core: listagem de mídia tratava `_` e `%` no nome da pasta como curinga

- Estado: corrigido; `media_listing_matches_folder_names_literally`.
- Local: `files.media_listing` com `include_descendants: true`.
- Reprodução anterior: em `album_x`, o padrão `LIKE 'album_x/%'` também casava `albumYx/sub/`, e a listagem trazia a mídia da pasta vizinha. O teste falhava com `["inside", "outside"]`.
- Correção: o padrão vem de `directory_subtree_patterns`, que escapa `%`, `_` e `\` e já era usado pela busca e pelo preflight; a consulta declara `ESCAPE '\'`.

## Core: resolução de caminho não achava pasta com ponto no nome

- Estado: corrigido; `paths_resolve_through_dotted_folders_and_sibling_locations` em `core/tests/file_queries_test.rs`. Com o resolvedor antigo o teste falha em `dotted folder`.
- Local: `PathResolver::find_entry_by_path` e `resolve_physical_to_entry` em `core/src/ops/indexing/path_resolver.rs`, usados por `files.by_path`, cópia e outras operações.
- Reprodução anterior: cada componente do caminho perdia o trecho após o último ponto antes da busca, porque arquivos guardam nome e extensão separados. Pastas guardam o nome inteiro, então `v1.2` virava `v1` e nada abaixo dela era encontrado. Pela mesma regra, `notes.txt` e a pasta `notes` buscavam o mesmo nome sem olhar a extensão, e o resultado dependia da ordem do banco. Extensões em maiúsculas (`README.MD`) não casavam com a extensão guardada em minúsculas.
- A escolha da location comparava strings: `/shelf` aceitava `/shelf2/other.txt`, procurava o arquivo na location errada e desistia sem testar as demais.
- O dispositivo era procurado pelo slug literal, então o placeholder `local` de `WingPath::local`, usado por clientes sem dispositivo registrado, nunca resolvia. As locations eram alcançadas só pelos volumes do dispositivo, e uma location com `volume_id` nulo (preenchido só no primeiro índice) ficava de fora. Os dois casos foram apontados na revisão do PR.
- Correção: cada componente casa uma entrada com o nome inteiro e sem extensão, ou um arquivo com o radical e a extensão em minúsculas; a location é escolhida por `Path::starts_with`, que compara componentes. `local` e o UUID do dispositivo valem como este dispositivo, e as locations são buscadas por `device_id`.

## Core: verificação de índice acusava a própria pasta e seus ancestrais

- Estado: corrigido; `a_fresh_index_matches_the_filesystem` em `core/tests/index_verify_test.rs`.
- Local: `IndexVerifyAction::compare_indexes`, ação `indexing.verify`.
- Reprodução anterior: logo depois de indexar uma location, a verificação retornava `is_valid: false` com `/`, `/tmp`, a pasta temporária e a raiz da location como "File exists on filesystem but not in index". O índice efêmero guarda a pasta verificada e cada ancestral até `/`; só a raiz era removida do lado do banco.
- Correção: os dois lados são limitados à pasta verificada e ao que fica abaixo dela. A pasta continua na comparação; a data de modificação dela vem do disco, porque a varredura a registra só como ancestral, sem metadados. O tamanho é comparado só para arquivos: o sistema de arquivos dá 0 para pastas, e a raiz da location guarda o tamanho do conteúdo.

## Core: arquivos por tag falhavam para tag desconhecida só com `include_children`

- Estado: corrigido; `files_by_tag_follow_child_tags_and_confidence`.
- Local: `core/src/ops/tags/files_by_tag.rs`, `tags.files_by_tag`.
- Reprodução anterior: para um UUID sem tag, a consulta devolvia lista vazia com `include_children: false` e o erro `Failed to get descendants: Tag not found` com `true`.
- Correção: tag inexistente devolve lista vazia nos dois casos.

## Core: exclusões e reordenação de Spaces não chegavam aos outros dispositivos

- Estado: corrigido; o caminho local está coberto por `groups_and_items_are_reordered_renamed_and_deleted` em `core/tests/space_layout_test.rs`.
- Local: `spaces.delete`, `spaces.delete_group`, `spaces.delete_item`, `spaces.reorder_groups` e `spaces.reorder_items` em `core/src/ops/spaces/`.
- Reprodução anterior: criar e editar espaços, grupos e itens chamava `library.sync_model`, mas apagar e reordenar não. Os três modelos já aplicam `ChangeType::Delete` recebido; a exclusão local só não era enviada, então o espaço, grupo ou item continuava nos outros dispositivos. A nova ordem também não era enviada, e a reordenação não emitia eventos de recurso para atualizar outras janelas.
- Correção: as três exclusões registram `ChangeType::Delete` antes de apagar a linha, então uma falha no registro não deixa uma exclusão que nenhum dispositivo recebe. A reordenação confere todas as linhas antes, grava a nova ordem numa transação, envia cada linha atualizada e emite os eventos de `space_group` ou `space_item`.
- Limite: o teste lê o log de mudanças que seria enviado (`peer_log`), sem um segundo dispositivo aplicando as mudanças.

## Core: grupos e itens de Space iam para outros dispositivos com o ID local do pai

- Estado: corrigido; `groups_and_items_are_reordered_renamed_and_deleted` lê o `peer_log` e exige `space_uuid` (e `group_uuid` nos itens) em cada inserção e atualização. Com o código anterior, o teste falha em `space_group Insert without space_uuid` com `"space_id":2` no payload. Apontado na revisão do PR.
- Local: `spaces.add_group`, `spaces.add_item`, `spaces.update_group` e a reordenação.
- Reprodução anterior: as ações chamavam `library.sync_model`, que não converte chaves estrangeiras. `space_group` e `space_item` declaram `space_id` e `group_id` em `foreign_key_mappings`, mas o payload saía com os inteiros locais. Quem recebe trata um inteiro sem o campo `_uuid` como já resolvido, então o grupo ou item podia ser gravado sob o espaço ou grupo que tivesse o mesmo número no outro banco.
- Correção: essas ações usam `sync_model_with_db`, que troca os inteiros pelos UUIDs antes do envio.

## Core: item de Space podia ser gravado em grupo de outro espaço

- Estado: corrigido; `groups_and_items_are_reordered_renamed_and_deleted`.
- Local: `spaces.add_item`, `spaces.reorder_groups` e `spaces.reorder_items`.
- Reprodução anterior: `add_item` conferia que o espaço e o grupo existiam, mas não que o grupo pertencia ao espaço; o item ficava com `space_id` de um espaço e `group_id` de outro e aparecia no grupo do segundo. `reorder_groups` aceitava grupos de outro espaço, e `reorder_items` aceitava itens de outro grupo.
- Correção: o grupo precisa pertencer ao espaço informado, e cada item reordenado precisa estar no grupo informado. Sem `group_id`, todos os itens precisam estar no nível do mesmo espaço; a primeira versão desta correção ainda aceitava itens de espaços diferentes, o que a revisão do PR apontou. O erro é o mesmo "not found" de um ID inexistente.

## Core: item fixado no Space resolvia o arquivo errado

- Estado: corrigido; `a_space_layout_lists_groups_items_and_pinned_files`.
- Local: `resolve_wing_path_to_entry_uuid` em `core/src/ops/spaces/add_item/action.rs`.
- Reprodução anterior: o caminho fixado era procurado pelo nome e pela extensão em todo o banco e conferido pelo pai. A pasta `notes` podia resolver para `notes.txt` da mesma pasta, e uma pasta `v1.2` era procurada como arquivo `v1` de extensão `2`, ficando sem arquivo resolvido no layout.
- Correção: o item usa `PathResolver::resolve_to_entry`, o mesmo resolvedor das consultas de arquivo.

## Core: criar uma sincronização de pastas falhava sempre

- Estado: corrigido; `file_sync_test` passou de 1 para 10 casos aprovados, e a migração tem o teste `conduits_reference_entries_and_keep_their_rows`.
- Local: `core/src/infra/db/migration/m20251015_000002_create_sync_tables.rs`, nova `m20261111_000001_fix_sync_conduit_entry_fk.rs`.
- Reprodução anterior: as duas chaves estrangeiras de `sync_conduit` apontavam para a tabela `entry`, que não existe (a tabela é `entries`). O SQLite verifica a tabela pai a cada inserção, então `ConduitManager::create_conduit` falhava com `no such table: main.entry`.
- Correção: a migração original passa a declarar `entries` para bancos novos. A nova migração recria `sync_conduit` e `sync_generation` com as chaves corretas nos bancos existentes, copia as linhas cujas entradas ainda existem e recria os índices, tudo numa transação: uma falha no meio deixa as tabelas originais, e a migração roda de novo na próxima abertura (teste `a_failed_rebuild_keeps_the_original_tables`).
- Limite: o serviço de sincronização de pastas ainda não tem ação nem interface (FSYNC-004 em To Do); o teste `test_mirror_sync_with_files` agora espera o monitor terminar em vez de dormir 100 ms.

## Desktop: arrastar no Linux e no Windows deixava uma sessão de arraste presa

- Estado: corrigido; `drag::tests::begin_drag_off_macos_fails_without_starting_a_session`.
- Local: `begin_drag` em `apps/tauri/src-tauri/src/drag/commands.rs`.
- Reprodução anterior: fora do macOS, o comando registrava a sessão no `DragCoordinator` e emitia `drag:began` para todas as janelas antes de responder que o arraste só existe no macOS. O círculo da visão de tamanhos (`SizeCircle.tsx`) chama `platform.startDrag` em qualquer plataforma.
- Impacto: as janelas recebiam o início de um arraste que nunca acontecia e nunca recebiam `drag:ended`. Por 30 segundos, até a sessão ser considerada antiga, cada nova tentativa falhava com "A drag operation is already in progress".
- Correção: fora do macOS o comando responde com o erro sem registrar sessão nem emitir eventos.

## Desktop: o frontend não conseguia encerrar nem reconhecer o fim de um arraste

- Estado: corrigido; `drag::tests::frontend_drag_config_and_results_keep_their_wire_format` e os testes de `drag:ended`.
- Local: `DragResult` em `apps/tauri/src-tauri/src/drag/events.rs`.
- Reprodução anterior: `#[serde(tag = "type", rename_all = "camelCase")]` gravava as variantes como `dropped`, `cancelled` e `failed`. O frontend envia `{ type: "Cancelled" }` em `useDragOperation.ts` e compara `event.result.type === "Dropped"` em `useDropZone.ts`. O comando `end_drag` recusava o argumento com "unknown variant `Cancelled`".
- Impacto: no macOS, cancelar pelo frontend não encerrava a sessão, que bloqueava novos arrastes por 30 segundos, e `useDropZone` nunca reconhecia um `drag:ended` como soltura.
- Correção: o enum mantém os nomes das variantes em Pascal case, como o tipo TypeScript declara. Os campos (`operation`, `target`, `error`) não mudam.
