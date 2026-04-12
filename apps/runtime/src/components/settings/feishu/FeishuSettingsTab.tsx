import type { ComponentProps } from "react";
import { FeishuAdvancedConsoleSection } from "./FeishuAdvancedConsoleSection";
import { FeishuAdvancedSection } from "./FeishuAdvancedSection";
import { FeishuSettingsSection } from "./FeishuSettingsSection";

interface FeishuSettingsTabProps {
  onOpenEmployees?: () => void;
  settingsSectionProps: ComponentProps<typeof FeishuSettingsSection>;
  advancedConsoleSectionProps: ComponentProps<typeof FeishuAdvancedConsoleSection>;
  advancedSectionProps: ComponentProps<typeof FeishuAdvancedSection>;
}

export function FeishuSettingsTab({
  onOpenEmployees,
  settingsSectionProps,
  advancedConsoleSectionProps,
  advancedSectionProps,
}: FeishuSettingsTabProps) {
  return (
    <div className="space-y-3">
      <FeishuSettingsSection onOpenEmployees={onOpenEmployees} {...settingsSectionProps} />
      <FeishuAdvancedConsoleSection
        onOpenEmployees={onOpenEmployees}
        {...advancedConsoleSectionProps}
      />
      <FeishuAdvancedSection {...advancedSectionProps} />
    </div>
  );
}
