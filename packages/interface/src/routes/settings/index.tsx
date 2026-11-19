import { useState, useEffect, useLayoutEffect, useRef } from "react";
import clsx from "clsx";
import {
  Books,
  Cpu,
  GearSix,
  Info,
  MagnifyingGlass,
  PaintBrush,
  PlugsConnected,
  ShieldCheck,
  Wrench,
  type Icon,
} from "@phosphor-icons/react";
import {
  GeneralSettings,
  AppearanceSettings,
  LibrarySettings,
  IndexerSettings,
  ServicesSettings,
  IntegrationsSettings,
  PrivacySettings,
  AdvancedSettings,
  AboutSettings,
} from "../../Settings/pages";
import { usePlatform } from "../../contexts/PlatformContext";
import { getCurrentPlatform } from "../../util/keybinds/platform";
import { useTranslation } from "../../i18n";

const sections = {
  general: { icon: GearSix, Page: GeneralSettings },
  appearance: { icon: PaintBrush, Page: AppearanceSettings },
  library: { icon: Books, Page: LibrarySettings },
  indexer: { icon: MagnifyingGlass, Page: IndexerSettings },
  services: { icon: Cpu, Page: ServicesSettings },
  integrations: { icon: PlugsConnected, Page: IntegrationsSettings },
  privacy: { icon: ShieldCheck, Page: PrivacySettings },
  advanced: { icon: Wrench, Page: AdvancedSettings },
  about: { icon: Info, Page: AboutSettings },
} satisfies Record<string, { icon: Icon; Page: React.ComponentType }>;

type SettingsPage = keyof typeof sections;
const pageIds = Object.keys(sections) as SettingsPage[];

function pageFromUrl(): SettingsPage {
  const requested = window.location.pathname.split("/").filter(Boolean)[1];
  return pageIds.find((id) => id === requested) ?? "general";
}

// The 52px top inset only makes sense on macOS, where the window has no
// native titlebar and the traffic lights overlay the content. On Linux and
// Windows the window keeps its native decorations, so that padding showed up
// as a dead strip and the drag region swallowed clicks on the nav header.
const isMacOS = getCurrentPlatform() === "macos";

function SettingsSidebar({
  currentPage,
  onPageChange,
}: {
  currentPage: SettingsPage;
  onPageChange: (page: SettingsPage) => void;
}) {
  const { t } = useTranslation();
  return (
    <>
      <select
        aria-label={t("title")}
        className="w-full rounded-md border border-sidebar-line bg-sidebar-box px-3 py-2 text-sm text-sidebar-ink focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent md:hidden"
        value={currentPage}
        onChange={(event) => {
          const selected = pageIds.find((id) => id === event.target.value);
          if (selected) onPageChange(selected);
        }}
      >
        {pageIds.map((id) => <option key={id} value={id}>{t(`sections.${id}`)}</option>)}
      </select>
      <div className="hidden flex-col gap-1 p-0.5 md:flex">
        {pageIds.map((id) => {
          const SectionIcon = sections[id].icon;
          const active = currentPage === id;
          return (
            <a
              key={id}
              href={`/settings/${id}`}
              aria-current={active ? "page" : undefined}
              onClick={(event) => {
                if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
                event.preventDefault();
                onPageChange(id);
              }}
              className={clsx(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
                active
                  ? "bg-sidebar-selected text-sidebar-ink"
                  : "text-sidebar-ink-dull hover:bg-sidebar-box hover:text-sidebar-ink"
              )}
            >
              <SectionIcon className="size-4 shrink-0" weight={active ? "fill" : "regular"} aria-hidden />
              {t(`sections.${id}`)}
            </a>
          );
        })}
      </div>
    </>
  );
}

function SettingsContentWrapper() {
  const { t } = useTranslation();
  const [currentPage, setCurrentPage] = useState(pageFromUrl);
  const mainRef = useRef<HTMLElement>(null);
  const Page = sections[currentPage].Page;

  useEffect(() => {
    const syncPage = () => setCurrentPage(pageFromUrl());
    window.addEventListener("popstate", syncPage);
    return () => window.removeEventListener("popstate", syncPage);
  }, []);

  const changePage = (page: SettingsPage) => {
    if (page === currentPage) return;
    window.history.pushState(window.history.state, "", `/settings/${page}`);
    setCurrentPage(page);
  };

  useLayoutEffect(() => {
    if (mainRef.current) mainRef.current.scrollTop = 0;
  }, [currentPage]);

  return (
    <div className="relative flex h-screen flex-col overflow-hidden bg-app [scrollbar-color:var(--color-app-frame)_transparent] [scrollbar-width:thin] md:flex-row">
      {/* Drag region for macOS traffic lights area */}
      {isMacOS && (
        <div
          data-tauri-drag-region
          className="absolute inset-x-0 top-0 h-[52px] z-50"
        />
      )}

      {/* Sidebar */}
      <nav
        aria-label={t("title")}
        className={clsx(
          "shrink-0 border-b p-2 md:w-48 md:overflow-y-auto md:border-b-0 md:border-r md:p-4",
          "bg-sidebar border-sidebar-line",
          isMacOS && "pt-[52px]"
        )}
      >
        <div className="mb-6 hidden md:block">
          <h1
            className="text-xl font-semibold text-sidebar-ink"
          >
            {t("title")}
          </h1>
        </div>
        <SettingsSidebar currentPage={currentPage} onPageChange={changePage} />
      </nav>

      {/* Main content */}
      <main
        ref={mainRef}
        aria-label={t(`sections.${currentPage}`)}
        className={clsx(
          "min-h-0 min-w-0 flex-1 overflow-auto px-4 py-6 md:px-6 md:py-8",
          isMacOS && "md:pt-[52px]"
        )}
      >
        <div className="mx-auto max-w-3xl">
          <Page />
        </div>
      </main>
    </div>
  );
}

/**
 * Settings component for separate settings window.
 * Renders immediately since daemon is already connected in main window.
 */
export function Settings() {
  const platform = usePlatform();

  useEffect(() => {
    // Apply macOS titlebar styling after window is ready
    if (platform.applyMacOSStyling) {
      platform.applyMacOSStyling().catch((err) => {
        console.warn("Failed to apply macOS styling:", err);
      });
    }
  }, [platform]);

  return <SettingsContentWrapper />;
}
