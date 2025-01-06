import React, { useEffect, useState } from "react";
import { PdfChunk } from "../../utils/types";
import { FileDTO } from "trieve-ts-sdk";
import { useModalState } from "../../utils/hooks/modal-context";
import { cached } from "../../utils/cache";
import { PdfSpotlight } from "react-pdf-spotlight";
import { useAtom } from "jotai";
import { pdfViewState } from "../PdfView/PdfViewer";

type Props = {
  item: PdfChunk;
  requestID: string;
  index: number;
  className?: string;
};

export function extractMarkedContent(text: string): string {
  const regex = /<mark><b>(.*?)<\/b><\/mark>/i;
  const match = text.match(regex);
  if (!match) return "";

  // Remove any remaining HTML tags and convert to lowercase
  return match[1].replace(/<[^>]*>/g, "").toLowerCase();
}

export const getPresignedUrl = async (
  baseUrl: string,
  datasetId: string,
  fileId: string,
  apiKey: string,
) => {
  const params = {
    content_type: "application/pdf",
  };
  const queryParams = new URLSearchParams(params).toString();
  const result = await fetch(`${baseUrl}/api/file/${fileId}?${queryParams}`, {
    headers: {
      "TR-Dataset": datasetId,
      Authorization: `Bearer ${apiKey}`,
    },
  });

  if (!result.ok) {
    throw new Error("Error fetching presigned url");
  }

  const presignedUrl = (await result.json()) as FileDTO;

  return presignedUrl.s3_url;
};

export const PdfItem = (props: Props) => {
  const [presigned, setPresigned] = useState<string | null>(null);
  const toHighlight = extractMarkedContent(props.item.chunk.highlight || "");
  const state = useModalState();
  const [hasFoundMatch, setHasFoundMatch] = useState(true);

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [fullScreenState, setFullScreenState] = useAtom(pdfViewState);

  useEffect(() => {
    const getPresigned = async () => {
      const presignedUrlResult = await cached(() => {
        return getPresignedUrl(
          state.props.baseUrl || "http://localhost:8090",
          state.props.datasetId,
          props.item.chunk.metadata.file_id,
          state.props.apiKey,
        );
      }, `file-presigned:${props.item.chunk.metadata.file_name}`);

      setPresigned(presignedUrlResult);
    };

    getPresigned();
  }, []);

  if (!hasFoundMatch) {
    return null;
  }

  return (
    <div
      onClick={() => {
        if (presigned && props.item.chunk.highlight) {
          setFullScreenState({
            url: presigned,
            page: props.item.chunk.metadata.page_num,
            file_name: props.item.chunk.metadata.file_name,
            searchFor: toHighlight,
          });
        }
      }}
      className="pdf-item"
    >
      {presigned && (
        <div className="max-w-[400px]">
          <PdfSpotlight
            height={180}
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
            onFoundResult={(r) => setHasFoundMatch(r)}
            page={props.item.chunk.metadata.page_num}
            searchFor={toHighlight}
            url={presigned}
          />
          <div className="pdf-result-page">
            <div className="pdf-result-filename">
              {props.item.chunk.metadata.file_name}
            </div>
            Page {props.item.chunk.metadata.page_num}
          </div>
        </div>
      )}
    </div>
  );
};
