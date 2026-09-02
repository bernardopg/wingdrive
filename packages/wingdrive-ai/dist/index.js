// src/types.ts
var TASK_STATUS_ORDER = [
  "in_progress",
  "ready",
  "pending_approval",
  "backlog",
  "done"
];
var TASK_STATUS_LABEL = {
  in_progress: "In Progress",
  ready: "Ready",
  pending_approval: "Pending Approval",
  backlog: "Backlog",
  done: "Done"
};
var TASK_PRIORITY_LABEL = {
  critical: "Critical",
  high: "High",
  medium: "Medium",
  low: "Low"
};
function tryParseJson(text) {
  if (!text || text.trim().length === 0) return null;
  try {
    const parsed = JSON.parse(text);
    if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
      return parsed;
    }
    return null;
  } catch {
    return null;
  }
}
function isErrorResult(text, parsed) {
  if (parsed?.error) return true;
  if (parsed?.status === "error") return true;
  if (parsed?.success === false) return true;
  if (typeof parsed?.exit_code === "number" && parsed.exit_code !== 0) return true;
  const lower = text.toLowerCase();
  return lower.startsWith("error:") || lower.startsWith("error -") || lower.startsWith("failed:") || lower.startsWith("toolset error:");
}
function pairTranscriptSteps(steps) {
  const items = [];
  const resultsById = /* @__PURE__ */ new Map();
  for (const step of steps) {
    if (step.type === "tool_result") {
      resultsById.set(step.call_id, { name: step.name, text: step.text });
    }
  }
  for (const step of steps) {
    if (step.type === "action") {
      for (const content of step.content) {
        if (content.type === "text") {
          items.push({ kind: "text", text: content.text });
        } else if (content.type === "tool_call") {
          const result = resultsById.get(content.id);
          const parsedArgs = tryParseJson(content.args);
          const parsedResult = result ? tryParseJson(result.text) : null;
          const hasError = result ? isErrorResult(result.text, parsedResult) : false;
          items.push({
            kind: "tool",
            pair: {
              id: content.id,
              name: content.name,
              argsRaw: content.args,
              args: parsedArgs,
              resultRaw: result?.text ?? null,
              result: parsedResult,
              status: result ? hasError ? "error" : "completed" : "running"
            }
          });
        }
      }
    }
  }
  return items;
}

// src/ToolCall.tsx
import { useState } from "react";
import { clsx } from "clsx";
import { jsx, jsxs } from "react/jsx-runtime";
var toolRenderers = {
  browser_launch: {
    summary(pair) {
      const headless = pair.args?.headless;
      return headless === false ? "Launch browser (visible)" : "Launch browser";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: "Browser session started" });
    }
  },
  browser_navigate: {
    summary(pair) {
      const url = pair.args?.url;
      return url ? truncate(String(url), 60) : null;
    },
    resultView(pair) {
      if (!pair.result) return /* @__PURE__ */ jsx(ResultText, { text: pair.resultRaw });
      const title = pair.result.title;
      const url = pair.result.url;
      return /* @__PURE__ */ jsxs("div", { className: "flex flex-col gap-1 px-3 py-2", children: [
        title && /* @__PURE__ */ jsxs("p", { className: "text-[11px] text-ink-dull", children: [
          /* @__PURE__ */ jsx("span", { className: "text-ink-faint", children: "Title: " }),
          title
        ] }),
        url && /* @__PURE__ */ jsx("p", { className: "font-mono text-[11px] text-ink-faint", children: truncate(url, 80) })
      ] });
    }
  },
  browser_snapshot: {
    summary(pair) {
      if (!pair.resultRaw) return "Taking snapshot...";
      const matches = pair.resultRaw.match(/\[\d+\]/g);
      const count = matches?.length ?? 0;
      return count > 0 ? `${count} interactive element${count !== 1 ? "s" : ""}` : "Page snapshot";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  },
  browser_click: {
    summary(pair) {
      const index = pair.args?.index;
      return index !== void 0 ? `Click element [${index}]` : "Click";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: pair.resultRaw });
    }
  },
  browser_type: {
    summary(pair) {
      const index = pair.args?.index;
      const hasSecret = pair.args?.secret !== void 0;
      const text = pair.args?.text;
      if (hasSecret) {
        return index !== void 0 ? `Type secret into [${index}]` : "Type secret";
      }
      if (text) {
        const display = truncate(String(text), 30);
        return index !== void 0 ? `Type "${display}" into [${index}]` : `Type "${display}"`;
      }
      return index !== void 0 ? `Type into [${index}]` : "Type";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: pair.resultRaw });
    }
  },
  browser_press_key: {
    summary(pair) {
      const key = pair.args?.key;
      return key ? `Press ${key}` : "Press key";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: pair.resultRaw });
    }
  },
  browser_screenshot: {
    summary() {
      return "Capture screenshot";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      if (pair.result?.base64) {
        const mimeType = pair.result.mime_type ?? "image/png";
        return /* @__PURE__ */ jsx("div", { className: "px-3 py-2", children: /* @__PURE__ */ jsx(
          "img",
          {
            src: `data:${mimeType};base64,${pair.result.base64}`,
            alt: "Browser screenshot",
            className: "max-h-60 rounded border border-app-line/30 object-contain"
          }
        ) });
      }
      return /* @__PURE__ */ jsx(ResultLine, { text: truncate(pair.resultRaw, 100) });
    }
  },
  browser_evaluate: {
    summary(pair) {
      const expression = pair.args?.expression;
      return expression ? truncate(String(expression), 50) : "Evaluate JS";
    },
    argsView(pair) {
      const expression = pair.args?.expression;
      if (!expression) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        /* @__PURE__ */ jsx("p", { className: "mb-1 text-[11px] font-medium text-ink-faint", children: "JavaScript" }),
        /* @__PURE__ */ jsx("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-dull", children: String(expression) })
      ] });
    }
  },
  browser_tab_open: {
    summary(pair) {
      const url = pair.args?.url;
      return url ? `Open tab: ${truncate(String(url), 50)}` : "Open new tab";
    }
  },
  browser_tab_list: {
    summary() {
      return "List tabs";
    }
  },
  browser_tab_close: {
    summary(pair) {
      const tabId = pair.args?.tab_id;
      return tabId !== void 0 ? `Close tab ${tabId}` : "Close tab";
    }
  },
  browser_close: {
    summary() {
      return "Close browser";
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: "Browser session closed" });
    }
  },
  shell: {
    summary(pair) {
      const command = pair.args?.command;
      if (!command) return null;
      if (pair.result && typeof pair.result.exit_code === "number") {
        const code = pair.result.exit_code;
        const cmdStr = truncate(String(command), 50);
        return code === 0 ? cmdStr : `${cmdStr} (exit ${code})`;
      }
      return truncate(String(command), 60);
    },
    argsView(pair) {
      const command = pair.args?.command;
      if (!command) return null;
      return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsxs("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-dull", children: [
        /* @__PURE__ */ jsx("span", { className: "select-none text-ink-faint", children: "$ " }),
        String(command)
      ] }) });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ShellResultView, { pair });
    }
  },
  file_read: {
    summary(pair) {
      if (pair.title) return pair.title;
      const path = pair.args?.path;
      return path ? truncate(String(path), 60) : null;
    },
    argsView(pair) {
      const path = pair.args?.path;
      if (!path) return null;
      const offset = pair.args?.offset;
      const limit = pair.args?.limit;
      return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsxs("p", { className: "font-mono text-[11px] text-ink-dull", children: [
        String(path),
        offset ? ` (from line ${offset})` : "",
        limit ? ` (${limit} lines)` : ""
      ] }) });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
    }
  },
  file_write: {
    summary(pair) {
      if (pair.title) return pair.title;
      const path = pair.args?.path;
      return path ? truncate(String(path), 60) : null;
    },
    argsView(pair) {
      const path = pair.args?.path;
      const content = pair.args?.content;
      if (!path && !content) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        !!path && /* @__PURE__ */ jsx("p", { className: "mb-1 font-mono text-[11px] text-ink-dull", children: String(path) }),
        !!content && /* @__PURE__ */ jsx("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-faint", children: truncate(String(content), 2e3) })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: truncate(pair.resultRaw, 100) });
    }
  },
  file_edit: {
    summary(pair) {
      if (pair.title) return pair.title;
      const path = pair.args?.path;
      return path ? truncate(String(path), 60) : null;
    },
    argsView(pair) {
      const path = pair.args?.path;
      const oldStr = pair.args?.old_string;
      const newStr = pair.args?.new_string;
      if (!path) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        /* @__PURE__ */ jsx("p", { className: "mb-1 font-mono text-[11px] text-ink-dull", children: String(path) }),
        !!oldStr && /* @__PURE__ */ jsxs("div", { className: "mt-1", children: [
          /* @__PURE__ */ jsx("p", { className: "text-[11px] font-medium text-red-400/70", children: "Old" }),
          /* @__PURE__ */ jsx("pre", { className: "max-h-20 overflow-auto font-mono text-[11px] text-red-300/60", children: truncate(String(oldStr), 500) })
        ] }),
        !!newStr && /* @__PURE__ */ jsxs("div", { className: "mt-1", children: [
          /* @__PURE__ */ jsx("p", { className: "text-[11px] font-medium text-emerald-400/70", children: "New" }),
          /* @__PURE__ */ jsx("pre", { className: "max-h-20 overflow-auto font-mono text-[11px] text-emerald-300/60", children: truncate(String(newStr), 500) })
        ] })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: truncate(pair.resultRaw, 100) });
    }
  },
  file_list: {
    summary(pair) {
      if (pair.title) return pair.title;
      const path = pair.args?.path;
      return path ? truncate(String(path), 60) : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
    }
  },
  exec: {
    summary(pair) {
      const program = pair.args?.program;
      const cmdArgs = pair.args?.args;
      if (!program) return null;
      const parts = [String(program)];
      if (Array.isArray(cmdArgs)) {
        for (const arg of cmdArgs) parts.push(String(arg));
      }
      const full = parts.join(" ");
      if (pair.result && typeof pair.result.exit_code === "number") {
        const code = pair.result.exit_code;
        const cmdStr = truncate(full, 50);
        return code === 0 ? cmdStr : `${cmdStr} (exit ${code})`;
      }
      return truncate(full, 60);
    },
    argsView(pair) {
      const program = pair.args?.program;
      if (!program) return null;
      const parts = [String(program)];
      const cmdArgs = pair.args?.args;
      if (Array.isArray(cmdArgs)) {
        for (const arg of cmdArgs) parts.push(String(arg));
      }
      return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsxs("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-dull", children: [
        /* @__PURE__ */ jsx("span", { className: "select-none text-ink-faint", children: "$ " }),
        parts.join(" ")
      ] }) });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ShellResultView, { pair });
    }
  },
  set_status: {
    summary(pair) {
      const kind = pair.args?.kind;
      const message = pair.args?.message;
      if (kind === "outcome") {
        return message ? `Outcome: ${truncate(String(message), 50)}` : "Outcome set";
      }
      return message ? truncate(String(message), 60) : null;
    },
    resultView() {
      return null;
    }
  },
  // OpenCode tools
  read: {
    summary(pair) {
      if (pair.title) return pair.title;
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      return filePath ? truncate(String(filePath), 60) : null;
    },
    argsView(pair) {
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      if (!filePath) return null;
      const offset = pair.args?.offset;
      const limit = pair.args?.limit;
      return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsxs("p", { className: "font-mono text-[11px] text-ink-dull", children: [
        String(filePath),
        offset ? ` (from line ${offset})` : "",
        limit ? ` (${limit} lines)` : ""
      ] }) });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
    }
  },
  write: {
    summary(pair) {
      if (pair.title) return pair.title;
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      return filePath ? truncate(String(filePath), 60) : null;
    },
    argsView(pair) {
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      const content = pair.args?.content;
      if (!filePath && !content) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        !!filePath && /* @__PURE__ */ jsx("p", { className: "mb-1 font-mono text-[11px] text-ink-dull", children: String(filePath) }),
        !!content && /* @__PURE__ */ jsx("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-faint", children: truncate(String(content), 2e3) })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: truncate(pair.resultRaw, 100) });
    }
  },
  edit: {
    summary(pair) {
      if (pair.title) return pair.title;
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      return filePath ? truncate(String(filePath), 60) : null;
    },
    argsView(pair) {
      const filePath = pair.args?.filePath ?? pair.args?.file_path;
      const oldStr = pair.args?.oldString ?? pair.args?.old_string;
      const newStr = pair.args?.newString ?? pair.args?.new_string;
      if (!filePath) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        /* @__PURE__ */ jsx("p", { className: "mb-1 font-mono text-[11px] text-ink-dull", children: String(filePath) }),
        !!oldStr && /* @__PURE__ */ jsxs("div", { className: "mt-1", children: [
          /* @__PURE__ */ jsx("p", { className: "text-[11px] font-medium text-red-400/70", children: "Old" }),
          /* @__PURE__ */ jsx("pre", { className: "max-h-20 overflow-auto font-mono text-[11px] text-red-300/60", children: truncate(String(oldStr), 500) })
        ] }),
        !!newStr && /* @__PURE__ */ jsxs("div", { className: "mt-1", children: [
          /* @__PURE__ */ jsx("p", { className: "text-[11px] font-medium text-emerald-400/70", children: "New" }),
          /* @__PURE__ */ jsx("pre", { className: "max-h-20 overflow-auto font-mono text-[11px] text-emerald-300/60", children: truncate(String(newStr), 500) })
        ] })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ResultLine, { text: truncate(pair.resultRaw, 100) });
    }
  },
  bash: {
    summary(pair) {
      if (pair.title) return pair.title;
      const command = pair.args?.command;
      if (!command) return null;
      if (pair.result && typeof pair.result.exit_code === "number") {
        const code = pair.result.exit_code;
        const cmdStr = truncate(String(command), 50);
        return code === 0 ? cmdStr : `${cmdStr} (exit ${code})`;
      }
      return truncate(String(command), 60);
    },
    argsView(pair) {
      const command = pair.args?.command;
      if (!command) return null;
      return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsxs("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-dull", children: [
        /* @__PURE__ */ jsx("span", { className: "select-none text-ink-faint", children: "$ " }),
        String(command)
      ] }) });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(ShellResultView, { pair });
    }
  },
  glob: {
    summary(pair) {
      if (pair.title) return pair.title;
      const pattern = pair.args?.pattern;
      return pattern ? truncate(String(pattern), 60) : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  },
  grep: {
    summary(pair) {
      if (pair.title) return pair.title;
      const pattern = pair.args?.pattern;
      const include = pair.args?.include;
      if (pattern && include) {
        return `/${pattern}/ in ${include}`;
      }
      return pattern ? `/${truncate(String(pattern), 40)}/` : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  },
  webfetch: {
    summary(pair) {
      if (pair.title) return pair.title;
      const url = pair.args?.url;
      return url ? truncate(String(url), 60) : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  },
  read_skill: {
    summary(pair) {
      if (pair.title) return pair.title;
      const name = pair.args?.name;
      return name ? String(name) : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
    }
  },
  web_search: {
    summary(pair) {
      if (pair.title) return pair.title;
      const query = pair.args?.query;
      const resultCount = pair.result?.result_count ?? (Array.isArray(pair.result?.results) ? pair.result.results.length : null);
      const queryStr = query ? truncate(String(query), 50) : null;
      if (queryStr && resultCount != null) {
        return `${queryStr} (${resultCount} result${resultCount !== 1 ? "s" : ""})`;
      }
      return queryStr;
    },
    argsView(pair) {
      const query = pair.args?.query;
      if (!query) return null;
      const count = pair.args?.count;
      const freshness = pair.args?.freshness;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        /* @__PURE__ */ jsxs("p", { className: "text-[11px] text-ink-dull", children: [
          /* @__PURE__ */ jsx("span", { className: "select-none text-ink-faint", children: "Search: " }),
          String(query)
        ] }),
        !!(count || freshness) && /* @__PURE__ */ jsxs("p", { className: "mt-0.5 text-[11px] text-ink-faint", children: [
          count ? `${count} results` : "",
          count && freshness ? " \xB7 " : "",
          freshness ? `${freshness}` : ""
        ] })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  },
  spacebot_docs: {
    summary(pair) {
      if (pair.title) return pair.title;
      const action = pair.args?.action;
      const docId = pair.args?.doc_id;
      if (action === "read" && docId) {
        return truncate(String(docId), 50);
      }
      return action ? String(action) : "list";
    },
    argsView(pair) {
      const action = pair.args?.action;
      const docId = pair.args?.doc_id;
      const query = pair.args?.query;
      if (!action && !docId) return null;
      return /* @__PURE__ */ jsxs("div", { className: "border-b border-app-line/20 px-3 py-2", children: [
        !!docId && /* @__PURE__ */ jsx("p", { className: "font-mono text-[11px] text-ink-dull", children: String(docId) }),
        !!query && /* @__PURE__ */ jsxs("p", { className: "mt-0.5 text-[11px] text-ink-faint", children: [
          "filter: ",
          String(query)
        ] })
      ] });
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
    }
  },
  todowrite: {
    summary(pair) {
      if (pair.title) return pair.title;
      return "Update tasks";
    },
    resultView() {
      return null;
    }
  },
  task: {
    summary(pair) {
      if (pair.title) return pair.title;
      const description = pair.args?.description;
      return description ? truncate(String(description), 60) : null;
    },
    resultView(pair) {
      if (!pair.resultRaw) return null;
      return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
    }
  }
};
var defaultRenderer = {
  summary(pair) {
    if (!pair.argsRaw || pair.argsRaw === "{}") return null;
    return truncate(pair.argsRaw, 60);
  }
};
function getRenderer(name) {
  return toolRenderers[name] ?? defaultRenderer;
}
function ResultLine({ text }) {
  return /* @__PURE__ */ jsx("p", { className: "px-3 py-2 text-[11px] text-ink-dull", children: text });
}
function ResultText({ text }) {
  if (!text) return null;
  return /* @__PURE__ */ jsx("pre", { className: "max-h-60 overflow-auto whitespace-pre-wrap px-3 py-2 font-mono text-[11px] text-ink-dull", children: text });
}
function CollapsiblePre({
  text,
  maxLines = 20
}) {
  const [expanded, setExpanded] = useState(false);
  const lines = text.split("\n");
  const needsCollapse = lines.length > maxLines;
  const displayText = needsCollapse && !expanded ? lines.slice(0, maxLines).join("\n") + "\n..." : text;
  return /* @__PURE__ */ jsxs("div", { children: [
    /* @__PURE__ */ jsx("pre", { className: "max-h-80 overflow-auto whitespace-pre-wrap px-3 py-2 font-mono text-[11px] text-ink-dull", children: displayText }),
    needsCollapse && /* @__PURE__ */ jsx(
      "button",
      {
        onClick: () => setExpanded(!expanded),
        className: "w-full border-t border-app-line/20 px-3 py-1 text-center text-[11px] text-ink-faint hover:text-ink-dull",
        children: expanded ? "Show less" : `Show all ${lines.length} lines`
      }
    )
  ] });
}
function ShellResultView({ pair }) {
  const r = pair.result;
  if (!r || typeof r.exit_code !== "number") {
    return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 30 });
  }
  const exitCode = r.exit_code;
  const stdout = typeof r.stdout === "string" ? r.stdout : "";
  const stderr = typeof r.stderr === "string" ? r.stderr : "";
  const hasStdout = stdout.trim().length > 0;
  const hasStderr = stderr.trim().length > 0;
  const isError = exitCode !== 0;
  if (!hasStdout && !hasStderr && exitCode === 0) {
    return /* @__PURE__ */ jsx(ResultLine, { text: "Completed with no output" });
  }
  return /* @__PURE__ */ jsxs("div", { className: "flex flex-col", children: [
    isError && /* @__PURE__ */ jsx("div", { className: "flex items-center gap-1.5 border-b border-app-line/20 px-3 py-1.5", children: /* @__PURE__ */ jsxs("span", { className: "rounded bg-red-500/15 px-1.5 py-0.5 font-mono text-[11px] font-medium text-red-400", children: [
      "exit ",
      exitCode
    ] }) }),
    hasStdout && /* @__PURE__ */ jsx("div", { className: hasStderr ? "border-b border-app-line/20" : "", children: /* @__PURE__ */ jsx(CollapsiblePre, { text: stdout.replace(/\n$/, ""), maxLines: 30 }) }),
    hasStderr && /* @__PURE__ */ jsxs("div", { children: [
      /* @__PURE__ */ jsx("div", { className: "flex items-center gap-1.5 border-b border-app-line/10 px-3 pt-1.5 pb-1", children: /* @__PURE__ */ jsx(
        "span",
        {
          className: clsx(
            "text-[11px] font-medium",
            isError ? "text-red-400/70" : "text-yellow-500/70"
          ),
          children: "stderr"
        }
      ) }),
      /* @__PURE__ */ jsx(
        "pre",
        {
          className: clsx(
            "max-h-40 overflow-auto whitespace-pre-wrap px-3 py-2 font-mono text-[11px]",
            isError ? "text-red-300/60" : "text-yellow-300/50"
          ),
          children: stderr.replace(/\n$/, "")
        }
      )
    ] })
  ] });
}
var STATUS_ICONS = {
  running: "\u25B6",
  completed: "\u2713",
  error: "\u2717"
};
var STATUS_COLORS = {
  running: "text-accent",
  completed: "text-status-success",
  error: "text-status-error"
};
function formatToolName(name) {
  const overrides = {
    webfetch: "Web Fetch",
    todowrite: "Todo",
    read_skill: "Read Skill",
    web_search: "Web Search",
    spacebot_docs: "Docs"
  };
  if (overrides[name]) return overrides[name];
  const stripped = name.replace(/^browser_/, "").replace(/^file_/, "").replace(/^tab_/, "Tab ");
  return stripped.split("_").map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(" ");
}
function toolCategory(name) {
  if (name.startsWith("browser_")) return "Browser";
  if (name.startsWith("file_")) return "File";
  return null;
}
function ToolCall({ pair }) {
  const [expanded, setExpanded] = useState(false);
  const renderer = getRenderer(pair.name);
  const summary = renderer.summary(pair);
  const category = toolCategory(pair.name);
  const displayName = formatToolName(pair.name);
  return /* @__PURE__ */ jsxs(
    "div",
    {
      className: clsx(
        "rounded-md border bg-app-dark-box/30",
        pair.status === "error" ? "border-status-error/30" : "border-app-line/50"
      ),
      children: [
        /* @__PURE__ */ jsxs(
          "button",
          {
            onClick: () => setExpanded(!expanded),
            className: "flex w-full items-center gap-2 px-3 py-2 text-left text-xs",
            children: [
              /* @__PURE__ */ jsx(
                "span",
                {
                  className: clsx(
                    STATUS_COLORS[pair.status],
                    pair.status === "running" ? "animate-pulse" : ""
                  ),
                  children: STATUS_ICONS[pair.status]
                }
              ),
              category && /* @__PURE__ */ jsx("span", { className: "text-[11px] text-ink-faint", children: category }),
              /* @__PURE__ */ jsx("span", { className: "font-medium text-ink-dull", children: displayName }),
              summary && !expanded && /* @__PURE__ */ jsx("span", { className: "flex-1 truncate text-ink-faint", children: summary }),
              pair.status === "running" && /* @__PURE__ */ jsx("span", { className: "h-1.5 w-1.5 animate-pulse rounded-full bg-accent" })
            ]
          }
        ),
        expanded && /* @__PURE__ */ jsxs("div", { className: "border-t border-app-line/30", children: [
          renderArgs(pair, renderer),
          renderResult(pair, renderer)
        ] })
      ]
    }
  );
}
function renderArgs(pair, renderer) {
  if (renderer.argsView) {
    const custom = renderer.argsView(pair);
    if (custom) return custom;
  }
  if (pair.args && Object.keys(pair.args).length > 0) {
    return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsx("div", { className: "flex flex-col gap-0.5", children: Object.entries(pair.args).map(([key, value]) => /* @__PURE__ */ jsxs("p", { className: "text-[11px]", children: [
      /* @__PURE__ */ jsxs("span", { className: "text-ink-faint", children: [
        key,
        ": "
      ] }),
      /* @__PURE__ */ jsx("span", { className: "text-ink-dull", children: formatArgValue(key, value) })
    ] }, key)) }) });
  }
  if (pair.argsRaw && pair.argsRaw !== "{}" && pair.argsRaw.trim().length > 0) {
    return /* @__PURE__ */ jsx("div", { className: "border-b border-app-line/20 px-3 py-2", children: /* @__PURE__ */ jsx("pre", { className: "max-h-40 overflow-auto font-mono text-[11px] text-ink-dull", children: pair.argsRaw }) });
  }
  return null;
}
function renderResult(pair, renderer) {
  if (pair.status === "running") {
    return /* @__PURE__ */ jsxs("div", { className: "flex items-center gap-2 px-3 py-2 text-[11px] text-ink-faint", children: [
      /* @__PURE__ */ jsx("span", { className: "h-1.5 w-1.5 animate-pulse rounded-full bg-accent" }),
      "Running..."
    ] });
  }
  if (renderer.resultView) {
    const custom = renderer.resultView(pair);
    if (custom !== void 0) return custom;
  }
  if (!pair.resultRaw) return null;
  if (pair.result && Object.keys(pair.result).length > 0) {
    return /* @__PURE__ */ jsx("div", { className: "px-3 py-2", children: /* @__PURE__ */ jsx("div", { className: "flex flex-col gap-0.5", children: Object.entries(pair.result).map(([key, value]) => /* @__PURE__ */ jsxs("p", { className: "text-[11px]", children: [
      /* @__PURE__ */ jsxs("span", { className: "text-ink-faint", children: [
        key,
        ": "
      ] }),
      /* @__PURE__ */ jsx("span", { className: "text-ink-dull", children: typeof value === "string" ? truncate(value, 200) : JSON.stringify(value) })
    ] }, key)) }) });
  }
  return /* @__PURE__ */ jsx(CollapsiblePre, { text: pair.resultRaw, maxLines: 20 });
}
function formatArgValue(key, value) {
  if (key === "secret" && typeof value === "string") {
    return "***";
  }
  if (typeof value === "string") {
    return truncate(value, 100);
  }
  if (typeof value === "boolean" || typeof value === "number") {
    return String(value);
  }
  return JSON.stringify(value);
}
function truncate(text, maxLen) {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen) + "...";
}

// src/Markdown.tsx
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { clsx as clsx2 } from "clsx";
import { forwardRef } from "react";
import { jsx as jsx2 } from "react/jsx-runtime";
var Markdown = forwardRef(
  ({ content, className }, ref) => {
    return /* @__PURE__ */ jsx2(
      "div",
      {
        ref,
        className: clsx2("markdown text-sm text-ink", className),
        children: /* @__PURE__ */ jsx2(
          ReactMarkdown,
          {
            remarkPlugins: [remarkGfm],
            children: content
          }
        )
      }
    );
  }
);
Markdown.displayName = "Markdown";

// src/MessageBubble.tsx
import { Copy } from "@phosphor-icons/react";
import { CircleButton } from "@wingdrive/primitives";
import clsx3 from "clsx";
import { jsx as jsx3, jsxs as jsxs2 } from "react/jsx-runtime";
function MessageBubble({
  content,
  isUser,
  isStreaming = false,
  onCopy
}) {
  return /* @__PURE__ */ jsxs2(
    "div",
    {
      className: clsx3(
        "group flex flex-col py-2",
        isUser ? "items-end" : "items-start"
      ),
      children: [
        /* @__PURE__ */ jsx3(
          "div",
          {
            className: clsx3(
              "max-w-[80%] rounded-2xl text-sm leading-6",
              isUser ? "bg-accent px-4 py-1 text-white" : "text-ink border border-none bg-transparent"
            ),
            children: isUser ? /* @__PURE__ */ jsx3("div", { className: "whitespace-pre-wrap break-words", children: content }) : /* @__PURE__ */ jsx3(Markdown, { content, className: "break-words" })
          }
        ),
        !isUser && onCopy ? /* @__PURE__ */ jsx3("div", { className: "mt-2 flex opacity-0 transition-opacity duration-150 group-hover:opacity-100", children: /* @__PURE__ */ jsx3(
          CircleButton,
          {
            icon: Copy,
            onClick: () => onCopy(content),
            "aria-label": isStreaming ? "Copy Streaming Message" : "Copy Message",
            title: isStreaming ? "Copy streaming message" : "Copy message",
            className: "h-7 w-7"
          }
        ) }) : null
      ]
    }
  );
}

// src/InlineWorkerCard.tsx
import { useMemo, useState as useState2 } from "react";
import { clsx as clsx4 } from "clsx";
import {
  CaretDown,
  CheckCircle,
  Copy as Copy2,
  Stop,
  Wrench
} from "@phosphor-icons/react";
import { Grid } from "react-loader-spinner";
import { AnimatePresence, motion } from "framer-motion";
import { jsx as jsx4, jsxs as jsxs3 } from "react/jsx-runtime";
function stripExcessWhitespace(text) {
  return text.replace(/ {3,}/g, "  ");
}
function InlineWorkerCard({
  title,
  status,
  toolCallCount,
  liveStatus,
  transcript,
  isTranscriptLoading,
  onCopyLogs,
  onCancel,
  className
}) {
  const [expanded, setExpanded] = useState2(false);
  const items = useMemo(() => pairTranscriptSteps(transcript), [transcript]);
  const isRunning = status === "running";
  const isDone = status === "completed";
  return /* @__PURE__ */ jsxs3("div", { className: clsx4("group flex min-w-0 flex-col items-start", className), children: [
    /* @__PURE__ */ jsxs3("div", { className: "min-w-0 max-w-full overflow-hidden rounded-2xl border border-app-line/50 bg-app-box/30 backdrop-blur-sm", children: [
      /* @__PURE__ */ jsxs3(
        "button",
        {
          onClick: () => setExpanded((v) => !v),
          className: "flex w-full items-start gap-3 px-4 py-3 text-left transition-colors hover:bg-app-box/30",
          children: [
            /* @__PURE__ */ jsx4("div", { className: "mt-0.5 shrink-0", children: isRunning ? /* @__PURE__ */ jsx4(Grid, { height: 16, width: 16, color: "currentColor", ariaLabel: "loading", wrapperClass: "text-accent" }) : isDone ? /* @__PURE__ */ jsx4("div", { className: "flex size-7 items-center justify-center rounded-full bg-emerald-500/15 text-emerald-400", children: /* @__PURE__ */ jsx4(CheckCircle, { className: "size-4", weight: "fill" }) }) : /* @__PURE__ */ jsx4("div", { className: "flex size-7 items-center justify-center rounded-full bg-app-hover text-ink-dull", children: /* @__PURE__ */ jsx4(Wrench, { className: "size-4", weight: "bold" }) }) }),
            /* @__PURE__ */ jsxs3("div", { className: "min-w-0 flex-1", children: [
              /* @__PURE__ */ jsxs3("div", { className: "flex items-center gap-2", children: [
                /* @__PURE__ */ jsx4("div", { className: "line-clamp-2 min-w-0 flex-1 text-sm font-medium leading-5 text-ink", children: title }),
                /* @__PURE__ */ jsx4(
                  "span",
                  {
                    className: clsx4(
                      "rounded-full px-2 py-0.5 text-[10px] font-medium uppercase tracking-[0.12em]",
                      isRunning ? "bg-accent/12 text-accent" : isDone ? "bg-emerald-500/12 text-emerald-400" : "bg-app-hover text-ink-dull"
                    ),
                    children: status
                  }
                )
              ] }),
              /* @__PURE__ */ jsxs3("div", { className: "mt-1 flex items-center gap-2 text-xs text-ink-dull", children: [
                /* @__PURE__ */ jsxs3("span", { children: [
                  toolCallCount,
                  " tool calls"
                ] }),
                liveStatus ? /* @__PURE__ */ jsx4("span", { className: "truncate text-ink-faint", children: liveStatus }) : null
              ] })
            ] }),
            /* @__PURE__ */ jsx4(
              CaretDown,
              {
                className: clsx4(
                  "mt-1 size-4 shrink-0 text-ink-faint transition-transform",
                  expanded ? "rotate-180" : ""
                ),
                weight: "bold"
              }
            )
          ]
        }
      ),
      /* @__PURE__ */ jsx4(AnimatePresence, { initial: false, children: expanded ? /* @__PURE__ */ jsx4(
        motion.div,
        {
          initial: { height: 0, opacity: 0 },
          animate: { height: "auto", opacity: 1 },
          exit: { height: 0, opacity: 0 },
          transition: { duration: 0.18, ease: "easeOut" },
          className: "overflow-hidden",
          children: /* @__PURE__ */ jsx4("div", { className: "flex flex-col gap-2 border-t border-app-line/30 px-4 py-3", children: isTranscriptLoading ? /* @__PURE__ */ jsx4("div", { className: "text-xs text-ink-faint", children: "Loading worker transcript..." }) : items.length > 0 ? items.map(
            (item, index) => item.kind === "tool" ? /* @__PURE__ */ jsx4(ToolCall, { pair: item.pair }, item.pair.id) : /* @__PURE__ */ jsx4(
              Markdown,
              {
                content: stripExcessWhitespace(item.text),
                className: "text-xs text-ink-dull"
              },
              `text-${index}`
            )
          ) : /* @__PURE__ */ jsx4("div", { className: "text-xs text-ink-faint", children: "No tool calls yet." }) })
        }
      ) : null })
    ] }),
    /* @__PURE__ */ jsxs3("div", { className: "mt-2 flex opacity-0 transition-opacity duration-150 group-focus-within:opacity-100 group-hover:opacity-100", children: [
      onCopyLogs && /* @__PURE__ */ jsxs3(
        "button",
        {
          onClick: onCopyLogs,
          className: "flex items-center gap-1 rounded-md px-2 py-1 text-xs text-ink-dull hover:bg-app-hover hover:text-ink",
          title: "Copy logs",
          children: [
            /* @__PURE__ */ jsx4(Copy2, { className: "size-3.5" }),
            "Copy"
          ]
        }
      ),
      onCancel && isRunning && /* @__PURE__ */ jsxs3(
        "button",
        {
          onClick: onCancel,
          className: "flex items-center gap-1 rounded-md px-2 py-1 text-xs text-ink-dull hover:bg-app-hover hover:text-ink",
          title: "Cancel",
          children: [
            /* @__PURE__ */ jsx4(Stop, { className: "size-3.5" }),
            "Cancel"
          ]
        }
      )
    ] })
  ] });
}

// src/InlineBranchCard.tsx
import { useState as useState3 } from "react";
import { clsx as clsx5 } from "clsx";
import { CaretDown as CaretDown2, GitBranch } from "@phosphor-icons/react";
import { Grid as Grid2 } from "react-loader-spinner";
import { jsx as jsx5, jsxs as jsxs4 } from "react/jsx-runtime";
function InlineBranchCard({
  description,
  completedAt,
  conclusion,
  className
}) {
  const [expanded, setExpanded] = useState3(false);
  const isRunning = !completedAt;
  return /* @__PURE__ */ jsx5("div", { className: clsx5("group flex min-w-0 flex-col items-start", className), children: /* @__PURE__ */ jsxs4("div", { className: "min-w-0 max-w-full overflow-hidden rounded-2xl border border-app-line/50 bg-app-box/30 backdrop-blur-sm", children: [
    /* @__PURE__ */ jsxs4(
      "button",
      {
        type: "button",
        onClick: () => setExpanded((v) => !v),
        className: "flex w-full items-start gap-3 px-4 py-3 text-left transition-colors hover:bg-app-box/30",
        children: [
          /* @__PURE__ */ jsx5("div", { className: "mt-0.5 shrink-0", children: isRunning ? /* @__PURE__ */ jsx5(Grid2, { height: 16, width: 16, color: "currentColor", ariaLabel: "loading", wrapperClass: "text-accent" }) : /* @__PURE__ */ jsx5("div", { className: "flex size-7 items-center justify-center rounded-full bg-accent/15 text-accent", children: /* @__PURE__ */ jsx5(GitBranch, { className: "size-4", weight: "bold" }) }) }),
          /* @__PURE__ */ jsx5("div", { className: "min-w-0 flex-1", children: /* @__PURE__ */ jsxs4("div", { className: "flex items-center gap-2", children: [
            /* @__PURE__ */ jsx5("div", { className: "line-clamp-2 min-w-0 flex-1 text-sm font-medium leading-5 text-ink", children: description }),
            /* @__PURE__ */ jsx5(
              "span",
              {
                className: clsx5(
                  "rounded-full px-2 py-0.5 text-[10px] font-medium uppercase tracking-[0.12em]",
                  isRunning ? "bg-accent/12 text-accent" : "bg-emerald-500/12 text-emerald-400"
                ),
                children: isRunning ? "thinking" : "done"
              }
            )
          ] }) }),
          conclusion && /* @__PURE__ */ jsx5(
            CaretDown2,
            {
              className: clsx5(
                "mt-1 size-4 shrink-0 text-ink-faint transition-transform",
                expanded ? "rotate-180" : ""
              ),
              weight: "bold"
            }
          )
        ]
      }
    ),
    expanded && conclusion && /* @__PURE__ */ jsx5("div", { className: "border-t border-app-line/30 px-4 py-3", children: /* @__PURE__ */ jsx5(Markdown, { content: conclusion, className: "text-xs text-ink-dull" }) })
  ] }) });
}

// src/ModelSelector.tsx
import { Check } from "@phosphor-icons/react";
import {
  Popover,
  usePopover,
  SelectPill,
  SearchBar,
  OptionList,
  OptionListItem
} from "@wingdrive/primitives";
import clsx6 from "clsx";
import { forwardRef as forwardRef2, useMemo as useMemo2, useState as useState4 } from "react";
import { jsx as jsx6, jsxs as jsxs5 } from "react/jsx-runtime";
var ModelSelector = forwardRef2(
  ({
    models,
    value,
    onChange,
    placeholder = "Select model...",
    searchPlaceholder = "Search models...",
    variant,
    size,
    popover: externalPopover,
    className,
    disabled
  }, ref) => {
    const internalPopover = usePopover();
    const popover = externalPopover ?? internalPopover;
    const [search, setSearch] = useState4("");
    const selectedModel = models.find((m) => m.id === value);
    const grouped = useMemo2(() => {
      const q = search.toLowerCase();
      const filtered = search.trim() ? models.filter(
        (m) => m.name.toLowerCase().includes(q) || m.provider.toLowerCase().includes(q)
      ) : models;
      return filtered.reduce(
        (acc, model) => {
          if (!acc[model.provider]) acc[model.provider] = [];
          acc[model.provider].push(model);
          return acc;
        },
        {}
      );
    }, [models, search]);
    return /* @__PURE__ */ jsxs5(
      Popover.Root,
      {
        open: popover.open,
        onOpenChange: (open) => {
          popover.setOpen(open);
          if (!open) setSearch("");
        },
        children: [
          /* @__PURE__ */ jsx6(Popover.Trigger, { asChild: true, children: /* @__PURE__ */ jsx6(
            SelectPill,
            {
              ref,
              variant,
              size,
              disabled,
              className: clsx6("w-full", className),
              children: selectedModel?.name ?? placeholder
            }
          ) }),
          /* @__PURE__ */ jsxs5(Popover.Content, { align: "end", sideOffset: 8, style: { width: 280 }, children: [
            /* @__PURE__ */ jsx6(
              SearchBar,
              {
                value: search,
                onChange: setSearch,
                placeholder: searchPlaceholder,
                autoFocus: true,
                className: "mb-1.5"
              }
            ),
            /* @__PURE__ */ jsxs5("div", { className: "max-h-[280px] overflow-y-auto", children: [
              Object.entries(grouped).map(([provider, providerModels]) => /* @__PURE__ */ jsxs5("div", { children: [
                /* @__PURE__ */ jsx6("div", { className: "px-3 py-1 text-[10px] font-semibold uppercase tracking-wider text-ink-faint", children: provider }),
                /* @__PURE__ */ jsx6(OptionList, { children: providerModels.map((model) => /* @__PURE__ */ jsx6(
                  OptionListItem,
                  {
                    selected: model.id === value,
                    onClick: () => {
                      onChange(model.id);
                      popover.setOpen(false);
                      setSearch("");
                    },
                    children: /* @__PURE__ */ jsxs5("div", { className: "flex w-full items-center justify-between", children: [
                      /* @__PURE__ */ jsxs5("div", { children: [
                        /* @__PURE__ */ jsx6("div", { children: model.name }),
                        model.context_window && /* @__PURE__ */ jsxs5("div", { className: "text-[10px] text-ink-faint", children: [
                          model.context_window.toLocaleString(),
                          " tokens"
                        ] })
                      ] }),
                      model.id === value && /* @__PURE__ */ jsx6(
                        Check,
                        {
                          className: "size-3.5 text-accent",
                          weight: "bold"
                        }
                      )
                    ] })
                  },
                  model.id
                )) })
              ] }, provider)),
              Object.keys(grouped).length === 0 && /* @__PURE__ */ jsx6("div", { className: "px-3 py-3 text-center text-xs text-ink-faint", children: "No models found" })
            ] })
          ] })
        ]
      }
    );
  }
);
ModelSelector.displayName = "ModelSelector";

// src/ChatComposer.tsx
import { Microphone, Sparkle } from "@phosphor-icons/react";
import {
  CircleButton as CircleButton2,
  OptionList as OptionList2,
  OptionListItem as OptionListItem2,
  Popover as Popover2,
  SelectPill as SelectPill2
} from "@wingdrive/primitives";
import { AnimatePresence as AnimatePresence2, motion as motion2 } from "framer-motion";
import { Fragment, jsx as jsx7, jsxs as jsxs6 } from "react/jsx-runtime";
function ChatComposer({
  draft,
  onDraftChange,
  onSend,
  placeholder = "Ask something...",
  heading,
  isSending = false,
  projectSelector,
  modelSelector,
  onOpenVoice,
  toolbarExtra
}) {
  const canSend = !isSending && draft.trim().length > 0;
  return /* @__PURE__ */ jsxs6(Fragment, { children: [
    heading && /* @__PURE__ */ jsxs6("div", { className: "text-ink-dull mb-3 flex items-center gap-2 px-1 text-xs font-medium", children: [
      /* @__PURE__ */ jsx7("span", { className: "text-accent inline-flex size-3.5 shrink-0", children: /* @__PURE__ */ jsx7(Sparkle, { size: "100%", weight: "fill" }) }),
      heading
    ] }),
    /* @__PURE__ */ jsxs6("div", { className: "border-app-line bg-app-box/70 rounded-[24px] border p-4 shadow-[0_20px_60px_rgba(0,0,0,0.18)] backdrop-blur-2xl", children: [
      /* @__PURE__ */ jsx7(
        "textarea",
        {
          value: draft,
          onChange: (event) => onDraftChange(event.target.value),
          onKeyDown: (event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              onSend();
            }
          },
          placeholder,
          rows: 2,
          className: "text-ink placeholder:text-ink-faint block w-full resize-none rounded-md border-0 bg-transparent text-sm leading-6 outline-none focus:border-0 focus:outline-none focus:ring-0"
        }
      ),
      /* @__PURE__ */ jsxs6("div", { className: "mt-4 flex items-center justify-between gap-3", children: [
        projectSelector ? /* @__PURE__ */ jsx7("div", { className: "w-[210px]", children: /* @__PURE__ */ jsxs6(
          Popover2.Root,
          {
            open: projectSelector.popover.open,
            onOpenChange: projectSelector.popover.setOpen,
            children: [
              /* @__PURE__ */ jsx7(Popover2.Trigger, { asChild: true, children: /* @__PURE__ */ jsx7(SelectPill2, { className: "w-full", children: projectSelector.value }) }),
              /* @__PURE__ */ jsx7(Popover2.Content, { align: "start", sideOffset: 8, children: /* @__PURE__ */ jsx7(OptionList2, { children: projectSelector.options.map((option) => /* @__PURE__ */ jsx7(
                OptionListItem2,
                {
                  selected: option === projectSelector.value,
                  onClick: () => {
                    projectSelector.onChange(option);
                    projectSelector.popover.setOpen(false);
                  },
                  children: option
                },
                option
              )) }) })
            ]
          }
        ) }) : /* @__PURE__ */ jsx7("div", {}),
        /* @__PURE__ */ jsxs6(motion2.div, { layout: true, className: "flex items-center gap-2", children: [
          modelSelector && /* @__PURE__ */ jsx7("div", { className: "w-[180px]", children: /* @__PURE__ */ jsx7(
            ModelSelector,
            {
              models: modelSelector.options,
              value: modelSelector.value,
              onChange: modelSelector.onChange
            }
          ) }),
          toolbarExtra,
          onOpenVoice && /* @__PURE__ */ jsx7(
            CircleButton2,
            {
              icon: Microphone,
              onClick: onOpenVoice,
              "aria-label": "Open Voice Input"
            }
          ),
          /* @__PURE__ */ jsx7(AnimatePresence2, { initial: false, children: canSend ? /* @__PURE__ */ jsx7(
            motion2.div,
            {
              layout: true,
              initial: { width: 0, opacity: 0, x: 12 },
              animate: { width: 76, opacity: 1, x: 0 },
              exit: { width: 0, opacity: 0, x: 12 },
              transition: { duration: 0.18, ease: "easeOut" },
              className: "overflow-hidden",
              children: /* @__PURE__ */ jsx7(
                "button",
                {
                  onClick: onSend,
                  className: "border-app-line bg-accent hover:bg-accent-faint flex h-9 w-[76px] items-center justify-center rounded-full border px-4 text-xs font-medium text-white",
                  children: /* @__PURE__ */ jsx7("span", { className: "whitespace-nowrap", children: "Send" })
                }
              )
            },
            "send-wrap"
          ) : null })
        ] })
      ] })
    ] })
  ] });
}

// src/TaskStatusIcon.tsx
import {
  CheckCircle as CheckCircle2,
  Circle,
  CircleDashed,
  CircleHalf,
  Clock
} from "@phosphor-icons/react";
import clsx7 from "clsx";
import { jsx as jsx8 } from "react/jsx-runtime";
var config = {
  done: { icon: CheckCircle2, weight: "fill", color: "text-emerald-400" },
  in_progress: { icon: CircleHalf, weight: "fill", color: "text-violet-400" },
  ready: { icon: Circle, weight: "fill", color: "text-accent" },
  pending_approval: { icon: Clock, weight: "fill", color: "text-amber-400" },
  backlog: { icon: CircleDashed, weight: "bold", color: "text-ink-faint" }
};
function TaskStatusIcon({ status, size = 16, className }) {
  const { icon: Icon, weight, color } = config[status];
  return /* @__PURE__ */ jsx8(
    Icon,
    {
      size,
      weight,
      className: clsx7(color, className)
    }
  );
}

// src/TaskPriorityIcon.tsx
import { ArrowDown, ArrowUp, Equals, Warning } from "@phosphor-icons/react";
import clsx8 from "clsx";
import { jsx as jsx9 } from "react/jsx-runtime";
var config2 = {
  critical: { icon: Warning, weight: "fill", color: "text-red-400" },
  high: { icon: ArrowUp, weight: "bold", color: "text-amber-400" },
  medium: { icon: Equals, weight: "bold", color: "text-ink-dull" },
  low: { icon: ArrowDown, weight: "bold", color: "text-ink-faint" }
};
function TaskPriorityIcon({ priority, size = 14, className }) {
  const { icon: Icon, weight, color } = config2[priority];
  return /* @__PURE__ */ jsx9(Icon, { size, weight, className: clsx8(color, className) });
}

// src/TaskRow.tsx
import { DotsThree } from "@phosphor-icons/react";
import { DropdownMenu } from "@wingdrive/primitives";
import clsx9 from "clsx";
import { jsx as jsx10, jsxs as jsxs7 } from "react/jsx-runtime";
function TaskRow({
  task,
  onClick,
  isActive,
  resolveAgentName,
  onStatusChange,
  onDelete,
  className
}) {
  const completedSubtasks = task.subtasks.filter((s) => s.completed).length;
  const assigneeName = resolveAgentName?.(task.assigned_agent_id) ?? task.assigned_agent_id;
  const hasActions = onStatusChange || onDelete;
  const isRowInteractive = typeof onClick === "function";
  const handleRowKeyDown = (e) => {
    if (e.repeat) return;
    if (e.key === "Enter") {
      e.preventDefault();
      onClick?.(task);
    }
  };
  const handleRowKeyUp = (e) => {
    if (e.repeat) return;
    if (e.key === " ") {
      e.preventDefault();
      onClick?.(task);
    }
  };
  return /* @__PURE__ */ jsxs7(
    "div",
    {
      role: isRowInteractive ? "button" : void 0,
      tabIndex: isRowInteractive ? 0 : void 0,
      onClick: isRowInteractive ? () => onClick(task) : void 0,
      onKeyDown: isRowInteractive ? handleRowKeyDown : void 0,
      onKeyUp: isRowInteractive ? handleRowKeyUp : void 0,
      className: clsx9(
        "task-row group grid w-full items-center border-b border-app-line/40 text-left transition-colors",
        "hover:bg-app-hover/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
        isActive ? "bg-app-selected/50" : "bg-transparent",
        className
      ),
      style: {
        gridTemplateColumns: "20px 18px 72px 1fr 56px 80px 28px",
        height: 36,
        paddingLeft: 12,
        paddingRight: 8,
        gap: 8
      },
      children: [
        /* @__PURE__ */ jsx10(TaskStatusIcon, { status: task.status }),
        /* @__PURE__ */ jsx10(TaskPriorityIcon, { priority: task.priority }),
        /* @__PURE__ */ jsxs7("span", { className: "truncate font-mono text-xs text-ink-faint", children: [
          "SPC-",
          task.task_number
        ] }),
        /* @__PURE__ */ jsx10("span", { className: "min-w-0 truncate text-sm text-ink", children: task.title }),
        /* @__PURE__ */ jsx10("span", { className: "text-right text-xs text-ink-faint", children: task.subtasks.length > 0 ? `${completedSubtasks}/${task.subtasks.length}` : "" }),
        /* @__PURE__ */ jsx10("span", { className: "truncate text-right text-xs text-ink-dull", children: assigneeName }),
        /* @__PURE__ */ jsx10("span", { className: "flex items-center justify-center", children: hasActions ? /* @__PURE__ */ jsxs7(DropdownMenu.Root, { children: [
          /* @__PURE__ */ jsx10(DropdownMenu.Trigger, { asChild: true, children: /* @__PURE__ */ jsx10(
            "button",
            {
              type: "button",
              "aria-label": "Task Actions",
              onClick: (e) => e.stopPropagation(),
              onKeyDown: (e) => {
                if (e.key === "Enter" || e.key === " ") e.stopPropagation();
              },
              className: "flex size-5 items-center justify-center rounded opacity-0 transition-opacity hover:bg-app-hover group-hover:opacity-100",
              children: /* @__PURE__ */ jsx10(DotsThree, { size: 16, weight: "bold", className: "text-ink-dull" })
            }
          ) }),
          /* @__PURE__ */ jsxs7(DropdownMenu.Content, { align: "end", sideOffset: 4, children: [
            onStatusChange && TASK_STATUS_ORDER.filter((s) => s !== task.status).map((s) => /* @__PURE__ */ jsxs7(
              DropdownMenu.Item,
              {
                onClick: (e) => {
                  e.stopPropagation();
                  onStatusChange(task, s);
                },
                children: [
                  /* @__PURE__ */ jsx10(TaskStatusIcon, { status: s, size: 14 }),
                  /* @__PURE__ */ jsx10("span", { className: "ml-2", children: TASK_STATUS_LABEL[s] })
                ]
              },
              s
            )),
            onStatusChange && onDelete && /* @__PURE__ */ jsx10(DropdownMenu.Separator, {}),
            onDelete && /* @__PURE__ */ jsx10(
              DropdownMenu.Item,
              {
                onClick: (e) => {
                  e.stopPropagation();
                  onDelete(task);
                },
                className: "text-red-400",
                children: "Delete"
              }
            )
          ] })
        ] }) : null })
      ]
    }
  );
}

// src/TaskList.tsx
import { CaretRight } from "@phosphor-icons/react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@wingdrive/primitives";
import clsx10 from "clsx";
import { useMemo as useMemo3 } from "react";
import { jsx as jsx11, jsxs as jsxs8 } from "react/jsx-runtime";
var COL_GRID = "20px 18px 72px 1fr 56px 80px 28px";
var GRID_STYLE = {
  gridTemplateColumns: COL_GRID,
  paddingLeft: 12,
  paddingRight: 8,
  gap: 8
};
function TaskList({
  tasks,
  groups = TASK_STATUS_ORDER,
  collapsedGroups,
  onToggleGroup,
  activeTaskId,
  onTaskClick,
  onStatusChange,
  onDelete,
  resolveAgentName,
  className
}) {
  const grouped = useMemo3(() => {
    const map = /* @__PURE__ */ new Map();
    for (const status of groups) map.set(status, []);
    for (const task of tasks) {
      const bucket = map.get(task.status);
      if (bucket) bucket.push(task);
    }
    return map;
  }, [tasks, groups]);
  return /* @__PURE__ */ jsxs8("div", { className: clsx10("flex flex-col", className), children: [
    /* @__PURE__ */ jsxs8(
      "div",
      {
        className: "grid items-center border-b border-app-line bg-app-box/40 text-[11px] font-medium uppercase tracking-wider text-ink-faint",
        style: { ...GRID_STYLE, height: 28 },
        children: [
          /* @__PURE__ */ jsx11("span", {}),
          /* @__PURE__ */ jsx11("span", {}),
          /* @__PURE__ */ jsx11("span", { children: "ID" }),
          /* @__PURE__ */ jsx11("span", { children: "Title" }),
          /* @__PURE__ */ jsx11("span", { className: "text-right", children: "Subs" }),
          /* @__PURE__ */ jsx11("span", { className: "text-right", children: "Assignee" }),
          /* @__PURE__ */ jsx11("span", {})
        ]
      }
    ),
    groups.map((status) => {
      const items = grouped.get(status) ?? [];
      const isOpen = !collapsedGroups?.has(status);
      return /* @__PURE__ */ jsxs8(
        Collapsible,
        {
          open: isOpen,
          onOpenChange: () => onToggleGroup?.(status),
          children: [
            /* @__PURE__ */ jsxs8(
              CollapsibleTrigger,
              {
                className: "grid w-full items-center border-b border-app-line/40 bg-app-box/20 text-xs font-medium text-ink-dull hover:bg-app-hover/40",
                style: { ...GRID_STYLE, height: 30 },
                children: [
                  /* @__PURE__ */ jsx11("span", { className: "flex items-center justify-center", children: /* @__PURE__ */ jsx11(
                    CaretRight,
                    {
                      size: 10,
                      weight: "bold",
                      className: clsx10(
                        "shrink-0 transition-transform",
                        isOpen && "rotate-90"
                      )
                    }
                  ) }),
                  /* @__PURE__ */ jsx11("span", { className: "flex items-center justify-center", children: /* @__PURE__ */ jsx11(TaskStatusIcon, { status, size: 12 }) }),
                  /* @__PURE__ */ jsxs8("span", { className: "col-span-5 flex items-center gap-1.5 text-left", children: [
                    TASK_STATUS_LABEL[status],
                    /* @__PURE__ */ jsxs8("span", { className: "text-ink-faint", children: [
                      "(",
                      items.length,
                      ")"
                    ] })
                  ] })
                ]
              }
            ),
            /* @__PURE__ */ jsx11(CollapsibleContent, { children: items.length > 0 ? items.map((task) => /* @__PURE__ */ jsx11(
              TaskRow,
              {
                task,
                isActive: task.id === activeTaskId,
                onClick: onTaskClick,
                onStatusChange,
                onDelete,
                resolveAgentName
              },
              task.id
            )) : /* @__PURE__ */ jsx11("div", { className: "border-b border-app-line/40 px-3 py-2 text-xs italic text-ink-faint", children: "No tasks" }) })
          ]
        },
        status
      );
    })
  ] });
}

// src/TaskDetail.tsx
import { Check as Check2, X } from "@phosphor-icons/react";
import { Button, Select, SelectOption } from "@wingdrive/primitives";
import clsx11 from "clsx";
import { jsx as jsx12, jsxs as jsxs9 } from "react/jsx-runtime";
function formatDate(iso) {
  return new Date(iso).toLocaleDateString(void 0, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
}
function TaskDetail({
  task,
  resolveAgentName,
  onStatusChange,
  onSubtaskToggle,
  onDelete,
  onClose,
  className,
  beforeSubtasks
}) {
  const ownerName = resolveAgentName?.(task.owner_agent_id) ?? task.owner_agent_id;
  const assigneeName = resolveAgentName?.(task.assigned_agent_id) ?? task.assigned_agent_id;
  return /* @__PURE__ */ jsxs9("div", { className: clsx11("flex flex-col gap-5 p-4", className), children: [
    /* @__PURE__ */ jsxs9("div", { className: "flex items-start gap-3", children: [
      /* @__PURE__ */ jsxs9("div", { className: "flex flex-1 items-center gap-2", children: [
        /* @__PURE__ */ jsx12(TaskStatusIcon, { status: task.status }),
        /* @__PURE__ */ jsx12(TaskPriorityIcon, { priority: task.priority }),
        /* @__PURE__ */ jsxs9("span", { className: "font-mono text-xs text-ink-faint", children: [
          "SPC-",
          task.task_number
        ] })
      ] }),
      onClose && /* @__PURE__ */ jsx12(
        "button",
        {
          type: "button",
          onClick: onClose,
          className: "shrink-0 rounded p-1 text-ink-dull hover:bg-app-hover hover:text-ink",
          children: /* @__PURE__ */ jsx12(X, { size: 16 })
        }
      )
    ] }),
    /* @__PURE__ */ jsx12("h2", { className: "text-lg font-medium text-ink", children: task.title }),
    /* @__PURE__ */ jsxs9("div", { className: "flex flex-wrap items-center gap-x-4 gap-y-2 text-sm", children: [
      onStatusChange ? /* @__PURE__ */ jsxs9("label", { className: "flex items-center gap-2 text-ink-dull", children: [
        "Status",
        /* @__PURE__ */ jsx12(
          Select,
          {
            value: task.status,
            size: "sm",
            onChange: (value) => onStatusChange(task, value),
            children: TASK_STATUS_ORDER.map((s) => /* @__PURE__ */ jsx12(SelectOption, { value: s, children: TASK_STATUS_LABEL[s] }, s))
          }
        )
      ] }) : /* @__PURE__ */ jsxs9("span", { className: "text-ink-dull", children: [
        "Status: ",
        /* @__PURE__ */ jsx12("span", { className: "text-ink", children: TASK_STATUS_LABEL[task.status] })
      ] }),
      /* @__PURE__ */ jsxs9("span", { className: "text-ink-dull", children: [
        "Priority:",
        " ",
        /* @__PURE__ */ jsx12("span", { className: "text-ink", children: TASK_PRIORITY_LABEL[task.priority] })
      ] }),
      /* @__PURE__ */ jsxs9("span", { className: "text-ink-dull", children: [
        "Owner: ",
        /* @__PURE__ */ jsx12("span", { className: "text-ink", children: ownerName })
      ] }),
      /* @__PURE__ */ jsxs9("span", { className: "text-ink-dull", children: [
        "Assigned: ",
        /* @__PURE__ */ jsx12("span", { className: "text-ink", children: assigneeName })
      ] })
    ] }),
    beforeSubtasks,
    task.subtasks.length > 0 && /* @__PURE__ */ jsxs9("div", { className: "flex flex-col gap-1", children: [
      /* @__PURE__ */ jsxs9("div", { className: "mb-1 flex items-center justify-between", children: [
        /* @__PURE__ */ jsx12("h3", { className: "text-xs font-medium uppercase tracking-wide text-ink-dull", children: "Subtasks" }),
        /* @__PURE__ */ jsxs9("span", { className: "text-xs text-ink-faint", children: [
          task.subtasks.filter((s) => s.completed).length,
          "/",
          task.subtasks.length
        ] })
      ] }),
      /* @__PURE__ */ jsx12("div", { className: "overflow-hidden rounded-md border border-app-line/60", children: task.subtasks.map((subtask, i) => /* @__PURE__ */ jsxs9(
        "button",
        {
          type: "button",
          onClick: () => onSubtaskToggle?.(task, i, !subtask.completed),
          className: clsx11(
            "flex w-full items-center gap-2.5 px-3 py-2 text-left text-sm transition-colors hover:bg-app-hover/50",
            i > 0 && "border-t border-app-line/40"
          ),
          children: [
            /* @__PURE__ */ jsx12(
              "span",
              {
                className: clsx11(
                  "flex size-4 shrink-0 items-center justify-center rounded border transition-colors",
                  subtask.completed ? "border-accent bg-accent" : "border-app-line bg-app-box"
                ),
                children: subtask.completed && /* @__PURE__ */ jsx12(Check2, { size: 10, weight: "bold", className: "text-white" })
              }
            ),
            /* @__PURE__ */ jsx12(
              "span",
              {
                className: clsx11(
                  "min-w-0 flex-1",
                  subtask.completed ? "text-ink-faint line-through" : "text-ink"
                ),
                children: subtask.title
              }
            )
          ]
        },
        i
      )) })
    ] }),
    /* @__PURE__ */ jsx12("div", { children: task.description ? /* @__PURE__ */ jsx12(Markdown, { content: task.description, className: "prose-sm prose-p:my-1 prose-headings:text-sm prose-headings:mt-3 prose-headings:mb-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0" }) : /* @__PURE__ */ jsx12("p", { className: "text-sm italic text-ink-faint", children: "No description" }) }),
    /* @__PURE__ */ jsxs9("div", { className: "flex flex-col gap-1 text-xs text-ink-faint", children: [
      /* @__PURE__ */ jsxs9("span", { children: [
        "Created ",
        formatDate(task.created_at)
      ] }),
      /* @__PURE__ */ jsxs9("span", { children: [
        "Updated ",
        formatDate(task.updated_at)
      ] }),
      task.completed_at && /* @__PURE__ */ jsxs9("span", { children: [
        "Completed ",
        formatDate(task.completed_at)
      ] })
    ] }),
    (onStatusChange || onDelete) && /* @__PURE__ */ jsxs9("div", { className: "flex flex-wrap gap-2", children: [
      onStatusChange && task.status === "pending_approval" && /* @__PURE__ */ jsx12(
        Button,
        {
          variant: "accent",
          size: "sm",
          onClick: () => onStatusChange(task, "ready"),
          children: "Approve"
        }
      ),
      onStatusChange && task.status === "backlog" && /* @__PURE__ */ jsx12(
        Button,
        {
          variant: "accent",
          size: "sm",
          onClick: () => onStatusChange(task, "in_progress"),
          children: "Execute"
        }
      ),
      onStatusChange && (task.status === "in_progress" || task.status === "ready") && /* @__PURE__ */ jsx12(
        Button,
        {
          variant: "accent",
          size: "sm",
          onClick: () => onStatusChange(task, "done"),
          children: "Mark Done"
        }
      ),
      onStatusChange && task.status === "done" && /* @__PURE__ */ jsx12(
        Button,
        {
          variant: "outline",
          size: "sm",
          onClick: () => onStatusChange(task, "ready"),
          children: "Reopen"
        }
      ),
      onDelete && /* @__PURE__ */ jsx12(
        Button,
        {
          variant: "outline",
          size: "sm",
          className: "text-red-400 hover:text-red-300",
          onClick: () => onDelete(task),
          children: "Delete"
        }
      )
    ] })
  ] });
}

// src/TaskCreateForm.tsx
import { Button as Button2, Select as Select2, SelectOption as SelectOption2 } from "@wingdrive/primitives";
import clsx12 from "clsx";
import { useCallback, useId, useState as useState5 } from "react";
import { jsx as jsx13, jsxs as jsxs10 } from "react/jsx-runtime";
var PRIORITIES = ["critical", "high", "medium", "low"];
function TaskCreateForm({
  onSubmit,
  onCancel,
  defaultPriority = "medium",
  isSubmitting,
  className
}) {
  const [title, setTitle] = useState5("");
  const [description, setDescription] = useState5("");
  const [priority, setPriority] = useState5(defaultPriority);
  const titleInputId = useId();
  const descriptionInputId = useId();
  const canSubmit = title.trim().length > 0 && !isSubmitting;
  const handleSubmit = useCallback(() => {
    if (!canSubmit) return;
    onSubmit({ title: title.trim(), description: description.trim(), priority });
    setTitle("");
    setDescription("");
    setPriority(defaultPriority);
  }, [canSubmit, onSubmit, title, description, priority, defaultPriority]);
  return /* @__PURE__ */ jsxs10("div", { className: clsx12("flex flex-col gap-2", className), children: [
    /* @__PURE__ */ jsxs10("div", { className: "flex items-center gap-2", children: [
      /* @__PURE__ */ jsx13("label", { htmlFor: titleInputId, className: "sr-only", children: "Task Title" }),
      /* @__PURE__ */ jsx13(
        "input",
        {
          id: titleInputId,
          type: "text",
          value: title,
          onChange: (e) => setTitle(e.target.value),
          onKeyDown: (e) => {
            if (e.key === "Enter") handleSubmit();
            if (e.key === "Escape") onCancel?.();
          },
          placeholder: "Task title...",
          className: "min-w-0 flex-1 rounded-md bg-transparent px-2 py-1.5 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus-visible:ring-2 focus-visible:ring-accent",
          disabled: isSubmitting,
          autoFocus: true
        }
      ),
      /* @__PURE__ */ jsx13(
        Select2,
        {
          value: priority,
          size: "sm",
          onChange: (value) => setPriority(value),
          disabled: isSubmitting,
          children: PRIORITIES.map((p) => /* @__PURE__ */ jsx13(SelectOption2, { value: p, children: TASK_PRIORITY_LABEL[p] }, p))
        }
      ),
      /* @__PURE__ */ jsx13(
        Button2,
        {
          variant: "accent",
          size: "sm",
          disabled: !canSubmit,
          onClick: handleSubmit,
          children: "Create"
        }
      )
    ] }),
    /* @__PURE__ */ jsx13("label", { htmlFor: descriptionInputId, className: "sr-only", children: "Description" }),
    /* @__PURE__ */ jsx13(
      "textarea",
      {
        id: descriptionInputId,
        value: description,
        onChange: (e) => setDescription(e.target.value),
        placeholder: "Description (optional)",
        rows: 2,
        className: "w-full resize-none rounded-md bg-transparent px-2 py-1.5 text-sm text-ink placeholder:text-ink-faint focus:outline-none focus-visible:ring-2 focus-visible:ring-accent",
        disabled: isSubmitting
      }
    )
  ] });
}
export {
  ChatComposer,
  InlineBranchCard,
  InlineWorkerCard,
  Markdown,
  MessageBubble,
  ModelSelector,
  TASK_PRIORITY_LABEL,
  TASK_STATUS_LABEL,
  TASK_STATUS_ORDER,
  TaskCreateForm,
  TaskDetail,
  TaskList,
  TaskPriorityIcon,
  TaskRow,
  TaskStatusIcon,
  ToolCall,
  isErrorResult,
  pairTranscriptSteps,
  tryParseJson
};
//# sourceMappingURL=index.js.map