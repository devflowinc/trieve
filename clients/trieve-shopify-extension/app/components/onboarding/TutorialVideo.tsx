import { ReactNode, useState } from "react";
import { XIcon } from "@shopify/polaris-icons";
import { createPortal } from "react-dom";

interface TutorialVideoProps {
  title: string;
  url: string;
  editStoreButton?: ReactNode;
}
export const TutorialVideo = (props: TutorialVideoProps) => {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button className="opacity-80" onClick={() => setOpen(true)}>
        Watch Tutorial
      </button>
      {open &&
        createPortal(
          <div
            onClick={() => setOpen(false)}
            className="bg-neutral-800/20 h-full w-full fixed top-0 left-0 z-[800]"
          >
            <div className="flex flex-col items-center justify-center h-full z-[900] w-full">
              <div className="w-full h-full bg-neutral-800/10 flex flex-col items-center justify-center">
                <div
                  onClick={(e) => e.stopPropagation()}
                  className="bg-white shadow rounded-lg overflow-hidden"
                >
                  <div className="p-2 pl-4 py-3 font-semibold flex justify-between items-center">
                    {props.title}
                    <div className="flex items-center gap-4">
                      {props.editStoreButton}
                      <button onClick={() => setOpen(false)} className="p-1">
                        <XIcon width={23} height={23}></XIcon>
                      </button>
                    </div>
                  </div>
                  <iframe
                    className="w-[80vw] aspect-video"
                    src="https://www.youtube.com/embed/_FUHj3XF8O0?si=1WCOl7evpyH5j4WY"
                    title="YouTube video player"
                    frameBorder="0"
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    referrerPolicy="strict-origin-when-cross-origin"
                    allowFullScreen
                  ></iframe>
                </div>
              </div>
            </div>
          </div>,
          document.body,
        )}
    </>
  );
};
