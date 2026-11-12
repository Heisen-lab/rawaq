import React from "react";
import { useI18n } from "../i18n/I18nContext";
import { BdiText } from "./BdiText";
import { DesktopBox, type DesktopBoxModel, type FolderStateInfo } from "./DesktopBox";

export interface BoxGroupTabsProps {
  groupName: string;
  groupId: number;
  boxes: DesktopBoxModel[];
  folderStates: Record<number, FolderStateInfo>;
  activeBoxId: number;
  onSelectBox: (boxId: number) => void;
  onDissolveGroup: (groupId: number) => void;
  onToggleCapsule: (boxId?: number) => void;
  onChangeDisplayMode: (boxId: number | undefined, mode: "grid" | "list") => void;
  onChangeSortOrder: (boxId: number | undefined, sort: "name" | "type" | "date" | "size") => void;
}

export function BoxGroupTabs({
  groupName,
  groupId,
  boxes,
  folderStates,
  activeBoxId,
  onSelectBox,
  onDissolveGroup,
  onToggleCapsule,
  onChangeDisplayMode,
  onChangeSortOrder,
}: BoxGroupTabsProps): React.ReactElement {
  const { t } = useI18n();

  const activeBox = boxes.find((b) => b.id === activeBoxId) ?? boxes[0];
  const activeState = (activeBox && activeBox.id !== undefined && folderStates[activeBox.id]) || {
    status: "Accessible" as const,
    items: [],
  };

  return (
    <section
      className="rawaq-box-group"
      data-testid={`box-group-${groupId}`}
      aria-label={t("boxes.folderGroupTab", { name: groupName })}
    >
      <header className="rawaq-group-header">
        <div className="rawaq-group-tabs-list" role="tablist">
          {boxes.map((b) => {
            const isSelected = b.id === activeBoxId;
            return (
              <button
                key={b.id}
                type="button"
                role="tab"
                aria-selected={isSelected}
                className={`rawaq-tab-btn ${isSelected ? "active" : ""}`}
                data-testid={`group-tab-${b.id}`}
                onClick={() => b.id !== undefined && onSelectBox(b.id)}
              >
                <BdiText text={b.name} />
              </button>
            );
          })}
        </div>

        <button
          type="button"
          className="rawaq-dissolve-btn"
          data-testid={`dissolve-group-btn-${groupId}`}
          onClick={() => onDissolveGroup(groupId)}
          title={t("boxes.dissolveGroup")}
        >
          {t("boxes.dissolveGroup")}
        </button>
      </header>

      {activeBox && (
        <DesktopBox
          box={activeBox}
          folderState={activeState}
          onToggleCapsule={onToggleCapsule}
          onChangeDisplayMode={onChangeDisplayMode}
          onChangeSortOrder={onChangeSortOrder}
        />
      )}
    </section>
  );
}
