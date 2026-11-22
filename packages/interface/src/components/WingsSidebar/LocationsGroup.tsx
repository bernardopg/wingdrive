import { useEffect } from "react";
import type { IntegrationAccountInfo, LocationsListOutput } from "@wingdrive/ts-client";
import { useNormalizedQuery } from "@wingdrive/ts-client";
import { WarningCircle } from "@phosphor-icons/react";
import { useQueryClient } from "@tanstack/react-query";
import { WingItem } from "./WingItem";
import { SortableDynamicList } from "./SortableDynamicList";
import { GroupHeader } from "./GroupHeader";
import { useLibraryQuery, useWingDriveClient } from "../../contexts/WingDriveContext";
import { useIntegrationConnect } from "../../hooks/useIntegrationConnect";
import { useTranslation } from "../../i18n";

interface LocationsGroupProps {
  groupId: string;
  isCollapsed: boolean;
  onToggle: () => void;
  sortableAttributes?: any;
  sortableListeners?: any;
}

function cloudLocationAccount(
  location: LocationsListOutput["locations"][number],
  accounts: IntegrationAccountInfo[] | undefined,
): IntegrationAccountInfo | undefined {
  const path = location.wing_path;
  if (!("Cloud" in path)) return undefined;
  const { service, identifier } = path.Cloud;
  return accounts?.find(
    (account) =>
      account.needs_reconnect &&
      account.drive?.identifier === identifier &&
      ((service === "gdrive" && account.provider === "google") ||
        (service === "onedrive" && account.provider === "microsoft")),
  );
}

function ReconnectAction({ account }: { account: IntegrationAccountInfo }) {
  const { t } = useTranslation();
  const { state, start, isStarting } = useIntegrationConnect();
  const waiting = state.phase === "waiting" || isStarting;
  if (account.provider !== "google") {
    return (
      <span
        className="mr-1 rounded bg-status-warning/15 px-1.5 py-0.5 text-[10px] font-medium text-status-warning"
        title={t("integrations.accounts.reconnectNeeded")}
      >
        {t("integrations.accounts.reconnect")}
      </span>
    );
  }
  return (
    <button
      type="button"
      title={t("integrations.accounts.reconnectNeeded")}
      disabled={waiting}
      onClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        void start({
          provider: "google",
          mode: account.mode,
          services: account.services.length > 0 ? account.services : ["drive"],
          loginHint: account.email,
        });
      }}
      className="mr-1 flex items-center gap-1 rounded bg-status-warning/15 px-1.5 py-0.5 text-[10px] font-medium text-status-warning hover:bg-status-warning/25 disabled:opacity-60"
    >
      <WarningCircle size={12} weight="fill" />
      {waiting ? t("integrations.accounts.waiting") : t("integrations.accounts.reconnect")}
    </button>
  );
}

export function LocationsGroup({
  groupId,
  isCollapsed,
  onToggle,
  sortableAttributes,
  sortableListeners,
}: LocationsGroupProps) {
  const { t } = useTranslation("navigation");
  const { t: tSettings } = useTranslation();
  const client = useWingDriveClient();
  const queryClient = useQueryClient();
  const libraryId = client.getCurrentLibraryId();
  const { data: locationsData } = useNormalizedQuery<null, LocationsListOutput>({
    query: "locations.list",
    input: null, // Unit struct serializes as null, not {}
    resourceType: "location",
  });
  const accounts = useLibraryQuery(
    { type: "integrations.accounts.list", input: {} },
    { enabled: !!libraryId },
  );

  useEffect(() => {
    if (!libraryId) return;
    let cancelled = false;
    let unsubscribe: (() => void) | undefined;
    void client
      .subscribe((event) => {
        if (
          typeof event === "object" &&
          event !== null &&
          "IntegrationAccountChanged" in event
        ) {
          const changed = (
            event as { IntegrationAccountChanged: { library_id: string } }
          ).IntegrationAccountChanged;
          if (changed.library_id === libraryId) {
            void queryClient.invalidateQueries({
              queryKey: ["integrations.accounts.list"],
            });
          }
        }
      })
      .then((unsub) => {
        if (cancelled) unsub();
        else unsubscribe = unsub;
      });
    return () => {
      cancelled = true;
      unsubscribe?.();
    };
  }, [client, libraryId, queryClient]);

  const locations = locationsData?.locations ?? [];

  return (
    <div>
      <GroupHeader
        label={t("sidebar.locations")}
        isCollapsed={isCollapsed}
        onToggle={onToggle}
        sortableAttributes={sortableAttributes}
        sortableListeners={sortableListeners}
      />

      {/* Items */}
      {!isCollapsed && (
        <div className="space-y-0.5">
          <SortableDynamicList groupId={groupId} items={locations} label={(location) => location.name}>
          {(location, index) => {
            const reconnectAccount = cloudLocationAccount(location, accounts.data);
            return (
              <WingItem
                key={location.id}
                item={location}
                allowInsertion={false}
                isLastItem={index === locations.length - 1}
                rightAction={
                  reconnectAccount ? <ReconnectAction account={reconnectAccount} /> : undefined
                }
              />
            );
          }}
          </SortableDynamicList>
        </div>
      )}
    </div>
  );
}
