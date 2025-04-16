-- CreateTable
CREATE TABLE "JudgeOAuthState" (
    "id" TEXT NOT NULL,
    "apiKeyId" TEXT NOT NULL,
    "code" TEXT NOT NULL,

    CONSTRAINT "JudgeOAuthState_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "JudgeMeKeys" (
    "id" TEXT NOT NULL,
    "trieveApiKeyId" TEXT NOT NULL,
    "authKey" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "JudgeMeKeys_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "JudgeOAuthState_code_key" ON "JudgeOAuthState"("code");

-- AddForeignKey
ALTER TABLE "JudgeOAuthState" ADD CONSTRAINT "JudgeOAuthState_apiKeyId_fkey" FOREIGN KEY ("apiKeyId") REFERENCES "ApiKey"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "JudgeMeKeys" ADD CONSTRAINT "JudgeMeKeys_trieveApiKeyId_fkey" FOREIGN KEY ("trieveApiKeyId") REFERENCES "ApiKey"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
