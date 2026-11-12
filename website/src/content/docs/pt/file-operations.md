---
title: Operações com arquivos e como desfazê-las
description: Copie, mova, compacte, verifique somas e conheça as proteções dos seus dados.
order: 5
group: Usando o WingDrive
---

O WingDrive oferece todas as ferramentas que você espera de um gerenciador de arquivos e cuida dos seus dados.

## Ferramentas

- **Copy To** e **Move To** para qualquer localização, volume ou dispositivo emparelhado.
- **Compactar** e **extrair** arquivos compactados pelo menu de contexto.
- **Modelos**: crie novos arquivos a partir da sua pasta de modelos.
- **Menus de serviço do Dolphin**: seus menus de serviço existentes do KDE aparecem no menu de contexto.
- **Open With**, incluindo entradas de desktop com códigos de campo como `%f` e `%u`.
- **Permissões** e **somas de verificação SHA-256** no Inspector.

## Desfazer

Cópias, substituições, movimentações, renomeações, envios à lixeira e novas pastas, arquivos e links são registrados em um histórico. Pressione **Ctrl+Z** no aplicativo desktop ou na interface web, ou execute `wing file undo`, para desfazer a última operação. `wing file history` lista o que ainda pode ser desfeito.

- Um arquivo substituído por uma cópia é mantido por 7 dias em uma pasta oculta `.wing-undo-*` ao lado dele.
- A operação de desfazer ignora tudo o que foi alterado desde a operação original, para nunca sobrescrever trabalho mais recente.
- O histórico fica no banco de dados de tarefas de cada biblioteca e nunca é sincronizado.

## Proteções

- **Verificação prévia de cópia** verifica espaço livre e conflitos antes de qualquer gravação.
- **Cópias atômicas**: os arquivos são gravados com um nome temporário e renomeados para o destino. Uma cópia cancelada ou com falha não deixa um arquivo truncado nem destrói o arquivo que estava substituindo.
- **Sem cópias dentro de si mesmas**: uma pasta não pode ser copiada para uma subpasta dela própria.
- **Datas e horários** de arquivos e pastas são preservados.
- **Links simbólicos** são copiados, movidos e renomeados como links.
- **Exclusão segura** recusa sistemas de arquivos copy-on-write (btrfs, ZFS, APFS, ReFS, bcachefs), nos quais não é possível garantir a sobrescrita no mesmo local.
- **Montagens do sistema** nunca são ejetadas.

## Tarefas

Operações longas são executadas como **tarefas** que informam o progresso, podem ser pausadas e canceladas e continuam após uma reinicialização. Acompanhe-as no gerenciador de tarefas ou com `wing job monitor`.
