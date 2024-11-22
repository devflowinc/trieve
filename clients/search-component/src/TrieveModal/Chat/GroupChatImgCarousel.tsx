import { useEffect, useState } from "react";
import React from "react";
import { useModalState } from "../../utils/hooks/modal-context";
import { cached } from "../../utils/cache";
import { getAllChunksForGroup } from "../../utils/trieve";

export const GroupChatImgCarousel = () => {
  const { currentGroup, trieveSDK } = useModalState();

  const [groupCarouselItems, setGroupCarouselItems] = useState<
    string[] | null
  >();

  useEffect(() => {
    const setGroupCarousel = async () => {
      if (currentGroup) {
        const groupChunks = await cached(() => {
          return getAllChunksForGroup(currentGroup.id, trieveSDK);
        }, `chunk-ids-${currentGroup.id}`);

        const images = groupChunks
          .map((chunk) => {
            return chunk.image_urls?.[0] || undefined;
          })
          .filter(Boolean) as string[];
        // Deduplicate with set
        const uniqueImages = [...new Set(images)];
        setGroupCarouselItems(uniqueImages);
      } else {
        setGroupCarouselItems(null);
      }
    };

    setGroupCarousel();
  }, [currentGroup]);

  return (
    <>
      {currentGroup && groupCarouselItems ? (
        <div className="group-chat-carousel">
          {groupCarouselItems.map((image) => (
            <div key={image}>
              <img className="max-h-[270px]" src={image} />
            </div>
          ))}
        </div>
      ) : undefined}
    </>
  );
};
