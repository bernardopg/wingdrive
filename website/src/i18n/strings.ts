// Copy for every marketing page, in English and Brazilian Portuguese.
// Docs pages live in src/content/docs/<lang>/ instead.

export type Lang = "en" | "pt";

export const LANGS: Lang[] = ["en", "pt"];

/** Prefixes a site path with the locale segment (English has none). */
export function href(lang: Lang, path: string): string {
	const clean = path.startsWith("/") ? path : `/${path}`;
	if (lang === "en") return clean;
	return clean === "/" ? "/pt/" : `/pt${clean}`;
}

/** Returns the same page in the other language. */
export function swapLang(pathname: string, to: Lang): string {
	const base = pathname.replace(/^\/pt(\/|$)/, "/");
	return href(to, base);
}

export const REPO = "https://github.com/bernardopg/wingdrive";
export const RELEASES = `${REPO}/releases`;
export const SPONSOR = "https://github.com/sponsors/bernardopg";
export const DISCORD = "https://discord.gg/n8uwvrYMCh";
export const AUR = "https://aur.archlinux.org/packages/wingdrive-bin/";
export const DOCKER = "ghcr.io/bernardopg/wingdrive/server";

const en = {
	meta: {
		title: "WingDrive — Every drive you own, in one fast file manager",
		description:
			"WingDrive is a local-first, open-source file manager that indexes your folders, external drives and cloud volumes into one fast library. No account, no telemetry.",
	},
	nav: {
		features: "Features",
		why: "Why WingDrive",
		docs: "Docs",
		download: "Download",
		about: "About",
		changelog: "Changelog",
		github: "GitHub",
		skip: "Skip to content",
		menu: "Menu",
		lang: "Português",
	},
	hero: {
		badge: "3.2 is out · Google Drive and Google Photos",
		title1: "Every drive you own,",
		title2: "in one fast file manager.",
		lead: "WingDrive indexes your folders, external drives and cloud volumes into one library. Files stay where they are; the index makes them fast to browse, search and recognize across your own devices.",
		ctaDownload: "Download for",
		ctaDocs: "Read the docs",
		meta: ["Open source · Apache-2.0", "No account", "No telemetry"],
	},
	strip: {
		title: "Built on a modern, auditable stack",
		items: ["Rust", "Tokio", "SQLite", "Iroh / QUIC", "BLAKE3", "OpenDAL", "Tauri 2", "React 19"],
	},
	universe: {
		eyebrow: "One library",
		title: "Your SSD, your HDD, your USB stick and your clouds, in one view.",
		lead: "Add a folder, plug in a drive or connect a bucket. WingDrive indexes it into the same virtual filesystem, so search, tags and previews work the same everywhere. Drag to spin the scene.",
		labels: { ssd: "Internal SSD", hdd: "External HDD", usb: "USB stick", nas: "Home server", cloud: "Cloud volumes" },
		stats: [
			{ k: "40+", v: "cloud services through OpenDAL" },
			{ k: "58 KB", v: "read to identify a 10 GB file" },
			{ k: "0", v: "bytes of telemetry" },
		],
	},
	features: {
		eyebrow: "Features",
		title: "Everything a file manager should do. Then some.",
		lead: "WingDrive aims to replace Dolphin, Nautilus or Yazi as your default file manager on Linux, and it already covers a full day of work.",
		items: [
			{ icon: "search", t: "Find it in seconds", d: "Search refines by kind, extension, size, date and hidden files, with counts over the whole result, not just the first page." },
			{ icon: "views", t: "See folders your way", d: "Grid, list, columns, media and size views, sections by kind or date, and Quick Preview for images, video, audio, PDF, text and code." },
			{ icon: "terminal", t: "Work where your code lives", d: "Git emblems on files and folders, expandable folders in the list, dual-pane Explorer (F3) and a terminal docked under the view (F4)." },
			{ icon: "undo", t: "Undo that means it", d: "Copy, overwrite, move and trash are journaled. Ctrl+Z reverses them, and anything that changed since is left alone." },
			{ icon: "box", t: "The full toolbox", d: "Copy To and Move To, compress and extract, templates, Dolphin service menus, permissions and SHA-256 checksums." },
			{ icon: "shield", t: "Guarded operations", d: "Copies preflight space and conflicts and never write into their own source. Secure delete refuses copy-on-write filesystems." },
			{ icon: "linux", t: "Feels native on Linux", d: "Opens folders for xdg-open and “Show in folder”, starts with your session and lives in the tray." },
			{ icon: "palette", t: "Make it yours", d: "Catppuccin, Gruvbox, Dracula and Tokyo Night themes, in English and Brazilian Portuguese." },
		],
	},
	showcase: {
		eyebrow: "A closer look",
		title: "Fast to read. Calm to use.",
		tabs: [
			{ id: "grid", t: "Grid + Inspector", img: "WingGridView", alt: "Grid view with the Inspector panel open, dark theme" },
			{ id: "list", t: "List + terminal", img: "WingListView", alt: "List view with Git emblems and the docked terminal" },
			{ id: "search", t: "Search", img: "WingSearch", alt: "Search with kind, extension, size and date filters" },
			{ id: "media", t: "Media", img: "WingMediaView", alt: "Media view of a photo folder" },
			{ id: "columns", t: "Columns", img: "WingColumnView", alt: "Column view of nested folders" },
			{ id: "size", t: "Size map", img: "WingSizeView", alt: "Size view showing which folders use the most space" },
		],
	},
	why: {
		eyebrow: "Why WingDrive",
		title: "Different by design.",
		lead: "Most tools either lock your files into a cloud or forget everything the moment a drive is unplugged. WingDrive does neither.",
		items: [
			{ img: "privacy", t: "Local-first. Yours.", d: "Indexing, thumbnails and search run on your machine. No account, no telemetry, no central server. Back up the whole library by copying one folder." },
			{ img: "identity", t: "It knows what a file is", d: "Every file gets a BLAKE3 content identity, so the same photo on your laptop and on Google Drive is recognized as one file. Large files need only 58 KB sampled." },
			{ img: "devices", t: "Your devices, paired directly", d: "Pair laptop, phone and home server with a 12-word code or a QR. Traffic is end-to-end encrypted over Iroh/QUIC, with no server of ours in the middle." },
			{ img: "cloud", t: "Clouds as plain volumes", d: "Google Drive and Google Photos sign in through your browser, with several accounts per library. S3, Dropbox, OneDrive, Azure Blob and GCS mount as volumes too. Copy, move, edit and sync them like local folders; credentials stay encrypted with your library key." },
		],
	},
	sync: {
		eyebrow: "Devices",
		title: "Pair once. Your library follows you.",
		lead: "Library sync shares the index, tags and metadata between the devices you own. Wingdrop sends files to a paired device in one click.",
		steps: [
			{ t: "Generate a code", d: "One device shows 12 words or a QR code, valid for five minutes and never reused." },
			{ t: "Join from the other", d: "Ed25519 signatures and a challenge prove both sides. Local network or across the internet with relay fallback." },
			{ t: "Sync and send", d: "The library stays in step, and renamed or reinstalled devices keep their names current." },
		],
	},
	cloud: {
		eyebrow: "Cloud",
		title: "Every cloud, one address space.",
		lead: "Cloud paths use the URIs you already know, like s3://, gdrive:// and onedrive://. Copy a path from WingDrive and paste it into the provider's own CLI.",
	},
	arch: {
		eyebrow: "Under the hood",
		title: "One daemon. Many thin clients.",
		lead: "A single wing-daemon owns all state. The desktop app, the wing CLI and wing-server with its web interface speak JSON-RPC to it, so a script and a window always see the same library.",
		clients: ["Desktop app", "wing CLI", "Web · wing-server", "Mobile (embedded core)"],
		core: "wing-daemon",
		layers: ["Actions & queries (CQRS)", "Jobs that resume", "Indexer + watcher", "P2P networking", "SQLite library"],
		storage: ["Local disks", "Removable drives", "Cloud volumes", "Paired devices"],
	},
	platforms: {
		eyebrow: "Platforms",
		title: "Where WingDrive runs today",
		items: [
			{ os: "Linux", s: "Supported", tone: "ok", d: "AppImage, .deb and AUR (wingdrive-bin, wingdrive-nightly)." },
			{ os: "Windows 10 / 11", s: "Beta", tone: "warn", d: ".exe and .msi installers. Unsigned for now, so SmartScreen warns." },
			{ os: "macOS", s: "From source", tone: "dim", d: "Signed builds and auto-updates are planned for milestone M3." },
			{ os: "Server", s: "Supported", tone: "ok", d: "Docker image for x86_64 and ARM64, or a static Linux binary." },
			{ os: "Android", s: "In development", tone: "dim", d: "Preview APKs ship with releases. Not ready for daily use." },
		],
	},
	roadmap: {
		eyebrow: "Roadmap",
		title: "Where we are, where we're going",
		items: [
			{ m: "M1", s: "Done", t: "A reliable desktop file manager on Linux, from a production bundle." },
			{ m: "M2", s: "Now", t: "The default file manager on Linux: dual-pane Explorer and a full day of use without falling back." },
			{ m: "M3", s: "Next", t: "Signed releases for macOS and Windows, and auto-updates. Copy preflight and undo already shipped." },
			{ m: "M4", s: "Later", t: "File sync between devices, public sharing links, Wingdrop to nearby devices, Tailscale." },
			{ m: "M5", s: "Later", t: "Semantic search, WASM extensions, encryption at rest, Android and iOS releases." },
		],
	},
	cta: {
		title: "Give your drives wings.",
		lead: "Free and open source. Install it in a minute, keep every file exactly where it is.",
		primary: "Download WingDrive",
		secondary: "Star on GitHub",
	},
	footer: {
		tagline: "Every drive you own, in one fast file manager. Local-first. Indexed. Yours.",
		product: "Product",
		resources: "Resources",
		community: "Community",
		links: {
			features: "Features",
			download: "Download",
			changelog: "Changelog",
			roadmap: "Roadmap",
			docs: "Documentation",
			install: "Installation",
			selfhost: "Self-hosting",
			cli: "CLI",
			github: "Source code",
			issues: "Report a bug",
			discussions: "Discussions",
			discord: "Discord",
			sponsor: "Sponsor",
			about: "About",
			privacy: "Privacy",
		},
		legal: "WingDrive is open source under the Apache License 2.0.",
	},
	download: {
		title: "Download WingDrive",
		lead: "Free and open source, with SHA-256 checksums for every file. Pick your platform below.",
		latest: "Latest release",
		published: "Published",
		notes: "Release notes",
		all: "All releases",
		verify: "Verify your download",
		verifyLead: "Every release ships SHA256SUMS files. Check the file before you run it:",
		recommended: "Recommended for you",
		linux: {
			t: "Linux",
			s: "Supported",
			d: "x86_64. The AppImage runs on any distribution; the .deb fits Debian, Ubuntu and derivatives.",
			aurT: "Arch Linux and derivatives",
			aurD: "wingdrive-bin follows stable releases. wingdrive-nightly follows every release, alpha, beta and rc included.",
		},
		windows: {
			t: "Windows 10 and 11",
			s: "Beta",
			d: "x64. The installers are not signed yet, so SmartScreen warns on first run: choose “More info”, then “Run anyway”.",
		},
		mac: {
			t: "macOS",
			s: "From source",
			d: "Prebuilt, signed builds are planned for milestone M3. Until then, build from source:",
		},
		android: {
			t: "Android",
			s: "Preview",
			d: "arm64-v8a. An early preview of the mobile app that embeds the same Rust core. Expect rough edges.",
		},
		server: {
			t: "Headless server",
			s: "Supported",
			d: "Run the daemon and the web interface on a NAS, a home server or a VPS. Set WING_AUTH before exposing it.",
		},
		source: {
			t: "Build from source",
			d: "Needs the Rust toolchain pinned in rust-toolchain.toml, Bun 1.3+ and just.",
		},
		after: "After installing, WingDrive can open folders for xdg-open and “Show in folder” in other apps, start with your session and stay in the tray.",
	},
	about: {
		title: "About WingDrive",
		lead: "A file manager that treats every drive you own as one library, and treats you as the owner of your data.",
		storyT: "The story",
		story: [
			"Our files are scattered: a laptop SSD, an external disk in a drawer, a phone, a couple of cloud accounts. Each place has its own app, its own search and its own idea of what a file is.",
			"WingDrive puts a fast index on top of all of them. Files never move unless you move them. The index lives on your machine, recognizes the same file wherever it is, and travels only between devices you paired yourself.",
			"The first goal is humble and concrete: to be a file manager good enough to be your default on Linux. Everything else, from device sync to semantic search, is built on that foundation.",
		],
		principlesT: "Principles",
		principles: [
			{ t: "You are sovereign", d: "Local-first by default. No account, no telemetry, no lock-in. Your files stay in their original locations." },
			{ t: "Content is king", d: "WingDrive understands what a file is, not only where it is, so duplicates and copies across devices make sense." },
			{ t: "Never lose data", d: "Destructive operations are guarded, copies are atomic, and undo exists for the moves you regret." },
			{ t: "Open by default", d: "Apache-2.0, built in the open, with the roadmap and every sprint published in the repository." },
		],
		makerT: "Who builds it",
		maker: "WingDrive is built in the open by Bernardo Gomes, its author and maintainer. Bug reports, ideas and pull requests are welcome, and sponsorships keep the project moving.",
		creditsT: "Credits",
		credits: "WingDrive is an independent project derived from open-source code whose original authors are credited in CREDITS.md. It builds on great open-source work such as Tauri, Iroh, OpenDAL, SQLite and BLAKE3.",
		contact: "Contact",
	},
	changelog: {
		title: "Changelog",
		lead: "Every notable change, release by release. Generated from CHANGELOG.md in the repository.",
	},
	privacy: {
		title: "Privacy",
		lead: "Short version: we don't collect anything.",
		body: [
			"The WingDrive app has no telemetry and no account. Indexing, thumbnails and search run on your device. Data travels only between devices you pair, end-to-end encrypted, or to cloud services you connect yourself.",
			"This website has no cookies, no analytics and no third-party trackers. The download page asks the public GitHub API for the latest release from your browser, which GitHub may log like any request.",
			"Questions? Write to the maintainer at the address on the About page.",
		],
	},
	docs: {
		title: "Documentation",
		lead: "Install WingDrive, learn the concepts and get the most out of it.",
		onThisPage: "On this page",
		edit: "Full technical docs on GitHub",
		editPage: "Edit this page",
		quickTitle: "Start in three steps",
		quick: [
			{ slug: "installation", t: "Install", d: "AUR, AppImage, .deb, Windows installer or Docker." },
			{ slug: "first-run", t: "Add your folders", d: "Create a library and pick the folders WingDrive indexes." },
			{ slug: "explorer", t: "Find your way", d: "Views, search, Quick Preview, tabs and shortcuts." },
		],
		newTitle: "New in 3.2",
		newText: "Google Drive and Google Photos, file operations on cloud volumes and two-way cloud sync.",
		newLink: "Set up cloud volumes",
		prev: "Previous",
		next: "Next",
		search: "Filter pages…",
	},
	notFound: {
		title: "This page flew away.",
		lead: "The address may have changed. Try the home page or the documentation.",
		home: "Go home",
	},
};

export type Strings = typeof en;

const pt: Strings = {
	meta: {
		title: "WingDrive — Todos os seus drives em um gerenciador de arquivos rápido",
		description:
			"WingDrive é um gerenciador de arquivos local-first e de código aberto que indexa suas pastas, drives externos e volumes na nuvem em uma biblioteca rápida. Sem conta, sem telemetria.",
	},
	nav: {
		features: "Recursos",
		why: "Por que WingDrive",
		docs: "Docs",
		download: "Download",
		about: "Sobre",
		changelog: "Novidades",
		github: "GitHub",
		skip: "Pular para o conteúdo",
		menu: "Menu",
		lang: "English",
	},
	hero: {
		badge: "3.2 saiu · Google Drive e Google Fotos",
		title1: "Todos os seus drives,",
		title2: "em um gerenciador de arquivos rápido.",
		lead: "O WingDrive indexa suas pastas, drives externos e volumes na nuvem em uma só biblioteca. Os arquivos ficam onde estão; o índice deixa tudo rápido de navegar, buscar e reconhecer entre os seus dispositivos.",
		ctaDownload: "Baixar para",
		ctaDocs: "Ler a documentação",
		meta: ["Código aberto · Apache-2.0", "Sem conta", "Sem telemetria"],
	},
	strip: {
		title: "Feito sobre uma base moderna e auditável",
		items: ["Rust", "Tokio", "SQLite", "Iroh / QUIC", "BLAKE3", "OpenDAL", "Tauri 2", "React 19"],
	},
	universe: {
		eyebrow: "Uma biblioteca",
		title: "Seu SSD, seu HD, seu pendrive e suas nuvens, em uma só visão.",
		lead: "Adicione uma pasta, conecte um drive ou ligue um bucket. O WingDrive indexa tudo no mesmo sistema de arquivos virtual, então busca, tags e prévias funcionam igual em todo lugar. Arraste para girar a cena.",
		labels: { ssd: "SSD interno", hdd: "HD externo", usb: "Pendrive", nas: "Servidor doméstico", cloud: "Volumes na nuvem" },
		stats: [
			{ k: "40+", v: "serviços de nuvem via OpenDAL" },
			{ k: "58 KB", v: "lidos para identificar um arquivo de 10 GB" },
			{ k: "0", v: "bytes de telemetria" },
		],
	},
	features: {
		eyebrow: "Recursos",
		title: "Tudo o que um gerenciador de arquivos deve fazer. E mais.",
		lead: "O WingDrive quer substituir o Dolphin, o Nautilus ou o Yazi como gerenciador de arquivos padrão no Linux, e já aguenta um dia inteiro de trabalho.",
		items: [
			{ icon: "search", t: "Encontre em segundos", d: "A busca refina por tipo, extensão, tamanho, data e arquivos ocultos, com contagens sobre o resultado inteiro, não só a primeira página." },
			{ icon: "views", t: "Veja pastas do seu jeito", d: "Visões em grade, lista, colunas, mídia e tamanho, seções por tipo ou data, e Quick Preview para imagens, vídeo, áudio, PDF, texto e código." },
			{ icon: "terminal", t: "Trabalhe onde o código está", d: "Emblemas do Git em arquivos e pastas, pastas expansíveis na lista, Explorer em dois painéis (F3) e um terminal acoplado à visão (F4)." },
			{ icon: "undo", t: "Desfazer de verdade", d: "Copiar, sobrescrever, mover e enviar à lixeira ficam registrados. Ctrl+Z reverte, e o que mudou desde então fica intocado." },
			{ icon: "box", t: "A caixa de ferramentas completa", d: "Copiar Para e Mover Para, compactar e extrair, modelos, service menus do Dolphin, permissões e checksums SHA-256." },
			{ icon: "shield", t: "Operações protegidas", d: "Cópias checam espaço e conflitos antes e nunca escrevem dentro da própria origem. A exclusão segura recusa sistemas de arquivos copy-on-write." },
			{ icon: "linux", t: "Nativo no Linux", d: "Abre pastas para o xdg-open e o “Mostrar na pasta”, inicia com a sessão e fica na bandeja." },
			{ icon: "palette", t: "Do seu jeito", d: "Temas Catppuccin, Gruvbox, Dracula e Tokyo Night, em inglês e português do Brasil." },
		],
	},
	showcase: {
		eyebrow: "De perto",
		title: "Rápido de ler. Calmo de usar.",
		tabs: [
			{ id: "grid", t: "Grade + Inspector", img: "WingGridView", alt: "Visão em grade com o painel Inspector aberto, tema escuro" },
			{ id: "list", t: "Lista + terminal", img: "WingListView", alt: "Visão em lista com emblemas do Git e o terminal acoplado" },
			{ id: "search", t: "Busca", img: "WingSearch", alt: "Busca com filtros de tipo, extensão, tamanho e data" },
			{ id: "media", t: "Mídia", img: "WingMediaView", alt: "Visão de mídia de uma pasta de fotos" },
			{ id: "columns", t: "Colunas", img: "WingColumnView", alt: "Visão em colunas de pastas aninhadas" },
			{ id: "size", t: "Mapa de tamanho", img: "WingSizeView", alt: "Visão de tamanho mostrando as pastas que mais ocupam espaço" },
		],
	},
	why: {
		eyebrow: "Por que WingDrive",
		title: "Diferente por projeto.",
		lead: "A maioria das ferramentas prende seus arquivos numa nuvem ou esquece tudo no momento em que um drive é desconectado. O WingDrive não faz nenhum dos dois.",
		items: [
			{ img: "privacy", t: "Local-first. Seu.", d: "Indexação, miniaturas e busca rodam na sua máquina. Sem conta, sem telemetria, sem servidor central. Faça backup da biblioteca inteira copiando uma pasta." },
			{ img: "identity", t: "Ele sabe o que um arquivo é", d: "Cada arquivo ganha uma identidade de conteúdo BLAKE3, então a mesma foto no notebook e no Google Drive é reconhecida como um só arquivo. Arquivos grandes precisam de só 58 KB amostrados." },
			{ img: "devices", t: "Seus dispositivos, pareados direto", d: "Pareie notebook, celular e servidor doméstico com um código de 12 palavras ou um QR. O tráfego é criptografado ponta a ponta via Iroh/QUIC, sem servidor nosso no meio." },
			{ img: "cloud", t: "Nuvens como volumes comuns", d: "Google Drive e Google Fotos entram pelo navegador, com várias contas por biblioteca. S3, Dropbox, OneDrive, Azure Blob e GCS também montam como volumes. Copie, mova, edite e sincronize como pastas locais; as credenciais ficam criptografadas com a chave da biblioteca." },
		],
	},
	sync: {
		eyebrow: "Dispositivos",
		title: "Pareie uma vez. Sua biblioteca vai junto.",
		lead: "A sincronização da biblioteca compartilha o índice, as tags e os metadados entre os seus dispositivos. O Wingdrop envia arquivos a um dispositivo pareado com um clique.",
		steps: [
			{ t: "Gere um código", d: "Um dispositivo mostra 12 palavras ou um QR code, válido por cinco minutos e nunca reutilizado." },
			{ t: "Entre pelo outro", d: "Assinaturas Ed25519 e um desafio provam os dois lados. Na rede local ou pela internet, com relay de reserva." },
			{ t: "Sincronize e envie", d: "A biblioteca fica em dia, e dispositivos renomeados ou reinstalados mantêm o nome atualizado." },
		],
	},
	cloud: {
		eyebrow: "Nuvem",
		title: "Todas as nuvens, um só espaço de endereços.",
		lead: "Caminhos na nuvem usam as URIs que você já conhece, como s3://, gdrive:// e onedrive://. Copie um caminho no WingDrive e cole na CLI do próprio provedor.",
	},
	arch: {
		eyebrow: "Por dentro",
		title: "Um daemon. Vários clientes leves.",
		lead: "Um único wing-daemon guarda todo o estado. O app desktop, a CLI wing e o wing-server com sua interface web falam JSON-RPC com ele, então um script e uma janela sempre veem a mesma biblioteca.",
		clients: ["App desktop", "CLI wing", "Web · wing-server", "Mobile (core embutido)"],
		core: "wing-daemon",
		layers: ["Ações e consultas (CQRS)", "Jobs que retomam", "Indexador + watcher", "Rede P2P", "Biblioteca SQLite"],
		storage: ["Discos locais", "Drives removíveis", "Volumes na nuvem", "Dispositivos pareados"],
	},
	platforms: {
		eyebrow: "Plataformas",
		title: "Onde o WingDrive roda hoje",
		items: [
			{ os: "Linux", s: "Suportado", tone: "ok", d: "AppImage, .deb e AUR (wingdrive-bin, wingdrive-nightly)." },
			{ os: "Windows 10 / 11", s: "Beta", tone: "warn", d: "Instaladores .exe e .msi. Ainda sem assinatura, então o SmartScreen avisa." },
			{ os: "macOS", s: "Pelo código-fonte", tone: "dim", d: "Builds assinados e atualização automática estão planejados para o marco M3." },
			{ os: "Servidor", s: "Suportado", tone: "ok", d: "Imagem Docker para x86_64 e ARM64, ou um binário estático para Linux." },
			{ os: "Android", s: "Em desenvolvimento", tone: "dim", d: "APKs de prévia saem com as releases. Ainda não é para uso diário." },
		],
	},
	roadmap: {
		eyebrow: "Roadmap",
		title: "Onde estamos, para onde vamos",
		items: [
			{ m: "M1", s: "Feito", t: "Um gerenciador de arquivos confiável no Linux, a partir de um bundle de produção." },
			{ m: "M2", s: "Agora", t: "O gerenciador de arquivos padrão no Linux: Explorer em dois painéis e um dia inteiro de uso sem voltar ao antigo." },
			{ m: "M3", s: "Próximo", t: "Releases assinadas para macOS e Windows e atualização automática. Pré-checagem de cópia e desfazer já saíram." },
			{ m: "M4", s: "Depois", t: "Sincronização de arquivos entre dispositivos, links públicos, Wingdrop para dispositivos próximos, Tailscale." },
			{ m: "M5", s: "Depois", t: "Busca semântica, extensões WASM, criptografia em repouso, releases para Android e iOS." },
		],
	},
	cta: {
		title: "Dê asas aos seus drives.",
		lead: "Livre e de código aberto. Instale em um minuto e mantenha cada arquivo exatamente onde está.",
		primary: "Baixar o WingDrive",
		secondary: "Estrela no GitHub",
	},
	footer: {
		tagline: "Todos os seus drives em um gerenciador de arquivos rápido. Local-first. Indexado. Seu.",
		product: "Produto",
		resources: "Recursos",
		community: "Comunidade",
		links: {
			features: "Recursos",
			download: "Download",
			changelog: "Novidades",
			roadmap: "Roadmap",
			docs: "Documentação",
			install: "Instalação",
			selfhost: "Auto-hospedagem",
			cli: "CLI",
			github: "Código-fonte",
			issues: "Relatar um bug",
			discussions: "Discussões",
			discord: "Discord",
			sponsor: "Apoiar",
			about: "Sobre",
			privacy: "Privacidade",
		},
		legal: "O WingDrive é código aberto sob a Licença Apache 2.0.",
	},
	download: {
		title: "Baixar o WingDrive",
		lead: "Livre e de código aberto, com checksums SHA-256 para cada arquivo. Escolha sua plataforma abaixo.",
		latest: "Última versão",
		published: "Publicada em",
		notes: "Notas da versão",
		all: "Todas as versões",
		verify: "Verifique o download",
		verifyLead: "Toda release traz arquivos SHA256SUMS. Confira o arquivo antes de executar:",
		recommended: "Recomendado para você",
		linux: {
			t: "Linux",
			s: "Suportado",
			d: "x86_64. O AppImage roda em qualquer distribuição; o .deb serve para Debian, Ubuntu e derivados.",
			aurT: "Arch Linux e derivados",
			aurD: "wingdrive-bin acompanha as versões estáveis. wingdrive-nightly acompanha todas, incluindo alpha, beta e rc.",
		},
		windows: {
			t: "Windows 10 e 11",
			s: "Beta",
			d: "x64. Os instaladores ainda não são assinados, então o SmartScreen avisa na primeira execução: escolha “Mais informações” e depois “Executar assim mesmo”.",
		},
		mac: {
			t: "macOS",
			s: "Pelo código-fonte",
			d: "Builds prontos e assinados estão planejados para o marco M3. Até lá, compile pelo código-fonte:",
		},
		android: {
			t: "Android",
			s: "Prévia",
			d: "arm64-v8a. Uma prévia inicial do app mobile, que embute o mesmo core em Rust. Espere arestas.",
		},
		server: {
			t: "Servidor headless",
			s: "Suportado",
			d: "Rode o daemon e a interface web num NAS, servidor doméstico ou VPS. Defina WING_AUTH antes de expor.",
		},
		source: {
			t: "Compilar pelo código-fonte",
			d: "Precisa do toolchain Rust fixado em rust-toolchain.toml, Bun 1.3+ e just.",
		},
		after: "Depois de instalar, o WingDrive pode abrir pastas para o xdg-open e o “Mostrar na pasta” de outros apps, iniciar com a sessão e ficar na bandeja.",
	},
	about: {
		title: "Sobre o WingDrive",
		lead: "Um gerenciador de arquivos que trata todos os seus drives como uma só biblioteca, e trata você como dono dos seus dados.",
		storyT: "A história",
		story: [
			"Nossos arquivos vivem espalhados: o SSD do notebook, um HD externo na gaveta, o celular, algumas contas de nuvem. Cada lugar tem seu app, sua busca e sua ideia do que é um arquivo.",
			"O WingDrive coloca um índice rápido por cima de todos eles. Os arquivos não saem do lugar a menos que você os mova. O índice vive na sua máquina, reconhece o mesmo arquivo onde quer que ele esteja e só viaja entre dispositivos que você mesmo pareou.",
			"O primeiro objetivo é modesto e concreto: ser um gerenciador de arquivos bom o bastante para ser o seu padrão no Linux. Todo o resto, da sincronização entre dispositivos à busca semântica, é construído sobre essa base.",
		],
		principlesT: "Princípios",
		principles: [
			{ t: "Você é soberano", d: "Local-first por padrão. Sem conta, sem telemetria, sem aprisionamento. Seus arquivos ficam nos lugares originais." },
			{ t: "O conteúdo manda", d: "O WingDrive entende o que um arquivo é, não só onde ele está, então duplicatas e cópias entre dispositivos fazem sentido." },
			{ t: "Nunca perder dados", d: "Operações destrutivas são protegidas, cópias são atômicas e existe desfazer para os movimentos que você se arrepende." },
			{ t: "Aberto por padrão", d: "Apache-2.0, feito às claras, com o roadmap e cada sprint publicados no repositório." },
		],
		makerT: "Quem faz",
		maker: "O WingDrive é feito às claras por Bernardo Gomes, autor e mantenedor. Relatos de bugs, ideias e pull requests são bem-vindos, e o patrocínio mantém o projeto andando.",
		creditsT: "Créditos",
		credits: "O WingDrive é um projeto independente derivado de código aberto cujos autores originais estão creditados em CREDITS.md. Ele se apoia em ótimos projetos abertos como Tauri, Iroh, OpenDAL, SQLite e BLAKE3.",
		contact: "Contato",
	},
	changelog: {
		title: "Novidades",
		lead: "Cada mudança relevante, versão a versão. Gerado a partir do CHANGELOG.md do repositório (em inglês).",
	},
	privacy: {
		title: "Privacidade",
		lead: "Versão curta: não coletamos nada.",
		body: [
			"O app WingDrive não tem telemetria nem conta. Indexação, miniaturas e busca rodam no seu dispositivo. Os dados só viajam entre dispositivos que você pareia, com criptografia ponta a ponta, ou para serviços de nuvem que você mesmo conecta.",
			"Este site não usa cookies, analytics nem rastreadores de terceiros. A página de download consulta a API pública do GitHub pelo seu navegador para saber a última versão, e o GitHub pode registrar essa requisição como qualquer outra.",
			"Dúvidas? Escreva para o mantenedor no endereço da página Sobre.",
		],
	},
	docs: {
		title: "Documentação",
		lead: "Instale o WingDrive, entenda os conceitos e aproveite ao máximo.",
		onThisPage: "Nesta página",
		edit: "Documentação técnica completa no GitHub",
		editPage: "Editar esta página",
		quickTitle: "Comece em três passos",
		quick: [
			{ slug: "installation", t: "Instale", d: "AUR, AppImage, .deb, instalador do Windows ou Docker." },
			{ slug: "first-run", t: "Adicione suas pastas", d: "Crie uma biblioteca e escolha as pastas que o WingDrive indexa." },
			{ slug: "explorer", t: "Encontre o caminho", d: "Visualizações, busca, Visualização Rápida, abas e atalhos." },
		],
		newTitle: "Novo na 3.2",
		newText: "Google Drive e Google Fotos, operações de arquivo em volumes na nuvem e sincronização com a nuvem nos dois sentidos.",
		newLink: "Configurar volumes na nuvem",
		prev: "Anterior",
		next: "Próxima",
		search: "Filtrar páginas…",
	},
	notFound: {
		title: "Esta página voou para longe.",
		lead: "O endereço pode ter mudado. Tente a página inicial ou a documentação.",
		home: "Ir para o início",
	},
};

export const t: Record<Lang, Strings> = { en, pt };
