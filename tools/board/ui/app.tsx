import { marked } from "marked";
import { type DragEvent, type ReactNode, StrictMode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";

import type { Board, Issue, Milestone, Priority, Sprint, Status, Task, TaskDetail, TaskPatch } from "../src/store";

type BoardData = Board & { issues: Issue[] };
type View = "board" | "sprints" | "roadmap";

const STATUSES: Status[] = ["To Do", "In Progress", "Done"];
const PRIORITIES: Priority[] = ["High", "Medium", "Low"];
const AREAS = ["core", "interface", "mobile", "testing", "adapters"];
const WIP_LIMIT = 2;
const DONE_PAGE = 24;
const STATUS_HINT: Record<Status, string> = {
	"To Do": "Na fila",
	"In Progress": "Em voo",
	Done: "Pousou",
};
const PRIORITY_LABEL: Record<Priority, string> = { High: "Alta", Medium: "Média", Low: "Baixa" };

marked.use({
	gfm: true,
	renderer: {
		// Raw HTML in task files is shown as text, never injected.
		html: ({ text }) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;"),
	},
});

// ---------------------------------------------------------------------------------------------
// Data

/** Path of one task's endpoint; ids come from the URL hash, so they are checked and encoded. */
function taskPath(id: string, suffix = ""): string {
	if (!/^[A-Za-z]+-\d+$/.test(id)) throw new Error(`invalid task id ${id}`);
	return `/api/tasks/${encodeURIComponent(id)}${suffix}`;
}

async function api<T>(path: string, init?: RequestInit & { json?: unknown }): Promise<T> {
	if (!path.startsWith("/api/")) throw new Error(`refusing request to ${path}`);
	const res = await fetch(new URL(path, location.origin), {
		...init,
		headers: init?.json !== undefined ? { "content-type": "application/json" } : undefined,
		body: init?.json !== undefined ? JSON.stringify(init.json) : undefined,
	});
	const data = await res.json();
	if (!res.ok) throw new Error(data.error ?? res.statusText);
	return data as T;
}

interface Filters {
	sprint: string; // "" = todos, "current" = sprint atual
	milestone: string;
	priority: string;
	area: string;
	parent: string;
	query: string;
}

const EMPTY_FILTERS: Filters = { sprint: "current", milestone: "", priority: "", area: "", parent: "", query: "" };

function loadFilters(): Filters {
	try {
		const raw = localStorage.getItem("board.filters");
		return raw ? { ...EMPTY_FILTERS, ...JSON.parse(raw), query: "" } : EMPTY_FILTERS;
	} catch {
		return EMPTY_FILTERS;
	}
}

function saveFilters(f: Filters) {
	try {
		localStorage.setItem("board.filters", JSON.stringify(f));
	} catch {
		/* storage blocked: filters just reset next time */
	}
}

function readHash(): { view: View; task?: string } {
	const p = new URLSearchParams(location.hash.slice(1));
	const view = (p.get("view") as View) || "board";
	return { view: ["board", "sprints", "roadmap"].includes(view) ? view : "board", task: p.get("task") ?? undefined };
}

function writeHash(view: View, task?: string) {
	const p = new URLSearchParams();
	if (view !== "board") p.set("view", view);
	if (task) p.set("task", task);
	const next = p.toString();
	if (next !== location.hash.slice(1)) history.replaceState(null, "", next ? `#${next}` : location.pathname);
}

// ---------------------------------------------------------------------------------------------
// Helpers

function count(tasks: Task[]) {
	const done = tasks.filter((t) => t.status === "Done").length;
	const doing = tasks.filter((t) => t.status === "In Progress").length;
	return { done, doing, todo: tasks.length - done - doing, total: tasks.length };
}

function pct(n: number, total: number) {
	return total ? Math.round((n / total) * 100) : 0;
}

function daysBetween(a: string, b: string) {
	return Math.round((Date.parse(b) - Date.parse(a)) / 86_400_000);
}

function fmtDate(d?: string) {
	if (!d) return "";
	const [, m, day] = d.split("-");
	const months = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
	return `${Number(day)} ${months[Number(m) - 1]}`;
}

function ago(today: string, d?: string) {
	if (!d) return "sem data";
	const n = daysBetween(d, today);
	if (n <= 0) return "hoje";
	if (n === 1) return "ontem";
	if (n < 30) return `há ${n} dias`;
	return `há ${Math.round(n / 30)} ${Math.round(n / 30) === 1 ? "mês" : "meses"}`;
}

function cx(...names: (string | false | undefined)[]) {
	return names.filter(Boolean).join(" ");
}

// ---------------------------------------------------------------------------------------------
// App

function App() {
	const [data, setData] = useState<BoardData>();
	const [error, setError] = useState<string>();
	const initial = readHash();
	const [view, setView] = useState<View>(initial.view);
	const [openId, setOpenId] = useState<string | undefined>(initial.task);
	const [filters, setFiltersState] = useState<Filters>(loadFilters);
	const [toasts, setToasts] = useState<{ id: number; text: string; tone?: "warn" | "error" }[]>([]);
	const [creating, setCreating] = useState(false);
	const [refreshKey, setRefreshKey] = useState(0);
	const ownWrite = useRef(0);
	const search = useRef<HTMLInputElement>(null);

	const toast = useCallback((text: string, tone?: "warn" | "error") => {
		const id = Date.now() + Math.random();
		setToasts((t) => [...t.slice(-3), { id, text, tone }]);
		setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), tone ? 7000 : 3500);
	}, []);

	const refresh = useCallback(async () => {
		try {
			setData(await api<BoardData>("/api/board"));
			setError(undefined);
		} catch (e) {
			setError((e as Error).message);
		}
	}, []);

	useEffect(() => {
		refresh();
		const events = new EventSource("/api/events");
		events.addEventListener("board", (e) => {
			const { changed } = JSON.parse((e as MessageEvent).data) as { changed: string[] };
			refresh();
			setRefreshKey((k) => k + 1);
			if (changed.length && Date.now() - ownWrite.current > 2000) {
				toast(`Atualizado no disco: ${changed.slice(0, 4).join(", ")}${changed.length > 4 ? ` e mais ${changed.length - 4}` : ""}`);
			}
		});
		events.onerror = () => setError("Sem conexão com o servidor do board. Rode `just board`.");
		events.onopen = () => setError(undefined);
		return () => events.close();
	}, [refresh, toast]);

	useEffect(() => writeHash(view, openId), [view, openId]);

	const setFilters = useCallback((patch: Partial<Filters>) => {
		setFiltersState((f) => {
			const next = { ...f, ...patch };
			saveFilters(next);
			return next;
		});
	}, []);

	useEffect(() => {
		const onKey = (e: KeyboardEvent) => {
			const typing = (e.target as HTMLElement).closest("input, textarea, select");
			if (e.key === "/" && !typing) {
				e.preventDefault();
				setView("board");
				setTimeout(() => search.current?.focus());
			} else if (e.key === "Escape") {
				if (typing) (e.target as HTMLElement).blur();
				else if (creating) setCreating(false);
				else setOpenId(undefined);
			} else if (!typing && !e.ctrlKey && !e.metaKey && !e.altKey) {
				const v = ({ "1": "board", "2": "sprints", "3": "roadmap" } as Record<string, View>)[e.key];
				if (v) setView(v);
				if (e.key === "n") setCreating(true);
			}
		};
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [creating]);

	const patchTask = useCallback(
		async (id: string, patch: TaskPatch) => {
			ownWrite.current = Date.now();
			if (data) {
				setData({ ...data, tasks: data.tasks.map((t) => (t.id === id ? ({ ...t, ...patch } as Task) : t)) });
			}
			try {
				const res = await api<{ task: TaskDetail; warnings: string[] }>(taskPath(id), { method: "PATCH", json: patch });
				for (const w of res.warnings) toast(w, "warn");
				await refresh();
				setRefreshKey((k) => k + 1);
			} catch (e) {
				toast((e as Error).message, "error");
				refresh();
			}
		},
		[data, refresh, toast],
	);

	if (!data) {
		return <div className="splash">{error ? <p className="splash-error">{error}</p> : <p>Lendo .tasks/…</p>}</div>;
	}

	const currentSprint = data.sprints.find((s) => s.id === data.currentSprint);
	const sprintFilter = filters.sprint === "current" ? (data.currentSprint ?? "") : filters.sprint;

	const openTask = (id: string) => {
		setOpenId(id);
	};
	const focusBoard = (patch: Partial<Filters>) => {
		setFilters({ ...EMPTY_FILTERS, sprint: "", ...patch });
		setView("board");
	};

	return (
		<div className="app">
			<header className="topbar">
				<div className="brand">
					<span className="brand-mark" aria-hidden>
						<i />
						<i />
						<i />
					</span>
					<span>
						WingDrive <b>board</b>
					</span>
				</div>
				<nav className="tabs" aria-label="Visões">
					{(
						[
							["board", "Quadro"],
							["sprints", "Sprints"],
							["roadmap", "Roadmap"],
						] as const
					).map(([v, label], i) => (
						<button key={v} className={cx("tab", view === v && "on")} onClick={() => setView(v)} title={`Tecla ${i + 1}`}>
							{label}
						</button>
					))}
				</nav>
				<div className="topbar-end">
					<IssuesButton issues={data.issues} onOpen={openTask} />
					<button className="btn primary" onClick={() => setCreating(true)} title="Tecla N">
						Nova task
					</button>
				</div>
			</header>

			{error && <div className="banner">{error}</div>}

			{currentSprint && (
				<SprintStrip
					sprint={currentSprint}
					tasks={data.tasks.filter((t) => t.sprint === currentSprint.id)}
					today={data.today}
					onOpen={() => focusBoard({ sprint: "current" })}
				/>
			)}

			<main className="main">
				{view === "board" && (
					<BoardView
						data={data}
						filters={filters}
						sprintFilter={sprintFilter}
						setFilters={setFilters}
						onOpen={openTask}
						onMove={(id, status) => patchTask(id, { status })}
						searchRef={search}
					/>
				)}
				{view === "sprints" && (
					<SprintsView data={data} onPick={(id) => focusBoard({ sprint: id === data.currentSprint ? "current" : id })} />
				)}
				{view === "roadmap" && (
					<RoadmapView
						data={data}
						onMilestone={(id) => focusBoard({ milestone: id })}
						onEpic={(id) => focusBoard({ parent: id })}
						onOpen={openTask}
					/>
				)}
			</main>

			{openId && (
				<Drawer
					key={openId}
					id={openId}
					data={data}
					refreshKey={refreshKey}
					onClose={() => setOpenId(undefined)}
					onOpen={openTask}
					onPatch={patchTask}
					onChecked={() => {
						ownWrite.current = Date.now();
					}}
					toast={toast}
				/>
			)}
			{creating && (
				<CreateDialog
					data={data}
					onClose={() => setCreating(false)}
					onCreated={(id) => {
						ownWrite.current = Date.now();
						setCreating(false);
						refresh();
						setOpenId(id);
						toast(`${id} criada`);
					}}
				/>
			)}

			<div className="toasts" role="status" aria-live="polite">
				{toasts.map((t) => (
					<div key={t.id} className={cx("toast", t.tone)}>
						{t.text}
					</div>
				))}
			</div>
		</div>
	);
}

// ---------------------------------------------------------------------------------------------
// Sprint strip (current sprint header)

function SprintStrip({ sprint, tasks, today, onOpen }: { sprint: Sprint; tasks: Task[]; today: string; onOpen: () => void }) {
	const c = count(tasks);
	const wip = tasks.filter((t) => t.status === "In Progress" && !t.isEpic).length;
	const total = sprint.start && sprint.end ? daysBetween(sprint.start, sprint.end) + 1 : 0;
	const elapsed = sprint.start ? Math.min(total, Math.max(0, daysBetween(sprint.start, today) + 1)) : 0;
	const left = total - elapsed;
	return (
		<section className="sprint-strip" aria-label={`Sprint atual ${sprint.id}`}>
			<button className="sprint-id" onClick={onOpen} title="Ver o quadro do sprint">
				{sprint.id}
			</button>
			<div className="sprint-body">
				<div className="sprint-line">
					<span className="sprint-dates">
						{fmtDate(sprint.start)} a {fmtDate(sprint.end)}
					</span>
					<span className="sprint-left">{left > 0 ? `${left} ${left === 1 ? "dia restante" : "dias restantes"}` : "último dia"}</span>
					{sprint.milestones && <span className="sprint-ms">{sprint.milestones}</span>}
				</div>
				{sprint.goal && <p className="sprint-goal">{sprint.goal}</p>}
				<div className="track" aria-hidden>
					<span className="track-done" style={{ width: `${pct(c.done, c.total)}%` }} />
					<span className="track-doing" style={{ width: `${pct(c.doing, c.total)}%` }} />
					<span className="track-time" style={{ left: `${pct(elapsed, total)}%` }} title="Hoje" />
				</div>
			</div>
			<dl className="sprint-stats">
				<div>
					<dt>Done</dt>
					<dd>
						{c.done}
						<small>/{c.total}</small>
					</dd>
				</div>
				<div className={cx(wip > WIP_LIMIT && "over")}>
					<dt>WIP</dt>
					<dd>
						{wip}
						<small>/{WIP_LIMIT}</small>
					</dd>
				</div>
			</dl>
		</section>
	);
}

// ---------------------------------------------------------------------------------------------
// Board view

function BoardView({
	data,
	filters,
	sprintFilter,
	setFilters,
	onOpen,
	onMove,
	searchRef,
}: {
	data: BoardData;
	filters: Filters;
	sprintFilter: string;
	setFilters: (p: Partial<Filters>) => void;
	onOpen: (id: string) => void;
	onMove: (id: string, status: Status) => void;
	searchRef: React.RefObject<HTMLInputElement | null>;
}) {
	const [doneShown, setDoneShown] = useState(DONE_PAGE);
	const [dragging, setDragging] = useState<string>();
	const [over, setOver] = useState<Status>();

	const epics = useMemo(() => data.tasks.filter((t) => t.isEpic).sort((a, b) => a.id.localeCompare(b.id)), [data.tasks]);
	const q = filters.query.trim().toLowerCase();
	const visible = data.tasks.filter(
		(t) =>
			(!sprintFilter || t.sprint === sprintFilter) &&
			(!filters.milestone || t.milestone === filters.milestone) &&
			(!filters.priority || t.priority === filters.priority) &&
			(!filters.area || t.area === filters.area) &&
			(!filters.parent || t.parent === filters.parent || t.id === filters.parent) &&
			(!q ||
				t.id.toLowerCase().includes(q) ||
				t.title.toLowerCase().includes(q) ||
				t.tags.some((tag) => tag.toLowerCase().includes(q))),
	);
	const active = Object.entries(filters).filter(([k, v]) => v && !(k === "sprint" && v === "current")).length;
	const prioRank = { High: 0, Medium: 1, Low: 2 };

	const drop = (status: Status) => (e: DragEvent) => {
		e.preventDefault();
		const id = e.dataTransfer.getData("text/plain");
		setOver(undefined);
		setDragging(undefined);
		const task = data.tasks.find((t) => t.id === id);
		if (task && task.status !== status) onMove(id, status);
	};

	return (
		<>
			<div className="filters" role="search">
				<input
					ref={searchRef}
					className="search"
					type="search"
					placeholder="Buscar id, título ou tag  /"
					value={filters.query}
					onChange={(e) => setFilters({ query: e.target.value })}
				/>
				<Select
					label="Sprint"
					value={filters.sprint}
					onChange={(v) => setFilters({ sprint: v })}
					options={[
						["current", `Atual (${data.currentSprint ?? "nenhum"})`],
						["", "Todos"],
						...data.sprints.map((s) => [s.id, `${s.id} · ${fmtDate(s.start)}`] as [string, string]),
					]}
				/>
				<Select
					label="Milestone"
					value={filters.milestone}
					onChange={(v) => setFilters({ milestone: v })}
					options={[["", "Todos"], ...data.milestones.map((m) => [m.id, `${m.id} ${m.title}`] as [string, string])]}
				/>
				<Select
					label="Épico"
					value={filters.parent}
					onChange={(v) => setFilters({ parent: v })}
					options={[["", "Todos"], ...epics.map((t) => [t.id, `${t.id} ${t.title.replace(/^Epic:\s*/i, "")}`] as [string, string])]}
				/>
				<Select
					label="Prioridade"
					value={filters.priority}
					onChange={(v) => setFilters({ priority: v })}
					options={[["", "Todas"], ...PRIORITIES.map((p) => [p, PRIORITY_LABEL[p]] as [string, string])]}
				/>
				<Select
					label="Área"
					value={filters.area}
					onChange={(v) => setFilters({ area: v })}
					options={[["", "Todas"], ...AREAS.map((a) => [a, a] as [string, string])]}
				/>
				{active > 0 && (
					<button className="btn ghost" onClick={() => setFilters({ ...EMPTY_FILTERS })}>
						Limpar filtros
					</button>
				)}
			</div>

			<div className="columns">
				{STATUSES.map((status) => {
					let list = visible.filter((t) => t.status === status);
					list =
						status === "Done"
							? list.sort((a, b) => (b.lastUpdated ?? "").localeCompare(a.lastUpdated ?? "") || a.id.localeCompare(b.id))
							: list.sort((a, b) => prioRank[a.priority] - prioRank[b.priority] || a.id.localeCompare(b.id, "en", { numeric: true }));
					const shown = status === "Done" ? list.slice(0, doneShown) : list;
					const wip = list.filter((t) => !t.isEpic).length;
					const overLimit = status === "In Progress" && sprintFilter && wip > WIP_LIMIT;
					return (
						<section
							key={status}
							className={cx("column", `col-${status.replace(" ", "").toLowerCase()}`, over === status && "drop")}
							onDragOver={(e) => {
								if (!dragging) return;
								e.preventDefault();
								setOver(status);
							}}
							onDragLeave={(e) => {
								if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) setOver(undefined);
							}}
							onDrop={drop(status)}
							aria-label={status}
						>
							<header className="column-head">
								<span className="column-dot" />
								<h2>{status}</h2>
								<span className="column-hint">{STATUS_HINT[status]}</span>
								<span className={cx("column-count", overLimit && "over")} title={overLimit ? `Limite de WIP: ${WIP_LIMIT}` : undefined}>
									{status === "In Progress" && sprintFilter ? `${wip}/${WIP_LIMIT}` : list.length}
								</span>
							</header>
							<div className="column-body">
								{shown.map((t) => (
									<Strip
										key={t.id}
										task={t}
										today={data.today}
										dragging={dragging === t.id}
										onOpen={() => onOpen(t.id)}
										onDragStart={(e) => {
											e.dataTransfer.setData("text/plain", t.id);
											e.dataTransfer.effectAllowed = "move";
											setDragging(t.id);
										}}
										onDragEnd={() => {
											setDragging(undefined);
											setOver(undefined);
										}}
										onMove={(s) => onMove(t.id, s)}
									/>
								))}
								{list.length === 0 && <p className="empty">{status === "In Progress" ? "Nada em voo. Arraste uma task da fila para começar." : "Vazio com estes filtros."}</p>}
								{list.length > shown.length && (
									<button className="btn ghost more" onClick={() => setDoneShown((n) => n + DONE_PAGE * 2)}>
										Mostrar mais {Math.min(DONE_PAGE * 2, list.length - shown.length)} de {list.length - shown.length}
									</button>
								)}
							</div>
						</section>
					);
				})}
			</div>
		</>
	);
}

function Strip({
	task: t,
	today,
	dragging,
	onOpen,
	onDragStart,
	onDragEnd,
	onMove,
}: {
	task: Task;
	today: string;
	dragging: boolean;
	onOpen: () => void;
	onDragStart: (e: DragEvent) => void;
	onDragEnd: () => void;
	onMove: (s: Status) => void;
}) {
	const progress = t.checklist.total ? pct(t.checklist.done, t.checklist.total) : undefined;
	const idx = STATUSES.indexOf(t.status);
	return (
		<article
			className={cx("strip", `prio-${t.priority.toLowerCase()}`, t.isEpic && "epic", dragging && "dragging")}
			draggable
			onDragStart={onDragStart}
			onDragEnd={onDragEnd}
			onClick={onOpen}
			onKeyDown={(e) => {
				if (e.key === "Enter") onOpen();
				if (e.altKey && e.key === "ArrowRight" && idx < 2) onMove(STATUSES[idx + 1]);
				if (e.altKey && e.key === "ArrowLeft" && idx > 0) onMove(STATUSES[idx - 1]);
			}}
			tabIndex={0}
			aria-label={`${t.id} ${t.title}, prioridade ${PRIORITY_LABEL[t.priority]}. Alt+setas move entre colunas.`}
		>
			<div className="strip-bay" title={`Prioridade ${PRIORITY_LABEL[t.priority]}`} />
			<div className="strip-id">
				<span>{t.id}</span>
				{t.isEpic && <em>épico</em>}
			</div>
			<div className="strip-main">
				<h3>{t.title}</h3>
				<div className="strip-meta">
					{t.milestone && <span className="tag">{t.milestone}</span>}
					{t.sprint && <span className="tag">{t.sprint}</span>}
					{t.parent && <span className="tag parent">↑ {t.parent}</span>}
					<span className="when">{ago(today, t.lastUpdated)}</span>
				</div>
			</div>
			{progress !== undefined && (
				<div className="strip-check" title={`${t.checklist.done} de ${t.checklist.total} itens`}>
					<span>
						{t.checklist.done}/{t.checklist.total}
					</span>
					<b>
						<i style={{ width: `${progress}%` }} />
					</b>
				</div>
			)}
		</article>
	);
}

function Select({
	label,
	value,
	onChange,
	options,
}: {
	label: string;
	value: string;
	onChange: (v: string) => void;
	options: [string, string][];
}) {
	return (
		<label className={cx("select", value && value !== "current" && "set")}>
			<span>{label}</span>
			<select value={value} onChange={(e) => onChange(e.target.value)}>
				{options.map(([v, l]) => (
					<option key={v} value={v}>
						{l}
					</option>
				))}
			</select>
		</label>
	);
}

// ---------------------------------------------------------------------------------------------
// Sprints view

function SprintsView({ data, onPick }: { data: BoardData; onPick: (id: string) => void }) {
	return (
		<ol className="sprints">
			{data.sprints.map((s) => {
				const tasks = data.tasks.filter((t) => t.sprint === s.id);
				const c = count(tasks);
				const past = s.end ? s.end < data.today : false;
				const current = s.id === data.currentSprint;
				return (
					<li key={s.id}>
						<button className={cx("sprint-row", current && "current", past && "past")} onClick={() => onPick(s.id)}>
							<span className="sr-id">{s.id}</span>
							<span className="sr-dates">
								{fmtDate(s.start)} a {fmtDate(s.end)}
								{current && <b>agora</b>}
							</span>
							<span className="sr-goal">{s.goal ?? s.milestones ?? "Sem meta escrita no arquivo do sprint."}</span>
							<span className="sr-bar" aria-label={`${c.done} de ${c.total} Done`}>
								<i className="d" style={{ width: `${pct(c.done, c.total)}%` }} />
								<i className="p" style={{ width: `${pct(c.doing, c.total)}%` }} />
							</span>
							<span className="sr-count">
								{c.done}/{c.total}
								{c.doing > 0 && <small>{c.doing} em voo</small>}
							</span>
						</button>
					</li>
				);
			})}
		</ol>
	);
}

// ---------------------------------------------------------------------------------------------
// Roadmap view

function RoadmapView({
	data,
	onMilestone,
	onEpic,
	onOpen,
}: {
	data: BoardData;
	onMilestone: (id: string) => void;
	onEpic: (id: string) => void;
	onOpen: (id: string) => void;
}) {
	const order: Record<Milestone["phase"], number> = { Now: 0, Next: 1, Later: 2, Done: 3 };
	const milestones = [...data.milestones].sort((a, b) => order[a.phase] - order[b.phase] || a.id.localeCompare(b.id));
	const [showDone, setShowDone] = useState(false);
	return (
		<div className="roadmap">
			{milestones
				.filter((m) => showDone || m.phase !== "Done")
				.map((m) => {
					const tasks = data.tasks.filter((t) => t.milestone === m.id);
					const c = count(tasks);
					const epics = tasks.filter((t) => t.isEpic);
					const loose = data.tasks.filter((t) => t.milestone === m.id && t.status !== "Done" && !t.isEpic && !t.parent).length;
					return (
						<section key={m.id} className={cx("milestone", `phase-${m.phase.toLowerCase()}`)}>
							<header>
								<button className="ms-title" onClick={() => onMilestone(m.id)}>
									<span className="ms-id">{m.id}</span>
									<span>{m.title}</span>
								</button>
								<span className="ms-phase">{m.phase}</span>
							</header>
							<div className="ms-progress">
								<span className="big">{pct(c.done, c.total)}%</span>
								<span className="ms-bar">
									<i className="d" style={{ width: `${pct(c.done, c.total)}%` }} />
									<i className="p" style={{ width: `${pct(c.doing, c.total)}%` }} />
								</span>
								<span className="ms-counts">
									{c.done} Done · {c.doing} In Progress · {c.todo} To Do
								</span>
							</div>
							{epics.length > 0 && (
								<ul className="epics">
									{epics.map((e) => {
										const kids = data.tasks.filter((t) => t.parent === e.id);
										const k = count(kids);
										return (
											<li key={e.id}>
												<button className="epic-row" onClick={() => onEpic(e.id)} title="Ver as tasks do épico no quadro">
													<span className="er-id">{e.id}</span>
													<span className="er-title">{e.title.replace(/^Epic:\s*/i, "")}</span>
													<span className="er-bar">
														<i style={{ width: `${pct(k.done, k.total)}%` }} />
													</span>
													<span className="er-count">
														{k.done}/{k.total}
													</span>
												</button>
												<button className="er-open" onClick={() => onOpen(e.id)} aria-label={`Abrir ${e.id}`}>
													Abrir
												</button>
											</li>
										);
									})}
								</ul>
							)}
							{loose > 0 && <p className="ms-loose">{loose === 1 ? "1 task aberta sem épico." : `${loose} tasks abertas sem épico.`}</p>}
						</section>
					);
				})}
			<button className="btn ghost" onClick={() => setShowDone((s) => !s)}>
				{showDone ? "Esconder milestones concluídos" : `Mostrar milestones concluídos (${data.milestones.filter((m) => m.phase === "Done").length})`}
			</button>
		</div>
	);
}

// ---------------------------------------------------------------------------------------------
// Drawer (task detail)

function Drawer({
	id,
	data,
	refreshKey,
	onClose,
	onOpen,
	onPatch,
	onChecked,
	toast,
}: {
	id: string;
	data: BoardData;
	refreshKey: number;
	onClose: () => void;
	onOpen: (id: string) => void;
	onPatch: (id: string, patch: TaskPatch) => Promise<void>;
	onChecked: () => void;
	toast: (t: string, tone?: "warn" | "error") => void;
}) {
	const [task, setTask] = useState<TaskDetail>();
	const [missing, setMissing] = useState<string>();
	const bodyRef = useRef<HTMLDivElement>(null);

	const load = useCallback(() => {
		api<TaskDetail>(taskPath(id))
			.then((t) => {
				setTask(t);
				setMissing(undefined);
			})
			.catch((e) => setMissing((e as Error).message));
	}, [id]);
	useEffect(load, [load, refreshKey]);

	const html = useMemo(() => (task ? (marked.parse(task.body, { async: false }) as string) : ""), [task]);

	useEffect(() => {
		const el = bodyRef.current;
		if (!el || !task) return;
		const boxes = [...el.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')];
		const interactive = boxes.length === task.items.length;
		boxes.forEach((box, i) => {
			box.disabled = !interactive;
			box.dataset.index = String(i);
			box.closest("li")?.classList.add("task-item");
		});
	}, [html, task]);

	const onBodyClick = async (e: React.MouseEvent) => {
		const box = e.target as HTMLInputElement;
		if (box.type !== "checkbox" || box.dataset.index === undefined || !task) return;
		const index = Number(box.dataset.index);
		const done = box.checked;
		onChecked();
		try {
			const res = await api<{ task: TaskDetail }>(taskPath(task.id, "/checklist"), { method: "POST", json: { index, done } });
			setTask(res.task);
		} catch (err) {
			box.checked = !done;
			toast((err as Error).message, "error");
		}
	};

	const children = task ? data.tasks.filter((t) => t.parent === task.id) : [];
	const parent = task?.parent ? data.tasks.find((t) => t.id === task.parent) : undefined;
	const patch = async (p: TaskPatch) => {
		if (!task) return;
		await onPatch(task.id, p);
		load();
	};

	return (
		<>
			<div className="scrim" onClick={onClose} />
			<aside className="drawer" role="dialog" aria-modal="true" aria-label={task ? `${task.id} ${task.title}` : id}>
				{missing && (
					<div className="drawer-pad">
						<p className="splash-error">{missing}</p>
						<button className="btn" onClick={onClose}>
							Fechar
						</button>
					</div>
				)}
				{task && (
					<>
						<header className={cx("drawer-head", `prio-${task.priority.toLowerCase()}`)}>
							<div className="drawer-id">
								<span>{task.id}</span>
								<button
									className="btn ghost small"
									onClick={() => {
										navigator.clipboard?.writeText(task.file).then(() => toast(`Caminho copiado: ${task.file}`));
									}}
									title={task.file}
								>
									Copiar caminho
								</button>
								<button className="btn ghost small close" onClick={onClose} aria-label="Fechar">
									Esc
								</button>
							</div>
							<h2>{task.title}</h2>
						</header>
						<div className="drawer-scroll">
							<div className="fields">
								<Field label="Status">
									<select value={task.status} onChange={(e) => patch({ status: e.target.value as Status })}>
										{STATUSES.map((s) => (
											<option key={s}>{s}</option>
										))}
									</select>
								</Field>
								<Field label="Prioridade">
									<select value={task.priority} onChange={(e) => patch({ priority: e.target.value as Priority })}>
										{PRIORITIES.map((p) => (
											<option key={p} value={p}>
												{PRIORITY_LABEL[p]}
											</option>
										))}
									</select>
								</Field>
								<Field label="Sprint">
									<select value={task.sprint ?? ""} onChange={(e) => patch({ sprint: e.target.value || null })}>
										<option value="">Backlog</option>
										{data.sprints.map((s) => (
											<option key={s.id} value={s.id}>
												{s.id} {s.id === data.currentSprint ? "(atual)" : `· ${fmtDate(s.start)}`}
											</option>
										))}
									</select>
								</Field>
								<Field label="Milestone">
									<select value={task.milestone ?? ""} onChange={(e) => e.target.value && patch({ milestone: e.target.value })}>
										{!task.milestone && <option value="">—</option>}
										{data.milestones.map((m) => (
											<option key={m.id} value={m.id}>
												{m.id} {m.title}
											</option>
										))}
									</select>
								</Field>
							</div>

							<dl className="facts">
								{parent && (
									<>
										<dt>Épico</dt>
										<dd>
											<LinkTask task={parent} onOpen={onOpen} />
										</dd>
									</>
								)}
								{task.related.length > 0 && (
									<>
										<dt>Relacionadas</dt>
										<dd className="links">
											{task.related.map((r) => {
												const rt = data.tasks.find((t) => t.id === r);
												return rt ? <LinkTask key={r} task={rt} onOpen={onOpen} compact /> : <span key={r}>{r}</span>;
											})}
										</dd>
									</>
								)}
								{task.tags.length > 0 && (
									<>
										<dt>Tags</dt>
										<dd className="links">
											{task.tags.map((t) => (
												<span className="tag" key={t}>
													{t}
												</span>
											))}
										</dd>
									</>
								)}
								<dt>Atualizada</dt>
								<dd>
									{task.lastUpdated ?? "sem data"} ({ago(data.today, task.lastUpdated)})
								</dd>
							</dl>

							{children.length > 0 && (
								<section className="children">
									<h3>
										Tasks do épico <small>{count(children).done}/{children.length} Done</small>
									</h3>
									<ul>
										{children
											.sort((a, b) => STATUSES.indexOf(a.status) - STATUSES.indexOf(b.status) || a.id.localeCompare(b.id, "en", { numeric: true }))
											.map((c) => (
												<li key={c.id}>
													<LinkTask task={c} onOpen={onOpen} />
												</li>
											))}
									</ul>
								</section>
							)}

							{task.checklist.total > 0 && (
								<div className="checkline">
									<span>
										Checklist {task.checklist.done}/{task.checklist.total}
									</span>
									<i>
										<b style={{ width: `${pct(task.checklist.done, task.checklist.total)}%` }} />
									</i>
								</div>
							)}
							<div className="md" ref={bodyRef} onClick={onBodyClick} dangerouslySetInnerHTML={{ __html: html }} />
						</div>
					</>
				)}
			</aside>
		</>
	);
}

function Field({ label, children }: { label: string; children: ReactNode }) {
	return (
		<label className="field">
			<span>{label}</span>
			{children}
		</label>
	);
}

function LinkTask({ task, onOpen, compact }: { task: Task; onOpen: (id: string) => void; compact?: boolean }) {
	return (
		<button className={cx("link-task", `st-${task.status.replace(" ", "").toLowerCase()}`)} onClick={() => onOpen(task.id)}>
			<i />
			<span className="lt-id">{task.id}</span>
			{!compact && <span className="lt-title">{task.title}</span>}
		</button>
	);
}

// ---------------------------------------------------------------------------------------------
// Issues popover

function IssuesButton({ issues, onOpen }: { issues: Issue[]; onOpen: (id: string) => void }) {
	const [open, setOpen] = useState(false);
	const errors = issues.filter((i) => i.level === "error").length;
	const warns = issues.length - errors;
	if (!issues.length) return <span className="health ok">Sem inconsistências</span>;
	return (
		<div className="issues">
			<button className={cx("health", errors ? "bad" : "warn")} onClick={() => setOpen((o) => !o)} aria-expanded={open}>
				{errors > 0 && `${errors} ${errors === 1 ? "erro" : "erros"}`}
				{errors > 0 && warns > 0 && ", "}
				{warns > 0 && `${warns} ${warns === 1 ? "aviso" : "avisos"}`}
			</button>
			{open && (
				<div className="issues-pop">
					<p>Inconsistências entre as tasks, os sprints e o ROADMAP. Os agentes veem a mesma lista pela ferramenta `check` do MCP.</p>
					<ul>
						{issues.map((i, n) => (
							<li key={n} className={i.level}>
								{i.id ? (
									<button
										className="lt-id"
										onClick={() => {
											onOpen(i.id!);
											setOpen(false);
										}}
									>
										{i.id}
									</button>
								) : null}{" "}
								{i.message}
							</li>
						))}
					</ul>
				</div>
			)}
		</div>
	);
}

// ---------------------------------------------------------------------------------------------
// Create dialog

function CreateDialog({ data, onClose, onCreated }: { data: BoardData; onClose: () => void; onCreated: (id: string) => void }) {
	const prefixes = useMemo(() => [...new Set(data.tasks.map((t) => t.id.split("-")[0]))].sort(), [data.tasks]);
	const now = data.milestones.find((m) => m.phase === "Now")?.id ?? data.milestones[0]?.id ?? "";
	const [form, setForm] = useState({
		prefix: "CORE",
		title: "",
		area: "core",
		priority: "Medium",
		milestone: now,
		sprint: data.currentSprint ?? "",
		parent: "",
		description: "",
		criteria: "",
	});
	const [err, setErr] = useState<string>();
	const [busy, setBusy] = useState(false);
	const set = (k: keyof typeof form) => (e: { target: { value: string } }) => setForm((f) => ({ ...f, [k]: e.target.value }));
	const submit = async (e: React.FormEvent) => {
		e.preventDefault();
		setBusy(true);
		try {
			const res = await api<{ task: TaskDetail }>("/api/tasks", {
				method: "POST",
				json: {
					...form,
					prefix: form.prefix.toUpperCase(),
					sprint: form.sprint || undefined,
					parent: form.parent || undefined,
					criteria: form.criteria
						.split("\n")
						.map((s) => s.replace(/^[-*]\s*(\[.\])?\s*/, "").trim())
						.filter(Boolean),
				},
			});
			onCreated(res.task.id);
		} catch (e2) {
			setErr((e2 as Error).message);
			setBusy(false);
		}
	};
	return (
		<>
			<div className="scrim" onClick={onClose} />
			<form className="dialog" onSubmit={submit} aria-label="Nova task">
				<h2>Nova task</h2>
				<p className="dialog-note">Cria o arquivo em .tasks/ com o próximo id livre do prefixo.</p>
				<div className="grid2">
					<Field label="Prefixo">
						<input list="prefixes" value={form.prefix} onChange={set("prefix")} required pattern="[A-Za-z]+" />
						<datalist id="prefixes">
							{prefixes.map((p) => (
								<option key={p} value={p} />
							))}
						</datalist>
					</Field>
					<Field label="Área">
						<select value={form.area} onChange={set("area")}>
							{AREAS.map((a) => (
								<option key={a}>{a}</option>
							))}
						</select>
					</Field>
				</div>
				<Field label="Título">
					<input value={form.title} onChange={set("title")} required minLength={5} autoFocus />
				</Field>
				<div className="grid3">
					<Field label="Milestone">
						<select value={form.milestone} onChange={set("milestone")}>
							{data.milestones.map((m) => (
								<option key={m.id} value={m.id}>
									{m.id} {m.title}
								</option>
							))}
						</select>
					</Field>
					<Field label="Sprint">
						<select value={form.sprint} onChange={set("sprint")}>
							<option value="">Backlog</option>
							{data.sprints.map((s) => (
								<option key={s.id} value={s.id}>
									{s.id}
								</option>
							))}
						</select>
					</Field>
					<Field label="Prioridade">
						<select value={form.priority} onChange={set("priority")}>
							{PRIORITIES.map((p) => (
								<option key={p} value={p}>
									{PRIORITY_LABEL[p]}
								</option>
							))}
						</select>
					</Field>
				</div>
				<Field label="Épico">
					<select value={form.parent} onChange={set("parent")}>
						<option value="">Nenhum</option>
						{data.tasks
							.filter((t) => t.isEpic && t.status !== "Done")
							.map((t) => (
								<option key={t.id} value={t.id}>
									{t.id} {t.title.replace(/^Epic:\s*/i, "")}
								</option>
							))}
					</select>
				</Field>
				<Field label="Descrição">
					<textarea rows={3} value={form.description} onChange={set("description")} />
				</Field>
				<Field label="Critérios de aceite, um por linha">
					<textarea rows={3} value={form.criteria} onChange={set("criteria")} />
				</Field>
				{err && <p className="splash-error">{err}</p>}
				<div className="dialog-actions">
					<button type="button" className="btn ghost" onClick={onClose}>
						Cancelar
					</button>
					<button className="btn primary" disabled={busy}>
						Criar task
					</button>
				</div>
			</form>
		</>
	);
}

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<App />
	</StrictMode>,
);
