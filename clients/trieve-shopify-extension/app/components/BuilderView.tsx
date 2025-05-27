import React, { useState, useCallback, ReactNode } from "react";
import {
  Card,
  Button,
  Modal,
  ResourceList,
  ResourceItem,
  Text,
  BlockStack,
  InlineStack,
  ButtonGroup,
  EmptyState,
  Box,
  InlineGrid,
} from "@shopify/polaris";
import { PlusIcon, DeleteIcon, EditIcon } from "@shopify/polaris-icons";

export interface BuilderViewProps<T extends { id: string }> {
  // Data
  items: T[];
  onItemsChange: (items: T[]) => void;

  // Item rendering
  renderItemContent: (item: T) => ReactNode;
  renderEditForm: (props: EditFormProps<T>) => ReactNode;

  // Labels and text
  labels: {
    singular: string;
    plural: string;
    addButton: string;
    editTitle: string;
    addTitle: string;
    emptyStateHeading: string;
    emptyStateDescription?: string;
    deleteConfirmTitle?: string;
    deleteConfirmMessage?: string;
  };

  cardWrapper?: boolean;
  headerContent?: ReactNode;
  onItemClick?: (item: T) => void;
  validateItem?: (item: T) => boolean;
  onSaveSuccess?: (item: T, isNew: boolean) => void;
  customActions?: (item: T) => ReactNode;

  header?: {
    title: string;
    subtitle: string;
  };
}

export interface EditFormProps<T> {
  item: T;
  onChange: (item: T) => void;
  onSave: () => void;
  onCancel: () => void;
  isNew: boolean;
}

export function BuilderView<T extends { id: string }>({
  items,
  onItemsChange,
  renderItemContent,
  renderEditForm,
  labels,
  cardWrapper = true,
  headerContent,
  onItemClick,
  validateItem,
  onSaveSuccess,
  customActions,
  header,
}: BuilderViewProps<T>) {
  const [editingItemId, setEditingItemId] = useState<string | null>(null);
  const [isAddingNew, setIsAddingNew] = useState(false);
  const [editFormData, setEditFormData] = useState<T | null>(null);
  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [itemIdToDelete, setItemIdToDelete] = useState<string | null>(null);

  const handleSaveItem = useCallback(() => {
    if (!editFormData) return;

    if (validateItem && !validateItem(editFormData)) {
      return;
    }

    let updatedItems: T[];
    const isNew = isAddingNew;

    if (editingItemId) {
      updatedItems = items.map((item) =>
        item.id === editingItemId ? editFormData : item,
      );
    } else if (isAddingNew) {
      updatedItems = [...items, editFormData];
    } else {
      return;
    }

    onItemsChange(updatedItems);
    setEditFormData(null);
    setEditingItemId(null);
    setIsAddingNew(false);

    if (onSaveSuccess) {
      onSaveSuccess(editFormData, isNew);
    }
  }, [
    editFormData,
    editingItemId,
    isAddingNew,
    items,
    onItemsChange,
    validateItem,
    onSaveSuccess,
  ]);

  const handleStartAddNew = useCallback(() => {
    setEditingItemId(null);
    setIsAddingNew(true);
    // Create a new item with just an ID
    const newItem = { id: String(Date.now()) } as T;
    setEditFormData(newItem);
  }, []);

  const handleStartEdit = useCallback((item: T) => {
    setIsAddingNew(false);
    setEditingItemId(item.id);
    setEditFormData({ ...item });
  }, []);

  const handleCancelEdit = useCallback(() => {
    setEditingItemId(null);
    setIsAddingNew(false);
    setEditFormData(null);
  }, []);

  const handleDeleteTrigger = useCallback((id: string) => {
    setItemIdToDelete(id);
    setShowDeleteConfirmModal(true);
  }, []);

  const confirmDelete = useCallback(() => {
    if (itemIdToDelete) {
      const updatedItems = items.filter((item) => item.id !== itemIdToDelete);
      onItemsChange(updatedItems);
    }
    setItemIdToDelete(null);
    setShowDeleteConfirmModal(false);
  }, [itemIdToDelete, items, onItemsChange]);

  const cancelDelete = useCallback(() => {
    setItemIdToDelete(null);
    setShowDeleteConfirmModal(false);
  }, []);

  const renderItem = (item: T) => {
    if (editingItemId === item.id && editFormData) {
      return (
        <Box paddingBlockEnd="400">
          {renderEditForm({
            item: editFormData,
            onChange: setEditFormData,
            onSave: handleSaveItem,
            onCancel: handleCancelEdit,
            isNew: false,
          })}
        </Box>
      );
    }

    return (
      <ResourceItem
        id={item.id}
        accessibilityLabel={`View details for ${labels.singular}`}
        onClick={() => {
          if (onItemClick) {
            onItemClick(item);
          } else {
            handleStartEdit(item);
          }
        }}
      >
        <InlineStack
          wrap={false}
          blockAlign="center"
          align="space-between"
          gap="200"
        >
          <Box minWidth="0" maxWidth="calc(100% - 100px)" width="100%">
            {renderItemContent(item)}
          </Box>
          <ButtonGroup>
            {customActions && customActions(item)}
            <Button
              icon={EditIcon}
              accessibilityLabel={`Edit ${labels.singular}`}
              onClick={() => {
                handleStartEdit(item);
              }}
              variant="tertiary"
            />
            <Button
              icon={DeleteIcon}
              accessibilityLabel={`Delete ${labels.singular}`}
              onPointerDown={(e: React.PointerEvent) => {
                e.stopPropagation();
                handleDeleteTrigger(item.id);
              }}
              variant="tertiary"
              tone="critical"
            />
          </ButtonGroup>
        </InlineStack>
      </ResourceItem>
    );
  };

  const emptyStateMarkup = (
    <EmptyState
      heading={labels.emptyStateHeading}
      action={{
        content: labels.addButton,
        icon: PlusIcon,
        onAction: handleStartAddNew,
      }}
      image="https://cdn.shopify.com/s/files/1/0262/4071/2726/files/emptystate-files.png"
    >
      {labels.emptyStateDescription && (
        <Text as="p" variant="bodyMd">
          {labels.emptyStateDescription}
        </Text>
      )}
    </EmptyState>
  );

  const content = (
    <BlockStack gap="400">
      {headerContent}

      {items.length > 0 ? (
        <>
          <ResourceList
            resourceName={{ singular: labels.singular, plural: labels.plural }}
            items={items}
            renderItem={renderItem}
          />
        </>
      ) : (
        !isAddingNew && emptyStateMarkup
      )}

      {isAddingNew && editFormData && (
        <Box paddingBlockStart="400" paddingBlockEnd="400">
          {renderEditForm({
            item: editFormData,
            onChange: setEditFormData,
            onSave: handleSaveItem,
            onCancel: handleCancelEdit,
            isNew: true,
          })}
        </Box>
      )}

      {!isAddingNew && !editingItemId && items.length > 0 && (
        <InlineStack align="end">
          <ButtonGroup>
            <Button
              onClick={handleStartAddNew}
              variant="primary"
              icon={PlusIcon}
            >
              {labels.addButton}
            </Button>
          </ButtonGroup>
        </InlineStack>
      )}
    </BlockStack>
  );

  const listContent = cardWrapper ? <Card>{content}</Card> : content;

  const finalContent = header ? (
    <Box paddingInline="400">
      <BlockStack gap={{ xs: "800", sm: "400" }}>
        <InlineGrid columns={{ xs: "1fr", md: "2fr 5fr" }} gap="400">
          <Box
            as="section"
            paddingInlineStart={{ xs: "400", sm: "0" }}
            paddingInlineEnd={{ xs: "400", sm: "0" }}
          >
            <BlockStack gap="400">
              <Text as="h3" variant="headingMd">
                {header.title}
              </Text>
              <Text as="p" variant="bodyMd">
                {header.subtitle}
              </Text>
            </BlockStack>
          </Box>
          {listContent}
        </InlineGrid>
      </BlockStack>
    </Box>
  ) : (
    listContent
  );

  return (
    <>
      {finalContent}

      <Modal
        open={showDeleteConfirmModal}
        onClose={cancelDelete}
        title={labels.deleteConfirmTitle || "Confirm Deletion"}
        primaryAction={{
          content: "Delete",
          onAction: confirmDelete,
          destructive: true,
        }}
        secondaryActions={[
          {
            content: "Cancel",
            onAction: cancelDelete,
          },
        ]}
      >
        <Modal.Section>
          <BlockStack>
            <Text as="p">
              {labels.deleteConfirmMessage ||
                `Are you sure you want to delete this ${labels.singular}?`}
            </Text>
          </BlockStack>
        </Modal.Section>
      </Modal>
    </>
  );
}
