/*
  Warnings:

  - You are about to drop the column `userId` on the `ApiKey` table. All the data in the column will be lost.
  - A unique constraint covering the columns `[shop]` on the table `ApiKey` will be added. If there are existing duplicate values, this will fail.

*/
-- DropIndex
DROP INDEX "ApiKey_userId_shop_key";

-- AlterTable
ALTER TABLE "ApiKey" DROP COLUMN "userId";

-- CreateIndex
CREATE UNIQUE INDEX "ApiKey_shop_key" ON "ApiKey"("shop");
