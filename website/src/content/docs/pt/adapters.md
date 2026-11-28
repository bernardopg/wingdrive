---
title: Adaptadores
description: Traga notas, código e dados de outros apps para a sua biblioteca pela loja de adaptadores, e mantenha tudo sincronizado.
order: 7.5
group: Usando o WingDrive
---

Um **adaptador** traz dados de outro app ou serviço para a sua biblioteca como uma **fonte**: as notas de um cofre do Obsidian, seus repositórios, issues e pull requests do GitHub, e mais com o tempo. Os itens de uma fonte aparecem na busca junto com os seus arquivos, e cada sincronização lê só o que mudou.

Você instala adaptadores pela **loja de adaptadores** dentro do WingDrive. O catálogo também está em [wingdrive.org/pt/adapters](/pt/adapters/).

## Instalar um adaptador

<ol class="steps">
<li>Abra <strong>Fontes</strong> na barra lateral e clique em <strong>+</strong> (<em>Adicionar fonte</em>). A loja abre com um cartão para cada adaptador publicado.</li>
<li>Clique num cartão para ver o que o adaptador faz e <strong>o que ele pode acessar</strong>: os arquivos e pastas que lê, os servidores que acessa, os segredos que pede e os programas que executa.</li>
<li>Clique em <strong>Instalar</strong>. Ao instalar, você permite esse acesso.</li>
</ol>

<figure class="shot">
<a href="/img/adapters/store-pt.webp"><img src="/img/adapters/store-pt.webp" width="795" height="300" loading="lazy" alt="A loja de adaptadores com os cartões do GitHub e do Obsidian, cada um com o selo Oficial e um resumo de duas linhas"></a>
<figcaption>A loja: um cartão por adaptador, com quem o fez e o que ele faz.</figcaption>
</figure>

<figure class="shot">
<a href="/img/adapters/details-pt.webp"><img src="/img/adapters/details-pt.webp" width="540" height="510" loading="lazy" alt="Detalhes do Obsidian: os caminhos que ele lê, sem rede, e um aviso verde dizendo que o acesso é aplicado neste computador"></a>
<figcaption>Antes de instalar, os detalhes listam tudo o que o adaptador alcança.</figcaption>
</figure>

O selo **Oficial** marca adaptadores mantidos pela equipe do WingDrive. **Verificado** marca autores que já têm adaptadores revisados publicados. Todo adaptador, oficial ou não, é revisado antes de chegar à loja.

Pela linha de comando:

```bash
wing adapters browse            # baixa o índice mais novo e lista todos os adaptadores
wing adapters install obsidian  # mostra as permissões e pergunta antes de instalar
```

## Adicionar uma fonte

<ol class="steps">
<li>Nos detalhes do adaptador, clique em <strong>Adicionar fonte</strong>.</li>
<li>Preencha o formulário. Muitos adaptadores encontram seus dados sozinhos: o do Obsidian preenche o cofre que você abriu por último e oferece os outros, e o do GitHub preenche o usuário com que o GitHub CLI está conectado. Campos de pasta têm um seletor de pasta no app desktop.</li>
<li>Clique em <strong>Adicionar e sincronizar</strong>. A primeira sincronização roda em segundo plano; acompanhe no gerenciador de tarefas.</li>
</ol>

<figure class="shot">
<a href="/img/adapters/form-pt.webp"><img src="/img/adapters/form-pt.webp" width="540" height="470" loading="lazy" alt="O formulário de nova fonte com o caminho do cofre /home/ana/Notes preenchido e o aviso Encontrado neste computador"></a>
<figcaption>O cofre foi encontrado neste computador e já veio preenchido.</figcaption>
</figure>

A fonte aparece então em **Fontes**. Abra para navegar pelos itens, buscar dentro deles, sincronizar de novo ou apagar a fonte.

Para mudar uma fonte depois, abra-a e escolha **Editar fonte** no menu dela. Um token salvo continua salvo a menos que você digite um novo. O WingDrive guarda tokens e senhas no seu cofre cifrado, nunca em texto puro junto com as outras configurações.

<figure class="shot">
<a href="/img/adapters/items-pt.webp"><img src="/img/adapters/items-pt.webp" width="795" height="485" loading="lazy" alt="A fonte Notes com seis notas, cada uma com título, caminho e o começo do texto"></a>
<figcaption>Um cofre do Obsidian sincronizado: cada nota com o caminho e o começo do texto.</figcaption>
</figure>

## Buscar nas suas fontes

A busca com o escopo **Biblioteca** encontra os itens das suas fontes junto com os seus arquivos. No Explorador, digite na caixa de busca e escolha **Biblioteca** em *Pesquisar em*. Os resultados das fontes aparecem acima dos arquivos, agrupados por fonte, com as palavras buscadas destacadas. *Esta pasta* e *Local* buscam só arquivos.

<figure class="shot">
<a href="/img/adapters/search-pt.webp"><img src="/img/adapters/search-pt.webp" width="795" height="354" loading="lazy" alt="Uma busca por harness na biblioteca: resultados de uma fonte do GitHub e de um cofre do Obsidian acima de quatro arquivos"></a>
<figcaption>A busca na biblioteca encontra issues do GitHub e notas do Obsidian junto com os arquivos.</figcaption>
</figure>

Clique em um resultado para abrir a fonte com o item aberto ao lado da lista: a data, o ID no serviço de origem e o texto inteiro. **Abrir fonte** mostra todos os resultados daquela fonte. A caixa de busca da página de uma fonte busca em todos os itens dela, não só nos que estão na tela.

<figure class="shot">
<a href="/img/adapters/item-open-pt.webp"><img src="/img/adapters/item-open-pt.webp" width="795" height="230" loading="lazy" alt="Uma issue do GitHub aberta pela busca, com a data, o ID issue:1101 e o texto"></a>
<figcaption>Um item aberto pela busca.</figcaption>
</figure>

Todas as palavras digitadas precisam aparecer, e o começo de uma palavra basta: `tomat` encontra *tomatoes*. Itens que a verificação de segurança pôs em quarentena nunca aparecem.

## O que os adaptadores podem acessar

Cada adaptador declara do que precisa, e a loja mostra isso antes de instalar. O quanto o WingDrive prende o adaptador a essa lista depende do sistema:

| Sistema | O que vale |
| --- | --- |
| **Linux** (AppImage, `.deb`, AUR, arquivos do servidor) | O adaptador roda **isolado** com o [bubblewrap](https://github.com/containers/bubblewrap). Ele só enxerga os caminhos que declara e uma pasta privada dele, e não tem rede a não ser que declare servidores. A loja diz *Aplicado neste computador*. |
| **Linux Flatpak** | Vale o sandbox do Flatpak do WingDrive, mas o adaptador não fica limitado aos caminhos que declara. |
| **macOS, Windows** | Os adaptadores ainda não rodam isolados. A lista é o que o registro revisou. |

Todo adaptador também para no limite de tempo e em 2 GB de memória. Os adaptadores nunca recebem as configurações nem os tokens do próprio WingDrive; um token que você digita no formulário de uma fonte vai só para aquele adaptador.

## Atualizações

O WingDrive consulta a loja 30 segundos depois de abrir e a cada seis horas. Quando um adaptador instalado tem atualização, **Fontes** na barra lateral mostra *1 atualização de adaptador*, o botão **+** ganha um contador e a aba **Instalados** da loja conta a atualização. Abra o adaptador e clique em **Atualizar para …**.

- Uma atualização que pede **mais acesso** que a versão instalada avisa e pergunta de novo.
- Se o registro **revoga** uma versão (por exemplo, porque vazava dados), o WingDrive para de rodá-la na hora e a loja explica o motivo. Atualize ou remova.

```bash
wing adapters outdated       # adaptadores instalados com atualização ou versão revogada
wing adapters update --all   # pergunta de novo quando a atualização pede mais acesso
```

Versões novas do próprio WingDrive aparecem em **Configurações → Geral → Atualizações**, com uma notificação na área de trabalho por versão. O AppImage e os instaladores do Windows se atualizam sozinhos; instalações pelo AUR, `.deb` e outros gerenciadores de pacotes dizem para atualizar pelo seu gerenciador.

## Remover um adaptador

Abra o adaptador na loja e clique em **Remover**, ou rode `wing adapters remove <id>`. Antes, apague as fontes dele se não precisar mais dos itens: uma fonte cujo adaptador sumiu guarda os itens, mas não sincroniza.

## Solução de problemas

| Mensagem | O que fazer |
| --- | --- |
| *Os adaptadores rodam isolados com o bubblewrap, que falta ou está bloqueado aqui* | Instale o pacote `bubblewrap` (`sudo pacman -S bubblewrap`, `sudo apt install bubblewrap`). No Docker, rode o contêiner com `--security-opt seccomp=unconfined`, ou defina `WINGDRIVE_ADAPTER_SANDBOX=off` para rodar adaptadores sem isolamento. |
| *Este adaptador precisa do Python 3* | Instale o Python 3.9 ou mais novo (`python3` no Linux e no macOS, `python` no Windows) e abra a loja de novo. |
| *Não é para este sistema* | O autor do adaptador não o publicou para o seu sistema operacional. |
| Uma sincronização termina com erro | A página da fonte mostra a mensagem do próprio adaptador, como *vault_path is not a folder* ou *GitHub rate limit reached*. Corrija o campo ou espere, e clique em **Sincronizar**. |

## Publique o seu adaptador

Adaptadores são programas pequenos, em Python ou em qualquer linguagem compilada, que leem um documento JSON e imprimem linhas JSON. Leia o [guia de publicação](https://github.com/bernardopg/wingdrive-adapters/blob/main/CONTRIBUTING.md) e o [contrato dos adaptadores](https://github.com/bernardopg/wingdrive/blob/main/docs/core/adapters.mdx), e abra um pull request no registro. O Wingbot verifica e um mantenedor revisa; adaptadores novos são anunciados no canal **#adapters** do Discord.
