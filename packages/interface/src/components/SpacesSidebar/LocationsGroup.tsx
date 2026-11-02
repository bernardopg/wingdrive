import type { LocationsListOutput } from "@wingdrive/ts-client";
import { useNormalizedQuery } from "@wingdrive/ts-client";
import { SpaceItem } from "./SpaceItem";
import { SortableDynamicList } from "./SortableDynamicList";
import { GroupHeader } from "./GroupHeader";
import { useTranslation } from "../../i18n";

interface LocationsGroupProps {
  groupId: string;
  isCollapsed: boolean;
  onToggle: () => void;
  sortableAttributes?: any;
  sortableListeners?: any;
}

export function LocationsGroup({
  groupId,
  isCollapsed,
  onToggle,
  sortableAttributes,
  sortableListeners,
}: LocationsGroupProps) {
  const { t } = useTranslation("navigation");
  const { data: locationsData } = useNormalizedQuery<null, LocationsListOutput>({
    query: "locations.list",
    input: null, // Unit struct serializes as null, not {}
    resourceType: "location",
  });

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
          {(location, index) => (
            <SpaceItem
              key={location.id}
              item={location}
              allowInsertion={false}
              isLastItem={index === locations.length - 1}
            />
          )}
          </SortableDynamicList>
        </div>
      )}
    </div>
  );
}
