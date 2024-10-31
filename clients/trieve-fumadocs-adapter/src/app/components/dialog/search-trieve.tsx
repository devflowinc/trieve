"use client";

import { type ReactNode, useState } from "react";
import {
  SearchDialog,
  SharedProps,
  TagItem,
  TagsList,
} from "fumadocs-ui/components/dialog/search";
import { TrieveSDK } from "trieve-ts-sdk";
import { useTrieveSearch } from "../../search/client";
import React from "react";
import { useOnChange } from "fumadocs-core/utils/use-on-change";
import { RootProvider } from "fumadocs-ui/provider";

export interface TrieveSearchDialogProps extends SharedProps {
  trieveClient: TrieveSDK;
  footer?: ReactNode;

  defaultTag?: string;
  tags?: TagItem[];

  /**
   * Add the "Powered by Trieve" label
   *
   * @defaultValue true
   */
  showTrieve?: boolean;

  /**
   * Allow to clear tag filters
   *
   * @defaultValue false
   */
  allowClear?: boolean;
}

export default function TrieveSearchDialog({
  trieveClient,
  tags,
  defaultTag,
  showTrieve = true,
  allowClear = false,
  ...props
}: TrieveSearchDialogProps): React.ReactElement {
  const [tag, setTag] = useState(defaultTag);
  const { search, setSearch, query } = useTrieveSearch(
    trieveClient,
    undefined,
    tag,
  );

  useOnChange(defaultTag, (v) => {
    setTag(v);
  });

  return (
    <RootProvider>
      <SearchDialog
        search={search}
        onSearchChange={setSearch}
        results={query.data ?? []}
        isLoading={query.isLoading}
        {...props}
        footer={
          tags ? (
            <>
              <TagsList
                tag={tag}
                onTagChange={setTag}
                items={tags}
                allowClear={allowClear}>
                {showTrieve ? <TrieveTitle /> : null}
              </TagsList>
              {props.footer}
            </>
          ) : (
            props.footer
          )
        }
      />
    </RootProvider>
  );
}

function TrieveTitle(): React.ReactNode {
  return (
    <a href="https://trieve.ai/?ref=assistant-ui" target="_blank">
      <div className="text-muted-foreground text-end text-xs">
        Powered by <span className="font-semibold">Trieve.ai</span>
      </div>
    </a>
  );
}
