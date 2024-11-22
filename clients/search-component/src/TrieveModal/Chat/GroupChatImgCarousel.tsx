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
    <div>
      {currentGroup && groupCarouselItems ? (
        <div className="flex gap-2 w-full overflow-x-scroll">
          {groupCarouselItems.map((image) => (
            <div key={image}>
              <img className="min-w-[100px]" src={image} />
            </div>
          ))}
        </div>
      ) : undefined}
    </div>
  );
};
