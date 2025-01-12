import { PdfSpotlight } from "react-pdf-spotlight";
import { PdfChunk } from "../../utils/types";
import React, { memo, useEffect, useState } from "react";
import { extractMarkedContent, getPresignedUrl } from "../Search/PdfItem";
import { cached } from "../../utils/cache";
import { useModalState } from "../../utils/hooks/modal-context";
import { useAtom } from "jotai";
import { pdfViewState } from "./PdfViewer";

export const ChatPdfItem = memo((props: { chunk: PdfChunk["chunk"] }) => {
  const [found, setFound] = useState(true);
  const state = useModalState();
  const [presigned, setPresigned] = useState<string | null>(null);

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [fullScreenState, setFullScreenState] = useAtom(pdfViewState);

  useEffect(() => {
    const getPresigned = async () => {
      const presignedUrlResult = await cached(() => {
        return getPresignedUrl(
          state.props.baseUrl || "http://localhost:8090",
          state.props.datasetId,
          props.chunk.metadata.file_id,
          state.props.apiKey,
        );
      }, `file-presigned:${props.chunk.metadata.file_name}`);

      setPresigned(presignedUrlResult);
    };

    getPresigned();
  }, []);

  if (!found) return false;
  return (
    <div
      onClick={() => {
        if (presigned) {
          setFullScreenState({
            url: presigned,
            page: props.chunk.metadata.page_num,
            file_name: props.chunk.metadata.file_name,
            searchFor: extractMarkedContent(props.chunk.chunk_html || ""),
          });
        }
      }}
      className="tv-m-2 tv-min-w-[25rem]"
    >
      {presigned && (
        <PdfSpotlight
          height={140}
          padding={{
            horizontal: 170,
            vertical: 80,
          }}
          canvasStyle={{
            borderRadius: "8px",
            border: "1px solid #e5e5e5",
            background: "white",
            boxShadow:
              "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
          }}
          onFoundResult={setFound}
          page={props.chunk.metadata.page_num}
          searchFor={extractMarkedContent(props.chunk.chunk_html || "")}
          url={presigned}
        />
      )}
    </div>
  );
});
