import React, { useState, useEffect } from "react";
import {
  Box,
  Button,
  TextField,
  BlockStack,
  InlineStack,
  Text,
  ResourceList,
  ResourceItem,
  Thumbnail,
  EmptyState,
  Divider,
} from "@shopify/polaris";
import { Product } from "./settings/PresetQuestions"; // Corrected path
import {
  DeleteIcon,
  ChevronDownIcon,
  ChevronUpIcon,
} from "@shopify/polaris-icons";

export interface QuestionEditBlockProps {
  title: string;
  questionText: string;
  onQuestionTextChange: (text: string) => void;
  productSearchTerm: string;
  onProductSearchTermChange: (term: string) => void;

  products: Product[]; // Live search results
  selectedProducts: Product[]; // Products currently selected for THIS question
  onSelectedProductsChange: (updatedSelectedProducts: Product[]) => void;
  onRemoveSelectedProduct: (productId: string) => void; // New callback to remove a product

  onSave: () => void;
  onCancel: () => void;
}

export function QuestionEditBlock({
  title,
  questionText,
  onQuestionTextChange,
  productSearchTerm,
  onProductSearchTermChange,
  products: searchedProducts,
  selectedProducts,
  onSelectedProductsChange,
  onRemoveSelectedProduct,
  onSave,
  onCancel,
}: QuestionEditBlockProps) {
  const [displaySelectedIdsInSearchList, setDisplaySelectedIdsInSearchList] =
    useState<string[]>([]);
  const [isAddProductsSectionExpanded, setIsAddProductsSectionExpanded] =
    useState(selectedProducts.length == 0);

  useEffect(() => {
    const idsFromSelectedProductsInSearch = selectedProducts
      .filter((sp) => searchedProducts.some((p) => p.id === sp.id))
      .map((p) => p.id);
    setDisplaySelectedIdsInSearchList(idsFromSelectedProductsInSearch);
  }, [selectedProducts, searchedProducts]);

  const handleProductSelectionChangeInSearchList = (
    newlySelectedIdsInSearchList: string[],
  ) => {
    const newlySelectedFromSearch = searchedProducts.filter((p) =>
      newlySelectedIdsInSearchList.includes(p.id),
    );

    const previouslySelectedButNotInCurrentSearch = selectedProducts.filter(
      (p) => !searchedProducts.some((sp) => sp.id === p.id),
    );

    const finalUpdatedSelectedProducts = [
      ...previouslySelectedButNotInCurrentSearch,
      ...newlySelectedFromSearch,
    ];

    onSelectedProductsChange(finalUpdatedSelectedProducts);
  };

  const toggleAddProductsSection = () => {
    setIsAddProductsSectionExpanded(!isAddProductsSectionExpanded);
  };

  const searchResultsEmptyState = productSearchTerm ? (
    <EmptyState
      heading="No products found"
      image="https://cdn.shopify.com/s/files/1/0262/4074/files/empty-state.svg"
    >
      <p>
        No products matched your search term "{productSearchTerm}". Try a
        different search.
      </p>
    </EmptyState>
  ) : (
    <EmptyState
      heading="Search for products to target"
      image="https://cdn.shopify.com/s/files/1/0262/4074/files/empty-state.svg"
    >
      <p>
        Enter a search term above to find products to associate with this
        question.
      </p>
    </EmptyState>
  );

  return (
    <Box
      padding="400"
      borderWidth="025"
      borderColor="border"
      borderRadius="200"
    >
      <BlockStack gap="400">
        <Text variant="headingMd" as="h3">
          {title}
        </Text>
        <TextField
          label="Preset Question Text"
          value={questionText}
          onChange={onQuestionTextChange}
          autoComplete="off"
          multiline={2}
          placeholder="e.g., What materials is this product made of?"
        />

        {/* Section for Displaying Currently Selected Products */}
        {selectedProducts && selectedProducts.length > 0 && (
          <BlockStack gap="300">
            <Divider />
            <Text variant="headingMd" as="h4">
              Targeted Products ({selectedProducts.length})
            </Text>
            <Box
              borderWidth="025"
              borderColor="border"
              borderRadius="100"
              padding="0"
            >
              <ResourceList
                resourceName={{ singular: "product", plural: "products" }}
                items={selectedProducts}
                renderItem={(product: Product) => {
                  const { id, title: productTitle, price, image } = product;
                  const media = image ? (
                    <Thumbnail source={image} alt={productTitle} size="small" />
                  ) : undefined;
                  return (
                    <ResourceItem
                      id={id}
                      media={media}
                      accessibilityLabel={`View ${productTitle}`}
                      onClick={() => {}} // No-op for this list item itself
                    >
                      <InlineStack
                        align="space-between"
                        blockAlign="center"
                        wrap={false}
                        gap="400"
                      >
                        <Text variant="bodyMd" as="p" fontWeight="semibold">
                          {productTitle}
                        </Text>
                        <InlineStack gap="200" blockAlign="center">
                          {typeof price === "number" && (
                            <Text variant="bodyMd" as="p">
                              ${price.toFixed(2)}
                            </Text>
                          )}
                          <Button
                            icon={DeleteIcon}
                            onClick={() => onRemoveSelectedProduct(id)}
                            accessibilityLabel={`Remove ${productTitle}`}
                            variant="tertiary"
                            tone="critical"
                          />
                        </InlineStack>
                      </InlineStack>
                    </ResourceItem>
                  );
                }}
              />
            </Box>
            <Divider />
          </BlockStack>
        )}

        {/* Section for Searching and Adding Products */}
        <BlockStack gap="300">
          <InlineStack align="space-between" blockAlign="center" wrap={false}>
            <Text variant="headingMd" as="h4">
              {selectedProducts && selectedProducts.length > 0
                ? "Add More Products"
                : "Search and Add Target Products"}
            </Text>
            <Button
              onClick={toggleAddProductsSection}
              variant="tertiary"
              icon={
                isAddProductsSectionExpanded ? ChevronUpIcon : ChevronDownIcon
              }
              accessibilityLabel={
                isAddProductsSectionExpanded
                  ? "Collapse product search"
                  : "Expand product search"
              }
            >
              {isAddProductsSectionExpanded ? "Hide" : "Show"}
            </Button>
          </InlineStack>

          {isAddProductsSectionExpanded && (
            <BlockStack gap="300">
              <TextField
                label="Search for products by name."
                value={productSearchTerm}
                onChange={onProductSearchTermChange}
                autoComplete="off"
                placeholder="e.g., T-shirt, Snowboard"
              />
              <Box
                borderWidth="025"
                borderColor="border"
                borderRadius="100"
                padding="0"
                minHeight="200px"
              >
                {searchedProducts.length > 0 || productSearchTerm ? (
                  <ResourceList
                    resourceName={{ singular: "product", plural: "products" }}
                    items={searchedProducts}
                    renderItem={(product: Product) => {
                      const { id, title: productTitle, price, image } = product;
                      const media = image ? (
                        <Thumbnail
                          source={image}
                          alt={productTitle}
                          size="small"
                        />
                      ) : undefined;
                      return (
                        <ResourceItem
                          id={id}
                          media={media}
                          accessibilityLabel={`Select ${productTitle}`}
                          onClick={() => {}} // No-op, selection handled by ResourceList's onSelectionChange
                        >
                          <InlineStack
                            align="space-between"
                            blockAlign="center"
                            wrap={false}
                            gap="400"
                          >
                            <Text variant="bodyMd" as="p" fontWeight="semibold">
                              {productTitle}
                            </Text>
                            {typeof price === "number" && (
                              <Text variant="bodyMd" as="p">
                                ${price.toFixed(2)}
                              </Text>
                            )}
                          </InlineStack>
                        </ResourceItem>
                      );
                    }}
                    selectable
                    selectedItems={displaySelectedIdsInSearchList} // Uses IDs of selected products that are in the current search list
                    onSelectionChange={handleProductSelectionChangeInSearchList}
                    emptyState={
                      searchedProducts.length === 0 && productSearchTerm
                        ? searchResultsEmptyState
                        : undefined
                    }
                  />
                ) : (
                  <Box padding="400">{searchResultsEmptyState}</Box>
                )}
              </Box>
            </BlockStack>
          )}
        </BlockStack>

        <InlineStack gap="200" align="end">
          <Button onClick={onCancel}>Cancel</Button>
          <Button onClick={onSave} variant="primary">
            Save Question
          </Button>
        </InlineStack>
      </BlockStack>
    </Box>
  );
}
