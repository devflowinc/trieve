import { useState, useCallback } from "react";
import {
  Button,
  Popover,
  ActionList,
  Text,
  ActionListItemProps,
} from "@shopify/polaris";
import { formatDistanceToNow } from "date-fns";

export type ThemeChoice = {
  name: string;
  prefix: string;
  id: string;
  role: string;
  updatedAt: string;
};

interface ThemeSelectProps {
  themes: ThemeChoice[];
  selectedTheme: ThemeChoice | null;
  onChange: (theme: ThemeChoice) => void;
  disabled?: boolean;
}

export function ThemeSelect({
  themes,
  selectedTheme,
  onChange,
  disabled = false,
}: ThemeSelectProps) {
  const [popoverActive, setPopoverActive] = useState(false);

  const togglePopoverActive = useCallback(() => {
    setPopoverActive((active) => !active);
  }, []);

  const handleThemeSelect = useCallback(
    (theme: ThemeChoice) => {
      onChange(theme);
      setPopoverActive(false);
    },
    [onChange],
  );

  const activator = (
    <Button
      onClick={togglePopoverActive}
      disclosure
      disabled={disabled || themes.length < 2}
    >
      {selectedTheme?.name || "Select theme"}
    </Button>
  );

  const themeItems = themes.map(
    (theme) =>
      ({
        content: theme.name,
        onAction: () => handleThemeSelect(theme),
        active: theme.id === selectedTheme?.id,
        disabled: disabled,
        helpText:
          "Updated " +
          formatDistanceToNow(new Date(theme.updatedAt), {
            addSuffix: true,
          }),
      }) satisfies ActionListItemProps,
  );

  return (
    <div>
      <Text as="p" variant="bodyMd">
        Theme
      </Text>
      <Popover
        active={popoverActive}
        activator={activator}
        autofocusTarget="first-node"
        onClose={togglePopoverActive}
        fluidContent
      >
        <ActionList actionRole="menuitem" items={themeItems} />
      </Popover>
    </div>
  );
}
