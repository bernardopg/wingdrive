---
title: Dispositivos e sincronização
description: Emparelhe seus dispositivos, sincronize a biblioteca entre eles e envie arquivos com o Wingdrop.
order: 6
group: Usando o WingDrive
---

O WingDrive conecta diretamente seus próprios dispositivos. Não há um servidor WingDrive intermediário: as conexões usam Iroh/QUIC, com criptografia de ponta a ponta.

## Emparelhe dois dispositivos

O emparelhamento usa um código de uso único. Um dispositivo o **gera** e o outro **se conecta** com ele.

- **12 palavras** (um mnemônico BIP39): fácil de ler e digitar, funciona na mesma rede local.
- **QR code**: contém as mesmas palavras e o ID do nó do dispositivo, funcionando entre redes e pela internet, com relay como alternativa.

Os códigos são válidos por cinco minutos e nunca são reutilizados.

No aplicativo desktop:

<ol class="steps">
<li>No primeiro dispositivo, abra a Visão geral e clique em <strong>Parear</strong> na barra superior e depois em <strong>Gerar código de pareamento</strong>.</li>
<li>No segundo, clique em <strong>Parear → Conectar com código</strong> e leia o QR code ou digite as 12 palavras. Em outra rede, cole também o Node ID mostrado em <em>pareamento entre redes</em>.</li>
<li>Os dois aparecem em <em>Dispositivos</em> na barra lateral. Depois, clique em <strong>Configurar sincronização</strong> para compartilhar a biblioteca.</li>
</ol>

<figure class="shot">
<a href="/img/docs/pair-device.webp"><img src="/img/docs/pair-device.webp" width="620" height="830" loading="lazy" alt="Diálogo de pareamento com QR code, o código de 12 palavras e o Node ID"></a>
<figcaption><strong>Gerar código de pareamento</strong> mostra um QR code, as 12 palavras e o Node ID deste dispositivo.</figcaption>
</figure>

Pela CLI:

```bash
# No primeiro dispositivo
wing network pair generate

# No segundo dispositivo
wing network pair join "brave lion sunset river eagle mountain forest ocean thunder crystal diamond phoenix"

# Acompanhe o progresso
wing network pair status
```

### Segurança

- **Autenticação**: os dispositivos comprovam sua identidade com assinaturas Ed25519.
- **Confidencialidade**: todo o tráfego é criptografado com chaves de sessão.
- **Integridade**: um desafio e resposta comprova que quem se conecta conhece o código.
- **Sigilo futuro**: novas chaves são usadas em cada sessão.

Os dispositivos emparelhados enviam seu nome, tipo, sistema operacional e versão do aplicativo atuais sempre que se conectam, para que um dispositivo renomeado ou reinstalado apareça com a nova descrição.

## Remova um emparelhamento

```bash
wing network devices
wing network revoke <device>
```

A revogação avisa o outro dispositivo, que também remove o emparelhamento. Um dispositivo offline mantém o emparelhamento até que você o remova nele.

## Sincronização da biblioteca

A sincronização da biblioteca compartilha o **índice**, as etiquetas e os metadados entre dispositivos emparelhados, para que cada dispositivo possa navegar e pesquisar o que os outros têm. O conteúdo dos arquivos permanece onde está.

```bash
wing library sync-setup setup --action share-local   # no dispositivo que contém a biblioteca
wing library sync-setup setup --action join-remote   # no dispositivo que se conecta
```

A configuração oferece três ações: `share-local` (compartilhar esta biblioteca), `join-remote` (conectar-se a uma biblioteca do outro dispositivo) e `merge`.

## Wingdrop

O Wingdrop envia arquivos para um dispositivo emparelhado:

```bash
wing network wingdrop send <device-id> ./photo.jpg
```

A sincronização de arquivos entre dispositivos, os links públicos de compartilhamento e o Wingdrop para dispositivos próximos não emparelhados estão planejados para o marco M4.
