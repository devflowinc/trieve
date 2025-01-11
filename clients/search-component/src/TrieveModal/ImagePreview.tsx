import React from "react";
import { useModalState } from "../utils/hooks/modal-context";

const ImagePreview = ({
  imageUrl,
  isUploading,
  active,
}: {
  imageUrl: string;
  isUploading: boolean;
  active?: boolean;
}) => {
  const { setImageUrl } = useModalState();
  return (
    <>
      {isUploading ? (
        <div className="tv-relative tv-w-fit">
          <div className={`tv-max-h-96 tv-max-w-32 ${active ? "tv-border tv-p-2" : ""}`}>
            <div className="tv-animate-spin tv-h-8 tv-w-8 tv-border-4 tv-border-blue-500 tv-rounded-full tv-border-t-transparent"></div>
          </div>
        </div>
      ) : imageUrl ? (
        <div className="tv-relative tv-w-fit">
          {active && (
            <button
              onClick={() => {
                setImageUrl("");
              }}
              className={`tv-flex tv-items-center tv-justify-center tv-absolute tv--right-3 tv--top-2 tv-rounded-full tv-bg-zinc-500`}
            >
              <i className="fa-solid fa-close tv-p-1 tv-w-4 tv-h-4 tv-cursor-pointer tv-z-10"></i>
            </button>
          )}
          <div className={`tv-max-h-96 tv-max-w-32 ${active ? "tv-border tv-p-2" : ""}`}>
            <img src={imageUrl} alt="" />
          </div>
        </div>
      ) : null}
    </>
  );
};

export default ImagePreview;
