import { useState } from "react";
import { describe, expect, it } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { I18nProvider } from "../i18n/I18nContext";
import {
  DesktopBox,
  type DesktopBoxModel,
  type FolderStateInfo,
} from "../components/DesktopBox";
import { BoxGroupTabs } from "../components/BoxGroupTabs";

describe("Stage 3 Desktop Boxes, Capsule Mode & Groups UI", () => {
  const sampleBox: DesktopBoxModel = {
    id: 101,
    name: "مستندات العمل 2026",
    folder_path: "/home/ahmad/Documents/مستندات_العمل",
    display_mode: "grid",
    is_capsule_collapsed: false,
    sort_order: "name",
    position_x: 100,
    position_y: 100,
    width: 320,
    height: 400,
  };

  const sampleState: FolderStateInfo = {
    status: "Accessible",
    items: [
      {
        name: "تقرير_الميزانية_2026.pdf",
        path: "/home/ahmad/Documents/مستندات_العمل/تقرير_الميزانية_2026.pdf",
        is_dir: false,
        size_bytes: 1048576,
        modified_timestamp: 1700000000,
        extension: "pdf",
      },
      {
        name: "Project_Notes_ملاحظات.docx",
        path: "/home/ahmad/Documents/مستندات_العمل/Project_Notes_ملاحظات.docx",
        is_dir: false,
        size_bytes: 204800,
        modified_timestamp: 1700005000,
        extension: "docx",
      },
      {
        name: "مجلد_الأرشيف",
        path: "/home/ahmad/Documents/مستندات_العمل/مجلد_الأرشيف",
        is_dir: true,
        size_bytes: 0,
        modified_timestamp: 1700010000,
      },
    ],
  };

  function TestHarness({
    initialBox = sampleBox,
    initialState = sampleState,
    initialLocale = "ar" as "ar" | "en",
  }) {
    const [box, setBox] = useState<DesktopBoxModel>(initialBox);

    return (
      <I18nProvider initialLocale={initialLocale}>
        <DesktopBox
          box={box}
          folderState={initialState}
          onToggleCapsule={() =>
            setBox((prev) => ({
              ...prev,
              is_capsule_collapsed: !prev.is_capsule_collapsed,
            }))
          }
          onChangeDisplayMode={(_id, mode) =>
            setBox((prev) => ({ ...prev, display_mode: mode }))
          }
          onChangeSortOrder={(_id, sort) =>
            setBox((prev) => ({ ...prev, sort_order: sort }))
          }
        />
      </I18nProvider>
    );
  }

  it("ui::test_capsule_collapse_expand_and_rtl_layout in Arabic (RTL)", () => {
    render(<TestHarness initialLocale="ar" />);

    // Initially expanded box in RTL
    const boxEl = screen.getByTestId("desktop-box-101");
    expect(boxEl).toBeInTheDocument();
    expect(document.documentElement.dir).toBe("rtl");
    expect(screen.getByTestId("box-title-101")).toHaveTextContent(
      "مستندات العمل 2026",
    );

    // Verify 3 items rendered
    expect(screen.getByTestId("box-item-101-0")).toBeInTheDocument();
    expect(screen.getByTestId("box-item-101-1")).toBeInTheDocument();
    expect(screen.getByTestId("box-item-101-2")).toBeInTheDocument();

    // Verify mixed Arabic-English file names are wrapped in <bdi dir="auto">
    const item0Bdi = screen.getByTestId("box-item-name-101-0");
    expect(item0Bdi.tagName.toLowerCase()).toBe("bdi");
    expect(item0Bdi).toHaveAttribute("dir", "auto");

    // Click collapse button -> transitions to Capsule Mode
    const collapseBtn = screen.getByTestId("capsule-collapse-btn-101");
    fireEvent.click(collapseBtn);

    // Desktop box should now render as Capsule Bar
    expect(screen.queryByTestId("desktop-box-101")).not.toBeInTheDocument();
    const capsuleBar = screen.getByTestId("capsule-bar-101");
    expect(capsuleBar).toBeInTheDocument();
    expect(screen.getByTestId("capsule-title-101")).toHaveTextContent(
      "مستندات العمل 2026",
    );
    expect(screen.getByTestId("capsule-badge-101")).toHaveTextContent(
      "3 عنصر",
    );

    // Expand back to full box
    const expandBtn = screen.getByTestId("capsule-expand-btn-101");
    fireEvent.click(expandBtn);

    expect(screen.getByTestId("desktop-box-101")).toBeInTheDocument();
    expect(screen.queryByTestId("capsule-bar-101")).not.toBeInTheDocument();
  });

  it("ui::test_capsule_collapse_expand_and_rtl_layout in English (LTR)", () => {
    render(<TestHarness initialLocale="en" />);

    expect(document.documentElement.dir).toBe("ltr");
    const boxEl = screen.getByTestId("desktop-box-101");
    expect(boxEl).toBeInTheDocument();

    // Switch display mode to list
    const listBtn = screen.getByTestId("mode-list-btn-101");
    fireEvent.click(listBtn);
    const content = screen.getByTestId("box-content-101");
    expect(content).toHaveClass("list");

    // Collapse into capsule in LTR
    fireEvent.click(screen.getByTestId("capsule-collapse-btn-101"));
    const capsuleBar = screen.getByTestId("capsule-bar-101");
    expect(capsuleBar).toBeInTheDocument();
    expect(screen.getByTestId("capsule-badge-101")).toHaveTextContent(
      "3 items",
    );

    // Expand back
    fireEvent.click(screen.getByTestId("capsule-expand-btn-101"));
    expect(screen.getByTestId("desktop-box-101")).toBeInTheDocument();
  });

  it("handles missing or disconnected folder gracefully with clear warning banner", () => {
    const disconnectedState: FolderStateInfo = {
      status: "MissingOrDisconnected",
      path: "/mnt/external_usb/MissingFolder",
      message: "Folder disconnected",
    };

    render(
      <TestHarness
        initialBox={{
          ...sampleBox,
          folder_path: "/mnt/external_usb/MissingFolder",
        }}
        initialState={disconnectedState}
      />,
    );

    // Box remains rendered without crashing
    expect(screen.getByTestId("desktop-box-101")).toBeInTheDocument();
    const banner = screen.getByTestId("disconnected-banner-101");
    expect(banner).toBeInTheDocument();
    expect(screen.getByTestId("disconnected-path-101")).toHaveTextContent(
      "/mnt/external_usb/MissingFolder",
    );

    // Normal box items are not rendered when disconnected
    expect(screen.queryByTestId("box-content-101")).not.toBeInTheDocument();
  });

  it("BoxGroupTabs allows switching between tabbed boxes and dissolving the group", () => {
    const boxA: DesktopBoxModel = { ...sampleBox, id: 1, name: "الصندوق الأول" };
    const boxB: DesktopBoxModel = {
      ...sampleBox,
      id: 2,
      name: "الصندوق الثاني",
      display_mode: "list",
    };

    let dissolved = false;
    function GroupHarness() {
      const [activeId, setActiveId] = useState(1);
      return (
        <I18nProvider initialLocale="ar">
          <BoxGroupTabs
            groupName="مجموعة العمل"
            groupId={10}
            boxes={[boxA, boxB]}
            folderStates={{
              1: sampleState,
              2: { status: "Accessible", items: [] },
            }}
            activeBoxId={activeId}
            onSelectBox={(id) => setActiveId(id)}
            onDissolveGroup={(_gid) => {
              dissolved = true;
            }}
            onToggleCapsule={() => {}}
            onChangeDisplayMode={() => {}}
            onChangeSortOrder={() => {}}
          />
        </I18nProvider>
      );
    }

    render(<GroupHarness />);

    expect(screen.getByTestId("box-group-10")).toBeInTheDocument();
    expect(screen.getByTestId("group-tab-1")).toHaveClass("active");
    expect(screen.getByTestId("box-title-1")).toHaveTextContent(
      "الصندوق الأول",
    );

    // Switch to tab 2
    fireEvent.click(screen.getByTestId("group-tab-2"));
    expect(screen.getByTestId("group-tab-2")).toHaveClass("active");
    expect(screen.getByTestId("box-title-2")).toHaveTextContent(
      "الصندوق الثاني",
    );

    // Click dissolve
    fireEvent.click(screen.getByTestId("dissolve-group-btn-10"));
    expect(dissolved).toBe(true);
  });
});
