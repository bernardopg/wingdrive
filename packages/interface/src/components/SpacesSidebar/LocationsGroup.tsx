import { useNormalizedQuery } from "@wingdrive/ts-client";
import { SpaceItem } from "./SpaceItem";
import { GroupHeader } from "./GroupHeader";
import { useTranslation } from "../../i18n";

interface LocationsGroupProps {
  isCollapsed: boolean;
  onToggle: () => void;
  sortableAttributes?: any;
  sortableListeners?: any;
}

export function LocationsGroup({
  isCollapsed,
  onToggle,
  sortableAttributes,
  sortableListeners,
}: LocationsGroupProps) {
  const { t } = useTranslation("navigation");
  const { data: locationsData } = useNormalizedQuery({
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
          {locations.map((location: any, index: number) => (
            <SpaceItem
              key={location.id}
              item={location}
              allowInsertion={false}
              isLastItem={index === locations.length - 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
