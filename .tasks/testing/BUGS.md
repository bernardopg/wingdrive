# Bugs encontrados na ampliação de cobertura

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
