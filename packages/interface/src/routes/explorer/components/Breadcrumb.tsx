import clsx from "clsx";
import type { WingPath } from "@wingdrive/ts-client";

interface BreadcrumbProps {
  path: WingPath;
  onNavigate: (path: WingPath) => void;
}

interface PathSegment {
  name: string;
  path: WingPath;
}

function parseWingPathSegments(wdPath: WingPath): PathSegment[] {
  if ("Physical" in wdPath) {
    const { device_slug, path } = wdPath.Physical;
    const parts = path.split("/").filter(Boolean);

    return parts.map((part, index) => ({
      name: part,
      path: {
        Physical: {
          device_slug,
          path: "/" + parts.slice(0, index + 1).join("/"),
        },
      },
    }));
  }

  if ("Cloud" in wdPath) {
    const { service, identifier, path } = wdPath.Cloud;
    const parts = path.split("/").filter(Boolean);

    return parts.map((part, index) => ({
      name: part,
      path: {
        Cloud: {
          service,
          identifier,
          path: parts.slice(0, index + 1).join("/"),
        },
      },
    }));
  }

  return [];
}

export function Breadcrumb({ path, onNavigate }: BreadcrumbProps) {
  const segments = parseWingPathSegments(path);

  return (
    <div className="flex items-center gap-1 text-sm">
      {segments.map((segment, index) => {
        const isLast = index === segments.length - 1;
        return (
          <div key={index} className="flex items-center gap-1">
            {index > 0 && <span className="text-ink-faint">/</span>}
            <button
              onClick={() => !isLast && onNavigate(segment.path)}
              disabled={isLast}
              className={clsx(
                isLast
                  ? "text-ink font-medium cursor-default"
                  : "text-ink-dull hover:text-ink cursor-pointer"
              )}
            >
              {segment.name}
            </button>
          </div>
        );
      })}
    </div>
  );
}