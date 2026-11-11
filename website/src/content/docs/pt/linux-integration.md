---
title: Integração com Linux
description: Defina o WingDrive como gerenciador de arquivos padrão, inicie-o com sua sessão e mantenha-o na bandeja.
order: 8
group: Usando o WingDrive
---

O marco atual do WingDrive (M2) é ser seu **gerenciador de arquivos padrão** no Linux. Ele se integra ao desktop da mesma forma que Dolphin ou Nautilus.

## Gerenciador de arquivos padrão

Defina o WingDrive como o aplicativo responsável por pastas:

```bash
xdg-mime default wingdrive.desktop inode/directory
xdg-mime query default inode/directory   # deve exibir wingdrive.desktop
```

Depois:

- `xdg-open ~/Downloads` abre a pasta no WingDrive.
- **Show in folder** em navegadores e outros aplicativos abre o WingDrive com o arquivo selecionado. O WingDrive responde à interface D-Bus `org.freedesktop.FileManager1` enquanto for o aplicativo padrão para `inode/directory`.

Para voltar, associe `inode/directory` ao gerenciador de arquivos anterior, por exemplo, `org.kde.dolphin.desktop` ou `org.gnome.Nautilus.desktop`.

## Segundo plano e bandeja

Em **Settings**, na seção **Background**:

- **Keep running in the background**: fechar a janela mantém o WingDrive na bandeja, para que as pastas abram imediatamente.
- **Start at login**: inicia o WingDrive oculto na bandeja quando você entra na sessão.

## Terminal

F4 acopla um terminal abaixo do Explorer, e Shift+F4 abre seu próprio terminal na pasta atual. Defina o comando em **Settings**. Se estiver vazio, o WingDrive usa `xdg-terminal-exec`, depois `$TERMINAL` e, por fim, um terminal instalado.

## Menus de serviço e Open With

Seus **menus de serviço** do KDE/Dolphin aparecem no menu de contexto. **Open With** lista aplicativos a partir de suas entradas de desktop e passa os arquivos corretamente para entradas que usam códigos de campo como `%f`, `%F`, `%u` e `%U`.

## Wayland

Se a janela não iniciar em uma sessão Wayland devido a um erro do GTK, execute-a pelo XWayland:

```bash
GDK_BACKEND=x11 wingdrive
```
