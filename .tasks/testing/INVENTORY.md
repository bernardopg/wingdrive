# Inventário de referência

Artefato `coverage-rust` do [run 41686685198](https://github.com/bernardopg/wingdrive/actions/runs/41686685198),
concluído com sucesso no SHA `1064b6b8b6e94f5b7040e3c112e784d5c0e57271`.
Consultado em 2026-11-03. Estes dados precedem os testes da COV-02.

```bash
gh run download 41686685198 -R bernardopg/wingdrive -n coverage-rust -D /tmp/wingdrive-cov-baseline-41686685198
```

A lista ordena os 20 maiores déficits de linhas de cada componente Rust. É um diagnóstico,
não a ordem de execução: COV-03 prioriza perda de dados antes de módulos de rede extensos.
Arquivo com zero hits foi instrumentado e não executado. Arquivo ausente do LCOV tem cobertura
desconhecida, podendo estar fora do build por plataforma ou feature, ou não conter código executável.
Linhas de teste em módulos inline podem aparecer no LCOV atual; COV-01 deve auditar essa inclusão
antes de congelar o denominador e ativar os pisos da COV-11. As novas suites ficam em diretórios
`tests/`, que o agregador já exclui.

## Core

| Arquivo | Cobertas / instrumentadas | Cobertura | Linhas sem execução |
|---|---|---|---|
| `core/src/service/network/protocol/pairing/mod.rs` | 109/1573 | 6.9% | 1464 |
| `core/src/infra/job/manager.rs` | 366/1666 | 22.0% | 1300 |
| `core/src/ops/tags/manager.rs` | 0/1179 | 0.0% | 1179 |
| `core/src/service/network/protocol/file_transfer.rs` | 33/1160 | 2.8% | 1127 |
| `core/src/service/network/core/mod.rs` | 177/1302 | 13.6% | 1125 |
| `core/src/ops/search/query.rs` | 0/1122 | 0.0% | 1122 |
| `core/src/volume/manager.rs` | 386/1274 | 30.3% | 888 |
| `core/src/ops/files/query/directory_listing.rs` | 0/866 | 0.0% | 866 |
| `core/src/ops/indexing/ephemeral/index.rs` | 21/808 | 2.6% | 787 |
| `core/src/ops/files/copy/strategy.rs` | 0/716 | 0.0% | 716 |
| `core/src/ops/archive/engine.rs` | 0/690 | 0.0% | 690 |
| `core/src/ops/files/copy/job.rs` | 0/681 | 0.0% | 681 |
| `core/src/service/sync/peer.rs` | 939/1593 | 58.9% | 654 |
| `core/src/library/manager.rs` | 690/1327 | 52.0% | 637 |
| `core/src/library/mod.rs` | 439/1060 | 41.4% | 621 |
| `core/src/ops/locations/export/action.rs` | 0/604 | 0.0% | 604 |
| `core/src/service/network/protocol/messaging.rs` | 18/613 | 2.9% | 595 |
| `core/src/service/network/core/event_loop.rs` | 64/648 | 9.9% | 584 |
| `core/src/domain/addressing.rs` | 72/654 | 11.0% | 582 |
| `core/src/service/network/device/registry.rs` | 48/628 | 7.6% | 580 |

Arquivos Rust atuais fora desse LCOV (257). Conferir cfg/features e presença de linhas executáveis:

- `core/src/common/mod.rs`
- `core/src/common/types.rs`
- `core/src/crypto/mod.rs`
- `core/src/data/mod.rs`
- `core/src/device/mod.rs`
- `core/src/domain/memory/mod.rs`
- `core/src/domain/mod.rs`
- `core/src/infra/action/context_test.rs`
- `core/src/infra/action/output.rs`
- `core/src/infra/api/mod.rs`
- `core/src/infra/daemon/mod.rs`
- `core/src/infra/db/entities/cloud_credential.rs`
- `core/src/infra/db/entities/indexer_rule.rs`
- `core/src/infra/db/entities/mod.rs`
- `core/src/infra/extension/mod.rs`
- `core/src/infra/fs/mod.rs`
- `core/src/infra/job/mod.rs`
- `core/src/infra/mod.rs`
- `core/src/infra/sync/event_log/mod.rs`
- `core/src/infra/sync/mod.rs`
- `core/src/infra/wire/mod.rs`
- `core/src/library/error.rs`
- `core/src/ops/adapters/config/mod.rs`
- `core/src/ops/adapters/list/mod.rs`
- `core/src/ops/adapters/mod.rs`
- `core/src/ops/adapters/update/mod.rs`
- `core/src/ops/archive/mod.rs`
- `core/src/ops/config/app/mod.rs`
- `core/src/ops/config/library/mod.rs`
- `core/src/ops/config/mod.rs`
- `core/src/ops/core/ephemeral_status/mod.rs`
- `core/src/ops/core/ephemeral_status/output.rs`
- `core/src/ops/core/events/mod.rs`
- `core/src/ops/core/mod.rs`
- `core/src/ops/core/reset/input.rs`
- `core/src/ops/core/reset/mod.rs`
- `core/src/ops/core/reset/output.rs`
- `core/src/ops/core/status/mod.rs`
- `core/src/ops/core/status/output.rs`
- `core/src/ops/devices/list/mod.rs`
- `core/src/ops/devices/list/output.rs`
- `core/src/ops/devices/mod.rs`
- `core/src/ops/extension_test/mod.rs`
- `core/src/ops/files/copy/mod.rs`
- `core/src/ops/files/create_folder/mod.rs`
- `core/src/ops/files/delete/mod.rs`
- `core/src/ops/files/duplicate_detection/input.rs`
- `core/src/ops/files/duplicate_detection/mod.rs`
- `core/src/ops/files/mod.rs`
- `core/src/ops/files/query/mod.rs`
- `core/src/ops/files/rename/mod.rs`
- `core/src/ops/indexing/change_detection/mod.rs`
- `core/src/ops/indexing/ephemeral/mod.rs`
- `core/src/ops/indexing/handlers/mod.rs`
- `core/src/ops/indexing/mod.rs`
- `core/src/ops/indexing/phases/mod.rs`
- `core/src/ops/indexing/verify/mod.rs`
- `core/src/ops/jobs/active/mod.rs`
- `core/src/ops/jobs/active/output.rs`
- `core/src/ops/jobs/control/mod.rs`
- `core/src/ops/jobs/copy_metadata/mod.rs`
- `core/src/ops/jobs/info/mod.rs`
- `core/src/ops/jobs/info/output.rs`
- `core/src/ops/jobs/list/mod.rs`
- `core/src/ops/jobs/list/output.rs`
- `core/src/ops/jobs/mod.rs`
- `core/src/ops/jobs/remote_list/mod.rs`
- `core/src/ops/jobs/remote_list/output.rs`
- `core/src/ops/libraries/create/mod.rs`
- `core/src/ops/libraries/delete/mod.rs`
- `core/src/ops/libraries/export/input.rs`
- `core/src/ops/libraries/export/mod.rs`
- `core/src/ops/libraries/info/mod.rs`
- `core/src/ops/libraries/info/output.rs`
- `core/src/ops/libraries/list/mod.rs`
- `core/src/ops/libraries/mod.rs`
- `core/src/ops/libraries/open/input.rs`
- `core/src/ops/libraries/open/mod.rs`
- `core/src/ops/libraries/rename/mod.rs`
- `core/src/ops/locations/add/mod.rs`
- `core/src/ops/locations/enable_indexing/mod.rs`
- `core/src/ops/locations/export/mod.rs`
- `core/src/ops/locations/import/input.rs`
- `core/src/ops/locations/import/mod.rs`
- `core/src/ops/locations/list/mod.rs`
- `core/src/ops/locations/list/output.rs`
- `core/src/ops/locations/mod.rs`
- `core/src/ops/locations/remove/mod.rs`
- `core/src/ops/locations/rescan/mod.rs`
- `core/src/ops/locations/suggested/mod.rs`
- `core/src/ops/locations/suggested/output.rs`
- `core/src/ops/locations/trigger_job/mod.rs`
- `core/src/ops/locations/trigger_job/output.rs`
- `core/src/ops/locations/update/mod.rs`
- `core/src/ops/locations/update/output.rs`
- `core/src/ops/locations/validate/mod.rs`
- `core/src/ops/locations/validate/output.rs`
- `core/src/ops/media/mod.rs`
- `core/src/ops/media/speech/job.rs`
- `core/src/ops/media/speech/processor.rs`
- `core/src/ops/media/thumbstrip/config.rs`
- `core/src/ops/media/thumbstrip/error.rs`
- `core/src/ops/media/thumbstrip/generator.rs`
- `core/src/ops/media/thumbstrip/job.rs`
- `core/src/ops/media/thumbstrip/processor.rs`
- `core/src/ops/media/thumbstrip/state.rs`
- `core/src/ops/metadata/favorite/input.rs`
- `core/src/ops/metadata/favorite/mod.rs`
- `core/src/ops/metadata/favorite/output.rs`
- `core/src/ops/metadata/mod.rs`
- `core/src/ops/mod.rs`
- `core/src/ops/models/types.rs`
- `core/src/ops/network/devices/mod.rs`
- `core/src/ops/network/devices/output.rs`
- `core/src/ops/network/mod.rs`
- `core/src/ops/network/pair/cancel/input.rs`
- `core/src/ops/network/pair/cancel/mod.rs`
- `core/src/ops/network/pair/cancel/output.rs`
- `core/src/ops/network/pair/confirm_proxy/input.rs`
- `core/src/ops/network/pair/confirm_proxy/mod.rs`
- `core/src/ops/network/pair/confirm_proxy/output.rs`
- `core/src/ops/network/pair/generate/input.rs`
- `core/src/ops/network/pair/generate/mod.rs`
- `core/src/ops/network/pair/generate/output.rs`
- `core/src/ops/network/pair/join/input.rs`
- `core/src/ops/network/pair/join/mod.rs`
- `core/src/ops/network/pair/join/output.rs`
- `core/src/ops/network/pair/mod.rs`
- `core/src/ops/network/pair/status/mod.rs`
- `core/src/ops/network/pair/vouch/input.rs`
- `core/src/ops/network/pair/vouch/mod.rs`
- `core/src/ops/network/pair/vouch/output.rs`
- `core/src/ops/network/pair/vouching_session/input.rs`
- `core/src/ops/network/pair/vouching_session/mod.rs`
- `core/src/ops/network/pair/vouching_session/output.rs`
- `core/src/ops/network/revoke/input.rs`
- `core/src/ops/network/revoke/mod.rs`
- `core/src/ops/network/revoke/output.rs`
- `core/src/ops/network/start/input.rs`
- `core/src/ops/network/start/mod.rs`
- `core/src/ops/network/start/output.rs`
- `core/src/ops/network/status/mod.rs`
- `core/src/ops/network/status/output.rs`
- `core/src/ops/network/stop/input.rs`
- `core/src/ops/network/stop/mod.rs`
- `core/src/ops/network/stop/output.rs`
- `core/src/ops/network/sync_setup/discovery/mod.rs`
- `core/src/ops/network/sync_setup/discovery/output.rs`
- `core/src/ops/network/sync_setup/input.rs`
- `core/src/ops/network/sync_setup/mod.rs`
- `core/src/ops/network/sync_setup/output.rs`
- `core/src/ops/network/wingdrop/mod.rs`
- `core/src/ops/redundancy/mod.rs`
- `core/src/ops/redundancy/summary/input.rs`
- `core/src/ops/redundancy/summary/mod.rs`
- `core/src/ops/redundancy/summary/output.rs`
- `core/src/ops/search/mod.rs`
- `core/src/ops/search/tests.rs`
- `core/src/ops/sidecar/mod.rs`
- `core/src/ops/sources/create/input.rs`
- `core/src/ops/sources/create/mod.rs`
- `core/src/ops/sources/delete/mod.rs`
- `core/src/ops/sources/get/mod.rs`
- `core/src/ops/sources/list/mod.rs`
- `core/src/ops/sources/list_items/mod.rs`
- `core/src/ops/sources/mod.rs`
- `core/src/ops/sources/sync/mod.rs`
- `core/src/ops/spaces/add_group/input.rs`
- `core/src/ops/spaces/add_group/mod.rs`
- `core/src/ops/spaces/add_group/output.rs`
- `core/src/ops/spaces/add_item/input.rs`
- `core/src/ops/spaces/add_item/mod.rs`
- `core/src/ops/spaces/add_item/output.rs`
- `core/src/ops/spaces/create/input.rs`
- `core/src/ops/spaces/create/mod.rs`
- `core/src/ops/spaces/create/output.rs`
- `core/src/ops/spaces/delete/input.rs`
- `core/src/ops/spaces/delete/mod.rs`
- `core/src/ops/spaces/delete/output.rs`
- `core/src/ops/spaces/delete_group/input.rs`
- `core/src/ops/spaces/delete_group/mod.rs`
- `core/src/ops/spaces/delete_group/output.rs`
- `core/src/ops/spaces/delete_item/input.rs`
- `core/src/ops/spaces/delete_item/mod.rs`
- `core/src/ops/spaces/delete_item/output.rs`
- `core/src/ops/spaces/get/mod.rs`
- `core/src/ops/spaces/get/output.rs`
- `core/src/ops/spaces/get_layout/mod.rs`
- `core/src/ops/spaces/get_layout/output.rs`
- `core/src/ops/spaces/list/mod.rs`
- `core/src/ops/spaces/list/output.rs`
- `core/src/ops/spaces/mod.rs`
- `core/src/ops/spaces/reorder/input.rs`
- `core/src/ops/spaces/reorder/mod.rs`
- `core/src/ops/spaces/reorder/output.rs`
- `core/src/ops/spaces/update/input.rs`
- `core/src/ops/spaces/update/mod.rs`
- `core/src/ops/spaces/update/output.rs`
- `core/src/ops/spaces/update_group/input.rs`
- `core/src/ops/spaces/update_group/mod.rs`
- `core/src/ops/spaces/update_group/output.rs`
- `core/src/ops/sync/get_activity/input.rs`
- `core/src/ops/sync/get_activity/mod.rs`
- `core/src/ops/sync/get_activity/output.rs`
- `core/src/ops/sync/get_event_log/input.rs`
- `core/src/ops/sync/get_event_log/mod.rs`
- `core/src/ops/sync/get_event_log/output.rs`
- `core/src/ops/sync/get_metrics/input.rs`
- `core/src/ops/sync/get_metrics/mod.rs`
- `core/src/ops/sync/get_metrics/output.rs`
- `core/src/ops/sync/get_sync_partners/input.rs`
- `core/src/ops/sync/get_sync_partners/mod.rs`
- `core/src/ops/sync/get_sync_partners/output.rs`
- `core/src/ops/sync/mod.rs`
- `core/src/ops/tags/apply/mod.rs`
- `core/src/ops/tags/create/mod.rs`
- `core/src/ops/tags/delete/mod.rs`
- `core/src/ops/tags/delete/output.rs`
- `core/src/ops/tags/mod.rs`
- `core/src/ops/tags/search/mod.rs`
- `core/src/ops/tags/unapply/mod.rs`
- `core/src/ops/tags/unapply/output.rs`
- `core/src/ops/tags/validation/mod.rs`
- `core/src/ops/volumes/add_cloud/mod.rs`
- `core/src/ops/volumes/eject/input.rs`
- `core/src/ops/volumes/eject/mod.rs`
- `core/src/ops/volumes/eject/output.rs`
- `core/src/ops/volumes/index/output.rs`
- `core/src/ops/volumes/list/mod.rs`
- `core/src/ops/volumes/list/output.rs`
- `core/src/ops/volumes/mod.rs`
- `core/src/ops/volumes/refresh/mod.rs`
- `core/src/ops/volumes/remove_cloud/mod.rs`
- `core/src/ops/volumes/speed_test/mod.rs`
- `core/src/ops/volumes/track/input.rs`
- `core/src/ops/volumes/track/mod.rs`
- `core/src/ops/volumes/track/output.rs`
- `core/src/ops/volumes/untrack/input.rs`
- `core/src/ops/volumes/untrack/mod.rs`
- `core/src/ops/volumes/untrack/output.rs`
- `core/src/service/network/protocol/library_messages.rs`
- `core/src/service/network/protocol/mod.rs`
- `core/src/service/network/protocol/pairing/messages.rs`
- `core/src/service/network/protocol/pairing/proxy.rs`
- `core/src/service/network/protocol/sync/mod.rs`
- `core/src/service/network/transports/mod.rs`
- `core/src/service/network/utils/mod.rs`
- `core/src/service/sync/metrics/mod.rs`
- `core/src/service/watcher/mod.rs`
- `core/src/testing/mod.rs`
- `core/src/volume/fs/ntfs.rs`
- `core/src/volume/fs/refs.rs`
- `core/src/volume/platform/ios.rs`
- `core/src/volume/platform/macos.rs`
- `core/src/volume/platform/mod.rs`
- `core/src/volume/platform/windows.rs`
- `core/src/volume/types.rs`

## Desktop

| Arquivo | Cobertas / instrumentadas | Cobertura | Linhas sem execução |
|---|---|---|---|
| `apps/tauri/src-tauri/src/main.rs` | 132/1450 | 9.1% | 1318 |
| `apps/tauri/src-tauri/src/windows.rs` | 19/360 | 5.3% | 341 |
| `apps/tauri/src-tauri/src/server.rs` | 259/529 | 49.0% | 270 |
| `apps/tauri/src-tauri/src/files.rs` | 26/153 | 17.0% | 127 |
| `apps/tauri/crates/file-opening-linux/src/lib.rs` | 0/114 | 0.0% | 114 |
| `apps/tauri/src-tauri/src/terminal.rs` | 343/450 | 76.2% | 107 |
| `apps/tauri/src-tauri/src/gvfs.rs` | 149/253 | 58.9% | 104 |
| `apps/tauri/src-tauri/src/drag/mod.rs` | 0/98 | 0.0% | 98 |
| `apps/tauri/src-tauri/src/background.rs` | 93/190 | 48.9% | 97 |
| `apps/tauri/src-tauri/src/file_opening.rs` | 0/89 | 0.0% | 89 |
| `apps/tauri/src-tauri/src/clipboard.rs` | 34/108 | 31.5% | 74 |
| `apps/tauri/crates/file-opening-linux/src/mime_apps.rs` | 289/351 | 82.3% | 62 |
| `apps/tauri/crates/file-opening-linux/src/icon_theme.rs` | 182/240 | 75.8% | 58 |
| `apps/tauri/src-tauri/src/launch.rs` | 131/186 | 70.4% | 55 |
| `apps/tauri/src-tauri/src/file_drag.rs` | 0/54 | 0.0% | 54 |
| `apps/tauri/src-tauri/src/filemanager1.rs` | 0/50 | 0.0% | 50 |
| `apps/tauri/src-tauri/src/keybinds.rs` | 0/46 | 0.0% | 46 |
| `apps/tauri/src-tauri/src/drag/commands.rs` | 0/39 | 0.0% | 39 |
| `apps/tauri/src-tauri/src/admin.rs` | 199/233 | 85.4% | 34 |
| `apps/tauri/crates/file-opening-linux/src/desktop_entry.rs` | 220/247 | 89.1% | 27 |

Arquivos Rust atuais fora desse LCOV (8). Conferir cfg/features e presença de linhas executáveis:

- `apps/tauri/crates/file-opening-macos/build.rs`
- `apps/tauri/crates/file-opening-macos/src/lib.rs`
- `apps/tauri/crates/file-opening-windows/src/lib.rs`
- `apps/tauri/crates/macos/build.rs`
- `apps/tauri/crates/macos/src/lib.rs`
- `apps/tauri/src-tauri/build.rs`
- `apps/tauri/src-tauri/src/drag/events.rs`
- `apps/tauri/wing-tauri-core/src/lib.rs`

## CLI

| Arquivo | Cobertas / instrumentadas | Cobertura | Linhas sem execução |
|---|---|---|---|
| `apps/cli/src/main.rs` | 0/516 | 0.0% | 516 |
| `apps/cli/src/domains/sync/mod.rs` | 0/490 | 0.0% | 490 |
| `apps/cli/src/domains/index/mod.rs` | 0/326 | 0.0% | 326 |
| `apps/cli/src/domains/cloud/setup.rs` | 0/322 | 0.0% | 322 |
| `apps/cli/src/domains/job/mod.rs` | 0/268 | 0.0% | 268 |
| `apps/cli/src/domains/library/mod.rs` | 0/257 | 0.0% | 257 |
| `apps/cli/src/domains/events/mod.rs` | 0/225 | 0.0% | 225 |
| `apps/cli/src/domains/redundancy/mod.rs` | 0/212 | 0.0% | 212 |
| `apps/cli/src/domains/logs/mod.rs` | 0/200 | 0.0% | 200 |
| `apps/cli/src/domains/network/mod.rs` | 0/197 | 0.0% | 197 |
| `apps/cli/src/domains/file/mod.rs` | 0/190 | 0.0% | 190 |
| `apps/cli/src/domains/daemon/mod.rs` | 13/192 | 6.8% | 179 |
| `apps/cli/src/domains/update/mod.rs` | 0/169 | 0.0% | 169 |
| `apps/cli/src/ui/progress.rs` | 0/167 | 0.0% | 167 |
| `apps/cli/src/domains/location/mod.rs` | 0/141 | 0.0% | 141 |
| `apps/cli/src/context.rs` | 0/141 | 0.0% | 141 |
| `apps/cli/src/domains/volume/args.rs` | 0/128 | 0.0% | 128 |
| `apps/cli/src/util/confirm.rs` | 0/119 | 0.0% | 119 |
| `apps/cli/src/domains/library/args.rs` | 0/92 | 0.0% | 92 |
| `apps/cli/src/domains/search/args.rs` | 0/91 | 0.0% | 91 |

Arquivos Rust atuais fora desse LCOV (7). Conferir cfg/features e presença de linhas executáveis:

- `apps/cli/src/domains/events/args.rs`
- `apps/cli/src/domains/logs/args.rs`
- `apps/cli/src/domains/mod.rs`
- `apps/cli/src/domains/sync/args.rs`
- `apps/cli/src/ui/mod.rs`
- `apps/cli/src/util/mod.rs`
- `apps/cli/src/util/prelude.rs`

## Próxima medição

Reexecutar os runners completos do [plano](README.md) no branch após a COV-02 e comparar os
mesmos arquivos e exclusões. Não deduzir ganho de porcentagem de um teste focado ou da quantidade
de casos. O inventário frontend exige também o artefato `coverage-frontend`, pois React e parte
do shell TypeScript não aparecem neste artefato Rust. Essa análise e a reprodução local completa
continuam abertas na COV-01.
