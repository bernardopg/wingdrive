import { useState, useRef, useEffect } from "react";
import { createPortal } from "react-dom";
import {
  SortAscending,
  TextAa,
  CalendarBlank,
  Ruler,
  FileText,
  Check,
  Camera,
  ArrowUp,
  ArrowDown,
} from "@phosphor-icons/react";
import { motion, AnimatePresence } from "framer-motion";
import clsx from "clsx";
import { CircleButton } from "@wingdrive/primitives";
import type { DirectorySortBy, MediaSortBy, SortDirection } from "@wingdrive/ts-client";
import { useTranslation } from "../../i18n";

interface SortMenuPanelProps {
  sortBy: DirectorySortBy | MediaSortBy;
  /** Shown on the active option; selecting it again flips the direction. */
  sortDirection: SortDirection;
  onSortChange: (sort: DirectorySortBy | MediaSortBy) => void;
  viewMode: "grid" | "list" | "media" | "column";
}

export function SortMenuPanel({ sortBy, sortDirection, onSortChange, viewMode }: SortMenuPanelProps) {
  const { t } = useTranslation("explorer");
  // Media listings have their own fixed ordering and ignore the direction.
  const ActiveIcon =
    viewMode === "media" ? Check : sortDirection === "Asc" ? ArrowUp : ArrowDown;
  const sortOptions = viewMode === "media"
    ? [
        { value: "datetaken", label: t("sort.dateTaken"), icon: Camera },
        { value: "modified", label: t("sort.dateModified"), icon: CalendarBlank },
        { value: "created", label: t("sort.dateCreated"), icon: CalendarBlank },
        { value: "name", label: t("sort.name"), icon: TextAa },
        { value: "size", label: t("sort.size"), icon: Ruler },
      ]
    : [
        { value: "name", label: t("sort.name"), icon: TextAa },
        { value: "modified", label: t("sort.dateModified"), icon: CalendarBlank },
        { value: "size", label: t("sort.size"), icon: Ruler },
        { value: "type", label: t("sort.type"), icon: FileText },
      ];

  return (
    <div className="w-56 bg-app-box border border-app-line rounded-lg shadow-lg overflow-hidden">
      <div className="px-3 py-2 text-xs font-semibold text-sidebar-ink uppercase tracking-wider border-b border-app-line">
        {t("sort.sortBy")}
      </div>

      <div className="py-1">
        {sortOptions.map((option) => {
          const Icon = option.icon;
          const isActive = sortBy === option.value;

          return (
            <button
              key={option.value}
              onClick={() => {
                onSortChange(option.value as DirectorySortBy | MediaSortBy);
              }}
              className={clsx(
                "flex items-center gap-2.5 w-full px-3 py-2 hover:bg-app-hover transition-colors text-sm",
                isActive ? "text-accent" : "text-ink"
              )}
            >
              <Icon className="size-4" weight="bold" />
              <span className="flex-1 text-left">{option.label}</span>
              {isActive && <ActiveIcon className="size-4" weight="bold" />}
            </button>
          );
        })}
      </div>
    </div>
  );
}

interface SortMenuProps {
  sortBy: DirectorySortBy | MediaSortBy;
  sortDirection: SortDirection;
  onSortChange: (sort: DirectorySortBy | MediaSortBy) => void;
  viewMode: "grid" | "list" | "media" | "column";
  className?: string;
}

export function SortMenu({ sortBy, sortDirection, onSortChange, viewMode, className }: SortMenuProps) {
  const { t } = useTranslation("explorer");
  const [isOpen, setIsOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ top: 0, right: 0 });

  // Update position when opened
  useEffect(() => {
    if (isOpen && buttonRef.current) {
      const rect = buttonRef.current.getBoundingClientRect();
      setPosition({
        top: rect.bottom + 8,
        right: window.innerWidth - rect.right,
      });
    }
  }, [isOpen]);

  // Close on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        panelRef.current &&
        buttonRef.current &&
        !panelRef.current.contains(e.target as Node) &&
        !buttonRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  return (
    <>
      <div className={clsx(className)}>
        <CircleButton
          ref={buttonRef}
          icon={SortAscending}
          onClick={() => setIsOpen(!isOpen)}
          active={isOpen}
          title={t("sort.title")}
        />
      </div>

      {isOpen &&
        createPortal(
          <AnimatePresence>
            <motion.div
              ref={panelRef}
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.15 }}
              style={{
                position: "fixed",
                top: `${position.top}px`,
                right: `${position.right}px`,
              }}
              className="z-50"
            >
              <SortMenuPanel
                sortBy={sortBy}
                sortDirection={sortDirection}
                onSortChange={onSortChange}
                viewMode={viewMode}
              />
            </motion.div>
          </AnimatePresence>,
          document.body
        )}
    </>
  );
}
