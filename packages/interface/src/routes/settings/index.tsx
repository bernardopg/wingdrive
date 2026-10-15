import { useState, useEffect, useRef } from "react";
import clsx from "clsx";
import {
  Books,
  Cpu,
  GearSix,
  Info,
  MagnifyingGlass,
  PaintBrush,
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
  PrivacySettings,
  AdvancedSettings,
  AboutSettings,
} from "../../Settings/pages";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import { usePlatform } from "../../contexts/PlatformContext";
import { getCurrentPlatform } from "../../util/keybinds/platform";
import { useTranslation } from "../../i18n";

interface SettingsSidebarProps {
  currentPage: string;
  onPageChange: (page: string) => void;
}

const sections: { id: string; icon: Icon }[] = [
  { id: "general", icon: GearSix },
  { id: "appearance", icon: PaintBrush },
  { id: "library", icon: Books },
  { id: "indexer", icon: MagnifyingGlass },
  { id: "services", icon: Cpu },
  { id: "privacy", icon: ShieldCheck },
  { id: "advanced", icon: Wrench },
  { id: "about", icon: Info },
];

// The 52px top inset only makes sense on macOS, where the window has no
// native titlebar and the traffic lights overlay the content. On Linux and
// Windows the window keeps its native decorations, so that padding showed up
// as a dead strip and the drag region swallowed clicks on the nav header.
const isMacOS = getCurrentPlatform() === "macos";

function SettingsSidebar({ currentPage, onPageChange }: SettingsSidebarProps) {
  const { t } = useTranslation();
  const listRef = useRef<HTMLDivElement>(null);

  // On narrow windows the nav is a horizontal strip; keep the open page visible in it
  useEffect(() => {
    listRef.current
      ?.querySelector('[aria-current="page"]')
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [currentPage]);

  return (
    <div ref={listRef} className="flex gap-1 overflow-x-auto sm:flex-col sm:overflow-visible">
      {sections.map(({ id, icon: SectionIcon }) => (
        <button
          key={id}
          type="button"
          aria-current={currentPage === id ? "page" : undefined}
          onClick={() => onPageChange(id)}
          className={clsx(
            "flex shrink-0 items-center gap-2 rounded-md px-3 py-2 text-left text-sm font-medium transition-colors duration-300 sm:w-full",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
            currentPage === id
              ? "bg-sidebar-selected text-sidebar-ink"
              : "text-sidebar-ink-dull hover:text-sidebar-ink hover:bg-sidebar-box"
          )}
        >
          <SectionIcon className="size-4 shrink-0" weight={currentPage === id ? "fill" : "regular"} />
          {t(`sections.${id}`)}
        </button>
      ))}
    </div>
  );
}

interface SettingsContentProps {
  page: string;
}

function SettingsContent({ page }: SettingsContentProps) {
  switch (page) {
    case "general":
      return <GeneralSettings />;
    case "appearance":
      return <AppearanceSettings />;
    case "library":
      return <LibrarySettings />;
    case "indexer":
      return <IndexerSettings />;
    case "services":
      return <ServicesSettings />;
    case "privacy":
      return <PrivacySettings />;
    case "advanced":
      return <AdvancedSettings />;
    case "about":
      return <AboutSettings />;
    default:
      return <GeneralSettings />;
  }
}

function SettingsContentWrapper() {
  const { t } = useTranslation();
  const pathname = window.location.pathname;
  const requested = pathname.split("/").filter(Boolean)[1];
  const initialPage = sections.some((s) => s.id === requested) ? requested! : "general";
  const [currentPage, setCurrentPage] = useState(initialPage);

  // Keep the URL on the open page so a reload or a reopened window lands there
  const changePage = (page: string) => {
    setCurrentPage(page);
    window.history.replaceState(null, "", `/settings/${page}`);
  };

  return (
    <div
      className={clsx(
        "h-screen flex flex-col sm:flex-row bg-app relative"
      )}
    >
      {/* Drag region for macOS traffic lights area */}
      {isMacOS && (
        <div
          data-tauri-drag-region
          className="absolute inset-x-0 top-0 h-[52px] z-50"
        />
      )}

      {/* Sidebar */}
      <nav
        className={clsx(
          "shrink-0 border-b p-2 transition-all duration-500 sm:w-52 sm:border-b-0 sm:border-r sm:p-4",
          "bg-sidebar border-sidebar-line",
          isMacOS && "pt-[52px]"
        )}
      >
        <div className="mb-6 hidden sm:block">
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
        className={clsx(
          "flex-1 overflow-auto px-4 py-6 sm:px-8 sm:py-8",
          isMacOS && "pt-[52px]"
        )}
      >
        <div className={clsx("mx-auto", currentPage !== "about" && "max-w-3xl")}>
          <SettingsContent page={currentPage} />
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

  return (
    <>
      <SettingsContentWrapper />
      <ReactQueryDevtools
        initialIsOpen={false}
        buttonPosition="bottom-right"
      />
    </>
  );
}