import React, { useState, useCallback, useEffect } from "react";
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
} from "@shopify/polaris";
import { PlusIcon, DeleteIcon, EditIcon } from "@shopify/polaris-icons";
import { useSubmit } from "@remix-run/react";
import { QuestionEditBlock } from "../QuestionEditBlock";
import { useTrieve } from "app/context/trieveContext";
import { ChunkMetadata } from "trieve-ts-sdk";

export interface PresetQuestion {
  id: string;
  questionText: string;
  products?: Product[];
  productSearchTerm?: string;
}

export interface PresetQuestionsProps {
  initialQuestions: PresetQuestion[];
}

interface QuestionFormData {
  questionText: string;
  products?: Product[];
  productSearchTerm?: string;
}

export interface Product {
  id: string;
  image: string;
  price: number;
  url: string;
  title: string;
  groupId: string;
}

const defaultQuestionFormData: QuestionFormData = {
  questionText: "",
  products: [],
  productSearchTerm: "",
};

export function PresetQuestions({ initialQuestions }: PresetQuestionsProps) {
  const submit = useSubmit();
  const trieve = useTrieve();

  const [presetQuestions, setPresetQuestions] = useState<PresetQuestion[]>(
    initialQuestions || [],
  );

  const [editingQuestionId, setEditingQuestionId] = useState<string | null>(
    null,
  );
  const [isAddingNewQuestion, setIsAddingNewQuestion] = useState(false);
  const [editFormData, setEditFormData] = useState<QuestionFormData>(
    defaultQuestionFormData,
  );

  const [showDeleteConfirmModal, setShowDeleteConfirmModal] = useState(false);
  const [questionIdToDelete, setQuestionIdToDelete] = useState<string | null>(
    null,
  );
  const [searchedProducts, setSearchedProducts] = useState<Product[]>([]);

  const handleQuestionSelectedProductsChange = useCallback(
    (updatedSelectedProducts: Product[]) => {
      setEditFormData((prev) => ({
        ...prev,
        products: updatedSelectedProducts,
      }));
    },
    [],
  );

  const handleRemoveSelectedProduct = useCallback(
    (productIdToRemove: string) => {
      setEditFormData((prev) => ({
        ...prev,
        products:
          prev.products?.filter((p) => p.id !== productIdToRemove) || [],
      }));
    },
    [],
  );

  const submitPresetQuestions = useCallback(
    (questions: PresetQuestion[]) => {
      submit(
        {
          type: "preset-questions",
          presetQuestions: JSON.stringify(questions),
        },
        {
          method: "POST",
        },
      );
    },
    [submit],
  );

  const handleSaveQuestion = useCallback(() => {
    let updatedQuestions;
    if (editingQuestionId) {
      updatedQuestions = presetQuestions.map((q) =>
        q.id === editingQuestionId
          ? {
              ...q,
              questionText: editFormData.questionText,
              products: editFormData.products,
              productSearchTerm: "",
            }
          : q,
      );
    } else if (isAddingNewQuestion) {
      const newQuestion: PresetQuestion = {
        id: String(Date.now()),
        questionText: editFormData.questionText,
        products: editFormData.products,
        productSearchTerm: "",
      };
      updatedQuestions = [...presetQuestions, newQuestion];
    } else {
      return;
    }

    setPresetQuestions(updatedQuestions);
    submitPresetQuestions(updatedQuestions);
    setEditingQuestionId(null);
    setIsAddingNewQuestion(false);
    setEditFormData(defaultQuestionFormData);
    setSearchedProducts([]);
  }, [
    editFormData,
    editingQuestionId,
    isAddingNewQuestion,
    presetQuestions,
    submitPresetQuestions,
  ]);

  const handleStartAddNewQuestion = useCallback(() => {
    setEditingQuestionId(null);
    setEditFormData(defaultQuestionFormData);
    setIsAddingNewQuestion(true);
    setSearchedProducts([]);
  }, []);

  const handleStartEditQuestion = useCallback((question: PresetQuestion) => {
    setIsAddingNewQuestion(false);
    setEditingQuestionId(question.id);
    setEditFormData({
      questionText: question.questionText,
      products: question.products || [],
      productSearchTerm: "",
    });
    setSearchedProducts([]);
  }, []);

  const handleCancelEdit = useCallback(() => {
    setEditingQuestionId(null);
    setIsAddingNewQuestion(false);
    setEditFormData(defaultQuestionFormData);
    setSearchedProducts([]);
  }, []);

  const handleDeleteQuestionTrigger = useCallback((idToDelete: string) => {
    setQuestionIdToDelete(idToDelete);
    setShowDeleteConfirmModal(true);
  }, []);

  const confirmDeleteQuestion = useCallback(() => {
    if (questionIdToDelete) {
      const newPresetQuestions = presetQuestions.filter(
        (question) => question.id !== questionIdToDelete,
      );
      setPresetQuestions(newPresetQuestions);
      submitPresetQuestions(newPresetQuestions);
    }
    setQuestionIdToDelete(null);
    setShowDeleteConfirmModal(false);
  }, [questionIdToDelete, presetQuestions, submitPresetQuestions]);

  const cancelDeleteQuestion = useCallback(() => {
    setQuestionIdToDelete(null);
    setShowDeleteConfirmModal(false);
  }, []);

  useEffect(() => {
    const getProducts = async (searchTerm: string) => {
      if (!trieve.trieve || searchTerm.trim().length === 0) {
        setSearchedProducts([]);
        return;
      }
      try {
        const results = await trieve.trieve.searchOverGroups({
          query: searchTerm,
          search_type: "fulltext",
        });

        const apiResults: Product[] = results.results
          .map((result) => {
            const chunk = result.chunks[0]?.chunk as ChunkMetadata | undefined;
            return {
              id: chunk?.id || String(Date.now() + Math.random()),
              image: chunk?.image_urls?.[0] || "",
              price: chunk?.num_value || 0,
              url: chunk?.link || "",
              title: (chunk?.metadata as any)?.title || "Untitled Product",
              groupId: result.group.id || "",
            };
          })
          .filter((p) => p.id && p.title !== "Untitled Product");
        setSearchedProducts(apiResults);
      } catch (error) {
        console.error("Failed to fetch products:", error);
        setSearchedProducts([]);
      }
    };

    if (!isAddingNewQuestion && !editingQuestionId) {
      setSearchedProducts([]); // Clear products if not in edit/add mode
      return;
    }

    if (
      !editFormData.productSearchTerm ||
      editFormData.productSearchTerm.trim().length === 0
    ) {
      setSearchedProducts([]);
      return;
    }

    const handler = setTimeout(() => {
      if (editFormData.productSearchTerm) {
        getProducts(editFormData.productSearchTerm);
      }
    }, 100);

    return () => {
      clearTimeout(handler);
    };
  }, [
    editFormData.productSearchTerm,
    trieve.trieve,
    isAddingNewQuestion,
    editingQuestionId,
  ]);

  const renderItem = (item: PresetQuestion) => {
    const { id, questionText, products: itemProducts } = item;

    if (editingQuestionId === id) {
      return (
        <Box paddingBlockEnd="400">
          <QuestionEditBlock
            title="Edit Preset Question"
            questionText={editFormData.questionText}
            onQuestionTextChange={(value: string) =>
              setEditFormData({ ...editFormData, questionText: value })
            }
            productSearchTerm={editFormData.productSearchTerm || ""}
            onProductSearchTermChange={(value: string) =>
              setEditFormData({ ...editFormData, productSearchTerm: value })
            }
            products={searchedProducts}
            selectedProducts={editFormData.products || []}
            onSelectedProductsChange={handleQuestionSelectedProductsChange}
            onRemoveSelectedProduct={handleRemoveSelectedProduct}
            onSave={handleSaveQuestion}
            onCancel={handleCancelEdit}
          />
        </Box>
      );
    }

    return (
      <ResourceItem
        id={id}
        accessibilityLabel={`View details for ${questionText}`}
        onClick={() => {
          handleStartEditQuestion(item);
        }}
      >
        <InlineStack
          wrap={false}
          blockAlign="center"
          align="space-between"
          gap="200"
        >
          <Box minWidth="0">
            <BlockStack gap="100">
              <Text variant="bodyMd" as="p" fontWeight="semibold" truncate>
                {questionText}
              </Text>
              <Text variant="bodySm" as="p" tone="subdued">
                Targets {itemProducts?.length ?? 0} product(s)
              </Text>
            </BlockStack>
          </Box>
          <ButtonGroup>
            <Button
              icon={EditIcon}
              accessibilityLabel={`Edit question: ${questionText}`}
              onClick={() => {
                handleStartEditQuestion(item);
              }}
              variant="tertiary"
            />
            <Button
              icon={DeleteIcon}
              accessibilityLabel={`Delete question: ${questionText}`}
              onPointerDown={(e: React.PointerEvent) => {
                e.stopPropagation();
                handleDeleteQuestionTrigger(id);
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
      heading="Define suggested questions for your customers"
      action={{
        content: "Add Suggested Question",
        icon: PlusIcon,
        onAction: handleStartAddNewQuestion,
      }}
      image="https://cdn.shopify.com/s/files/1/0262/4074/files/empty-state.svg"
    >
      <BlockStack gap="200">
        <Text as="p">
          Create preset questions to guide users and help them quickly find
          information about specific products.
        </Text>
      </BlockStack>
    </EmptyState>
  );

  return (
    <Card>
      <BlockStack gap="400">
        <BlockStack gap="200">
          <Text variant="headingMd" as="h1">
            Global Preset Questions
          </Text>
          <Text variant="bodyMd" as="p">
            These questions will be displayed in the global chat UI as preset
            questions.
          </Text>
        </BlockStack>

        {presetQuestions.length > 0 ? (
          <ResourceList
            resourceName={{ singular: "question", plural: "questions" }}
            items={presetQuestions}
            renderItem={renderItem}
          />
        ) : (
          !isAddingNewQuestion && emptyStateMarkup
        )}
        {isAddingNewQuestion && (
          <Box paddingBlockStart="400" paddingBlockEnd="400">
            <QuestionEditBlock
              title="Add New Preset Question"
              questionText={editFormData.questionText}
              onQuestionTextChange={(value: string) =>
                setEditFormData({ ...editFormData, questionText: value })
              }
              productSearchTerm={editFormData.productSearchTerm || ""}
              onProductSearchTermChange={(value: string) =>
                setEditFormData({ ...editFormData, productSearchTerm: value })
              }
              products={searchedProducts}
              selectedProducts={editFormData.products || []}
              onSelectedProductsChange={handleQuestionSelectedProductsChange}
              onRemoveSelectedProduct={handleRemoveSelectedProduct}
              onSave={handleSaveQuestion}
              onCancel={handleCancelEdit}
            />
          </Box>
        )}
        {!isAddingNewQuestion && presetQuestions.length > 0 && (
          <InlineStack align="end">
            <ButtonGroup>
              <Button
                onClick={handleStartAddNewQuestion}
                variant="primary"
                icon={PlusIcon}
              >
                Add Suggested Question
              </Button>
            </ButtonGroup>
          </InlineStack>
        )}
      </BlockStack>

      <Modal
        open={showDeleteConfirmModal}
        onClose={cancelDeleteQuestion}
        title="Confirm Deletion"
        primaryAction={{
          content: "Delete",
          onAction: confirmDeleteQuestion,
          destructive: true,
        }}
        secondaryActions={[
          {
            content: "Cancel",
            onAction: cancelDeleteQuestion,
          },
        ]}
      >
        <Modal.Section>
          <BlockStack>
            <Text as="p">
              Are you sure you want to delete this preset question?
            </Text>
          </BlockStack>
        </Modal.Section>
      </Modal>
    </Card>
  );
}
