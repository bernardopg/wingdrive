---
title: A CLI wing
description: Controle o daemon do WingDrive pelo terminal e por scripts.
order: 9
group: Avançado
---

`wing` é o cliente de linha de comando para o daemon do WingDrive. Ele se comunica com o mesmo daemon que o aplicativo desktop, então um script e uma janela sempre veem a mesma biblioteca.

## Daemon

```bash
wing start               # inicia o daemon em segundo plano
wing start --foreground  # executa em primeiro plano com logs
wing status              # status do daemon, da biblioteca e da rede
wing stop
wing restart
wing logs                # exibe e acompanha os logs
```

## Bibliotecas e localizações

```bash
wing library create "Personal"
wing library list
wing library switch <library>

wing location add ~/Pictures --name "Pictures"
wing location list
wing location rescan <location>
wing location remove <location>
```

## Indexação e pesquisa

```bash
wing index start ~/Projects
wing index verify <location>     # compara o índice com o disco
wing search files "invoice 2026"
```

## Arquivos

```bash
wing file list ~/Downloads
wing file info ./report.pdf
wing file copy ./a.txt ~/Backup/
wing file rename ./old.txt new.txt
wing file mkdir ~/Projects/new
wing file delete ./tmp.log
wing file undo                   # desfaz a última cópia, movimentação ou envio à lixeira
wing file history                # o que ainda pode ser desfeito
```

## Tarefas

```bash
wing job list
wing job monitor                 # progresso em tempo real
wing job pause <job>
wing job resume <job>
wing job cancel <job>
```

## Rede, dispositivos e volumes

```bash
wing network status
wing network pair generate
wing network pair join "<12 words>"
wing network devices
wing network wingdrop send <device-id> ./file
wing volume list
wing cloud                       # configuração interativa da nuvem
```

## Múltiplas instâncias

Execute daemons isolados lado a lado para testes ou para manter bibliotecas separadas:

```bash
wing --instance test start
wing --instance test library create "Test"
wing --instance test stop
```

## Atualizações

```bash
wing update    # verifica se há uma versão mais recente e mostra como instalá-la
```

Todos os comandos têm `--help`.
