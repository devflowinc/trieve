#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import {
  TrieveSDK,
  Topic,
  ChunkMetadata,
  UpdateDatasetReqPayload,
} from 'trieve-ts-sdk';
import { program } from 'commander';
import chalk from 'chalk';
import inquirer from 'inquirer';
import Conf from 'conf';
import os from 'os';
// Import readline with specific import for better Node.js compatibility
import * as readline from 'readline';

interface UploadedFile {
  fileName: string;
  filePath: string;
  trackingId: string;
  uploadedAt: string;
  status: 'pending' | 'completed';
}

const configDir = process.env.XDG_CONFIG_HOME || `${os.homedir()}/.config`;
const config = new Conf({
  cwd: `${configDir}/trieve`,
  configName: 'config',
});

// Path for storing uploaded files tracking data
const uploadedFilesPath = path.join(
  `${configDir}/trieve`,
  'uploaded_files.json',
);

// Path for storing topics data
const topicsPath = path.join(`${configDir}/trieve`, 'topics.json');

// Function to manage uploaded files tracking
function manageUploadedFiles(
  action: 'get' | 'add',
  fileData?: UploadedFile,
): UploadedFile[] {
  try {
    // Create the directory if it doesn't exist
    if (!fs.existsSync(path.dirname(uploadedFilesPath))) {
      fs.mkdirSync(path.dirname(uploadedFilesPath), { recursive: true });
    }

    // Initialize or read existing data
    let uploadedFiles: UploadedFile[] = [];
    if (fs.existsSync(uploadedFilesPath)) {
      const fileContent = fs.readFileSync(uploadedFilesPath, 'utf-8');
      uploadedFiles = fileContent ? JSON.parse(fileContent) : [];
    }

    if (action === 'add' && fileData) {
      // Add new file data
      uploadedFiles.push(fileData);
      fs.writeFileSync(
        uploadedFilesPath,
        JSON.stringify(uploadedFiles, null, 2),
      );
    }

    return uploadedFiles;
  } catch (error) {
    console.error(
      chalk.red('❌ Error managing file tracking:'),
      error instanceof Error ? error.message : error,
    );
    return [];
  }
}

// Function to manage topics
function manageTopics(action: 'get' | 'add', topicData?: Topic): Topic[] {
  try {
    // Create the directory if it doesn't exist
    if (!fs.existsSync(path.dirname(topicsPath))) {
      fs.mkdirSync(path.dirname(topicsPath), { recursive: true });
    }

    // Initialize or read existing data
    let topics: Topic[] = [];
    if (fs.existsSync(topicsPath)) {
      const fileContent = fs.readFileSync(topicsPath, 'utf-8');
      topics = fileContent ? JSON.parse(fileContent) : [];
    }

    if (action === 'add' && topicData) {
      // Add new topic data
      topics.push(topicData);
      fs.writeFileSync(topicsPath, JSON.stringify(topics, null, 2));
    }

    return topics;
  } catch (error) {
    console.error(
      chalk.red('❌ Error managing topics:'),
      error instanceof Error ? error.message : error,
    );
    return [];
  }
}

function getConfigOrEnv(key: string, envVar: string): string | undefined {
  return process.env[envVar] || (config.get(key) as string);
}

function ensureTrieveConfig() {
  const apiKey = getConfigOrEnv('TRIEVE_API_KEY', 'TRIEVE_API_KEY');
  const datasetId = getConfigOrEnv('TRIEVE_DATASET_ID', 'TRIEVE_DATASET_ID');
  const organizationId = getConfigOrEnv(
    'TRIEVE_ORGANIZATION_ID',
    'TRIEVE_ORGANIZATION_ID',
  );

  if (!apiKey) {
    console.error(
      chalk.red('Error: TRIEVE_API_KEY is not set in env or config.'),
    );
    console.log(chalk.yellow('Run the following command to set it:'));
    console.log(chalk.cyan('trieve configure'));
    process.exit(1);
  }
  if (!datasetId) {
    console.error(
      chalk.red('Error: TRIEVE_DATASET_ID is not set in env or config.'),
    );
    console.log(chalk.yellow('Run the following command to set it:'));
    console.log(chalk.cyan('trieve configure'));
    process.exit(1);
  }
  if (!organizationId) {
    console.error(
      chalk.red('Error: TRIEVE_ORGANIZATION_ID is not set in env or config.'),
    );
    console.log(chalk.yellow('Run the following command to set it:'));
    console.log(chalk.cyan('trieve configure'));
    process.exit(1);
  }
  return { apiKey, datasetId, organizationId };
}

const uploadFile = async (
  filePath: string,
  options: { trackingId?: string } = {},
) => {
  try {
    console.log(chalk.blue('📤 Uploading file:'), chalk.green(filePath));

    const file = fs.readFileSync(filePath);
    const fileEncoded = file.toString('base64');

    const { apiKey, datasetId } = ensureTrieveConfig();
    const trieveClient: TrieveSDK = new TrieveSDK({
      apiKey,
      datasetId,
    });

    const data = await trieveClient.uploadFile({
      base64_file: fileEncoded,
      file_name: filePath.split('/').pop() ?? filePath,
      group_tracking_id:
        options.trackingId ??
        `tracking-${filePath.split('/').pop() ?? filePath}`,
      chunkr_create_task_req_payload: {},
    });

    console.log('File upload response: ', data);
    console.log(chalk.green('✅ File uploaded successfully!'));
    console.log(
      chalk.yellow(
        '⏳ File processing has started. You can check the status with:',
      ),
    );
    console.log(
      chalk.cyan(
        `trieve check-upload-status --tracking-id "${options.trackingId ?? `tracking-${filePath.split('/').pop() ?? filePath}`}"`,
      ),
    );

    // Manage file tracking
    manageUploadedFiles('add', {
      fileName: filePath.split('/').pop() ?? filePath,
      filePath,
      trackingId:
        options.trackingId ??
        `tracking-${filePath.split('/').pop() ?? filePath}`,
      uploadedAt: new Date().toISOString(),
      status: 'pending',
    });
  } catch (error) {
    console.error(
      chalk.red('❌ Upload failed:'),
      error instanceof Error ? error.message : error,
    );
    process.exit(1);
  }
};

async function promptForFile() {
  const answers = await inquirer.prompt([
    {
      type: 'input',
      name: 'filePath',
      message: 'Enter the path to the file you want to upload:',
      validate: (input) => {
        if (!input) return 'File path is required';
        if (!fs.existsSync(input)) return 'File does not exist';
        return true;
      },
    },
    {
      type: 'confirm',
      name: 'confirm',
      message: 'Are you sure you want to upload this file?',
      default: false,
    },
  ]);

  if (answers.confirm) {
    return uploadFile(answers.filePath, {});
  } else {
    console.log(chalk.yellow('Upload cancelled'));
    process.exit(0);
  }
}

async function checkFileUploadStatus(
  groupTrackingId: string,
): Promise<boolean> {
  try {
    const { apiKey, datasetId, organizationId } = ensureTrieveConfig();
    const trieveClient: TrieveSDK = new TrieveSDK({
      apiKey,
      datasetId,
      organizationId,
    });

    const response = await trieveClient.getChunksGroupByTrackingId({
      groupTrackingId,
      page: 1,
    });

    const isCompleted = response.chunks && response.chunks.length > 0;

    return isCompleted;
  } catch (error) {
    return false;
  }
}

// Function to update tracking data with current status
async function updateFileStatuses(): Promise<UploadedFile[]> {
  const files = manageUploadedFiles('get');

  // Create a copy to modify
  const updatedFiles: UploadedFile[] = JSON.parse(JSON.stringify(files));

  for (let i = 0; i < updatedFiles.length; i++) {
    const file = updatedFiles[i];
    // Only check status for files that aren't already marked as completed
    if (file.status !== 'completed') {
      const isCompleted = await checkFileUploadStatus(file.trackingId);
      updatedFiles[i].status = isCompleted ? 'completed' : 'pending';
    }
  }

  // Sort files by uploadedAt timestamp (most recent first)
  updatedFiles.sort((a, b) => {
    return new Date(b.uploadedAt).getTime() - new Date(a.uploadedAt).getTime();
  });

  // Save the updated statuses
  if (updatedFiles.length > 0) {
    fs.writeFileSync(uploadedFilesPath, JSON.stringify(updatedFiles, null, 2));
  }

  return updatedFiles;
}

// Function to display uploaded files and their status
async function listUploadedFiles(): Promise<void> {
  console.log(chalk.blue('📋 Checking status of uploaded files...'));

  const files = await updateFileStatuses();

  if (files.length === 0) {
    console.log(chalk.yellow('No files have been uploaded yet.'));
    return;
  }

  console.log(chalk.green('\nUploaded Files:'));
  console.log('─'.repeat(100));
  console.log(
    chalk.cyan(
      'File Name'.padEnd(30) +
        'Tracking ID'.padEnd(40) +
        'Uploaded At'.padEnd(25) +
        'Status',
    ),
  );
  console.log('─'.repeat(100));

  files.forEach((file) => {
    console.log(
      file.fileName.padEnd(30) +
        file.trackingId.padEnd(40) +
        file.uploadedAt.slice(0, 19).replace('T', ' ').padEnd(25) +
        (file.status === 'completed'
          ? chalk.green('✅ Completed')
          : chalk.yellow('⏳ Pending')),
    );
  });
  console.log('─'.repeat(100));
}

// Function to check specific file by tracking ID
async function checkSpecificFile(trackingId: string): Promise<void> {
  console.log(
    chalk.blue(`📋 Checking status for tracking ID: ${trackingId}...`),
  );

  const isCompleted = await checkFileUploadStatus(trackingId);

  if (isCompleted) {
    console.log(
      chalk.green(
        `✅ File with tracking ID ${trackingId} has been processed successfully.`,
      ),
    );
  } else {
    console.log(
      chalk.yellow(
        `⏳ File with tracking ID ${trackingId} is still pending or not found.`,
      ),
    );
  }
}

// Interactive function to select and check a specific file
async function interactiveCheckStatus(): Promise<void> {
  let files = manageUploadedFiles('get');

  if (files.length === 0) {
    console.log(chalk.yellow('No files have been uploaded yet.'));
    return;
  }

  // Sort files by uploadedAt timestamp (most recent first)
  files.sort((a, b) => {
    return new Date(b.uploadedAt).getTime() - new Date(a.uploadedAt).getTime();
  });

  const fileChoices = files.map((file) => ({
    name: `${file.fileName} (${file.trackingId}) - ${new Date(
      file.uploadedAt,
    ).toLocaleString()}`,
    value: file.trackingId,
  }));

  // Add option to check all files
  fileChoices.unshift({
    name: 'Check all files',
    value: 'all',
  });

  const answer = await inquirer.prompt([
    {
      type: 'list',
      name: 'trackingId',
      message: 'Select a file to check:',
      choices: fileChoices,
    },
  ]);

  if (answer.trackingId === 'all') {
    await listUploadedFiles();
  } else {
    await checkSpecificFile(answer.trackingId);
  }
}

// Function to ask a question and stream back the response
async function askQuestion(question: string): Promise<void> {
  try {
    console.log(chalk.blue('🤔 Processing your question...'));

    const { apiKey, datasetId } = ensureTrieveConfig();
    const trieveClient: TrieveSDK = new TrieveSDK({
      apiKey,
      datasetId,
    });

    // Generate a topic name from the question (use first few words)
    const topicName = question.split(' ').slice(0, 5).join(' ') + '...';

    const ownerId =
      (config.get('userId') as string) ||
      'default-user-' + Math.random().toString(36).substring(2, 15);

    const topicData = await trieveClient.createTopic({
      name: topicName,
      owner_id: ownerId,
    });

    manageTopics('add', topicData as Topic);

    const { reader } = await trieveClient.createMessageReaderWithQueryId({
      topic_id: topicData.id,
      new_message_content: question,
      use_agentic_search: true,
      model: 'o3',
    });

    // Stream the response
    const decoder = new TextDecoder();
    let fullResponse = '';
    let parsedChunks: ChunkMetadata[] = [];
    let isCollapsed = true;
    let chunkData = '';
    let isParsingChunks = false;
    let actualAnswer = '';
    let isThinkingSection = false;

    // Define keypress handler before setting up event listener
    const keyPressHandler = (
      str: string,
      key: { name: string; ctrl?: boolean; sequence?: string },
    ) => {
      // Debug logging - always keep this to verify keypresses are being received
      console.log(
        chalk.dim(
          `DEBUG: Keypress detected: str='${str}', key.name='${key.name}', ctrl=${key.ctrl}, sequence='${key.sequence}'`,
        ),
      );
      // Handle both key.name and raw sequence for better compatibility
      if (str === 'j' || key.name === 'j' || key.sequence === 'j') {
        console.log(`Key 'j' pressed - toggling reference display...`);

        // Add a small delay to ensure the UI updates properly
        setTimeout(() => {
          isCollapsed = !isCollapsed;
          // Clear console and redisplay with updated collapse state
          console.clear();

          if (parsedChunks.length > 0) {
            if (isCollapsed) {
              if (actualAnswer) {
                console.log(actualAnswer);
              }

              console.log(
                chalk.cyan(
                  `📚 Found ${parsedChunks.length} reference chunks (press 'j' to expand,  Ctrl+C to exit)`,
                ),
              );
            } else {
              if (actualAnswer) {
                console.log(actualAnswer);
              }

              console.log(
                chalk.dim('─'.repeat(40) + ' References ' + '─'.repeat(40)),
              );
              console.log(formatChunksCollapsible(parsedChunks));
              process.exit(0);
            }
          } else if (actualAnswer) {
            console.log(actualAnswer);
          }
        }, 100); // 100ms timeout to ensure the keystroke is processed
      } else if (key.name === 'c' && key.ctrl) {
        // Allow Ctrl+C to exit
        process.exit();
      } else {
        // Provide debug feedback for any other keypress
        console.log(
          chalk.dim(`DEBUG: Unhandled key: ${key.name || str || key.sequence}`),
        );
      }
    };

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        fullResponse += chunk;

        // Detect the start of chunk JSON data section
        if (chunk.includes('[{') && !isParsingChunks) {
          isParsingChunks = true;
          chunkData = '';
        }

        // Handle the thinking/processing messages
        if (
          chunk.includes('🤔') ||
          chunk.includes('📝') ||
          chunk.includes('✅') ||
          chunk.includes('🔍')
        ) {
          isThinkingSection = true;
          process.stdout.write(chunk);
          continue;
        }

        // When we encounter the opening [ and "chunk" in the JSON, we're starting to receive chunks
        if (isParsingChunks) {
          chunkData += chunk;

          // Try to find the end of the JSON chunk data (marked by "||" or the start of the actual answer)
          if (chunkData.includes('||')) {
            isParsingChunks = false;
            const parts = chunkData.split('||');

            try {
              // The first part should contain the JSON array of chunks
              const chunksJson = parts[0].trim();
              if (chunksJson) {
                const parsedChunksRaw: {
                  chunk: ChunkMetadata;
                }[] = JSON.parse(chunksJson);
                parsedChunks = parsedChunksRaw.map((item) => item.chunk);
              }
            } catch (e) {
              console.error(chalk.red('❌ Error parsing chunks:'), e);
            }

            // Start displaying the actual answer from the second part
            if (parts[1]) {
              actualAnswer = parts[1] || '';
              process.stdout.write(actualAnswer);
            }
          }
        } else if (!isParsingChunks && !isThinkingSection) {
          // We're in the answer section, just display the chunk
          actualAnswer += chunk;
          process.stdout.write(chunk);
        }
      }
    } catch (e) {
      console.error(chalk.red('❌ Error streaming response:'), e);
    } finally {
      reader.releaseLock();
    }

    console.log('\n' + '─'.repeat(80));
    console.log(chalk.green('✅ Response complete'));

    if (parsedChunks.length > 0) {
      // If not collapsed, show the chunks again
      if (!isCollapsed) {
        console.log(formatChunksCollapsible(parsedChunks));
      }

      // Setup keyboard input handling for interacting with references
      readline.emitKeypressEvents(process.stdin);
      // Set raw mode to get keypress events
      if (process.stdin.isTTY) {
        process.stdin.setRawMode(true);
      }

      // Listen for keypresses
      process.stdin.on('keypress', (str, key) => {
        // Exit on Ctrl+C
        if (key.ctrl && key.name === 'c') {
          process.exit();
        }

        // Handle keypresses with the handler function
        keyPressHandler(str, key);
      });

      console.log(chalk.dim('Press j to expand references, or Ctrl+C to exit'));
    } else {
      console.log(
        chalk.yellow('⚠️ No reference chunks were found for this query.'),
      );
    }
  } catch (error) {
    console.error(
      chalk.red('❌ Failed to process question:'),
      error instanceof Error ? error.message : error,
    );
  }
}

// Function to format chunk metadata in a collapsible way
function formatChunksCollapsible(chunks: ChunkMetadata[]): string {
  if (!chunks || chunks.length === 0) {
    return '';
  }

  // Format each chunk in a more readable way
  const formattedChunks = chunks
    .map((chunk, index) => {
      const header = chalk.yellow(
        `\n📄 Reference #${index + 1}: ${chunk.tracking_id || chunk.id.substring(0, 8)}`,
      );

      // Extract important fields for preview
      const details = [
        chunk.link ? chalk.blue(`🔗 ${chunk.link}`) : '',
        chunk.tag_set?.length
          ? chalk.magenta(`🏷️  Tags: ${chunk.tag_set.join(', ')}`)
          : '',
        chalk.grey(
          `📅 Created: ${new Date(chunk.created_at).toLocaleString()}`,
        ),
      ]
        .filter(Boolean)
        .join('\n  ');

      // Create preview of chunk content (if available)
      let contentPreview = '';
      if (chunk.chunk_html) {
        // Strip HTML tags for clean preview and limit length
        const plainText = chunk.chunk_html.replace(/<[^>]*>?/gm, '');
        contentPreview = chalk.white(
          `\n  "${plainText.substring(0, 1000)}${plainText.length > 1000 ? '...' : ''}"`,
        );
      }

      return `${header}\n  ${details}${contentPreview}`;
    })
    .join('\n');

  return `${formattedChunks}`;
}

program
  .name('trieve')
  .description('A CLI tool for using Trieve')
  .version('0.0.4');

program
  .command('upload')
  .description('Upload a file to the server')
  .argument('[filePath]', 'Path to the file to upload')
  .option('-t, --tracking-id <trackingId>', 'Tracking ID for the upload')
  .action(async (filePath, options) => {
    if (filePath) {
      await uploadFile(filePath, options);
    } else {
      await promptForFile();
    }
  });

program
  .command('update-tool-config')
  .description('Update the tool configuration for a dataset')
  .option(
    '-t, --tool-description <toolDescription>',
    'Description that tells the LLM when it should use the search tool to retrieve information from your dataset',
    'ALWAYS call this search tool for EVERY user question, even if you think you already know the answer. Your knowledge is limited and potentially outdated - you must rely on the provided search tool to get the most accurate and up-to-date information.',
  )
  .option(
    '-q, --query-description <queryDescription>',
    'Description of how the LLM should write its search queries to retrieve relevant information from your dataset',
    'Write a specific query with critical keywords from the user question. Use multiple search queries with different terms if needed to get comprehensive results.',
  )
  .option(
    '-s, --system-prompt <systemPrompt>',
    'Custom system prompt for the AI assistant',
    "You are an AI assistant that helps people find information in a set of documents. You have access to a search tool that can retrieve relevant information from the documents based on a query. YOU MUST ALWAYS CALL AND USE THE SEARCH TOOL FOR EVERY USER QUESTION WITHOUT EXCEPTION. Do not rely on your own knowledge - it may be outdated or incorrect. For each user question: 1) Use the search tool with a well-crafted query 2) If the first search doesn't yield satisfactory results, try additional searches with different terms 3) Only after searching should you formulate your response, citing the information found. Always inform the user that your answer is based on search results from their documents. If you don't find relevant information after multiple searches, be honest about this limitation.",
  )
  .action(async (options) => {
    try {
      console.log(chalk.blue('🔧 Updating tool configuration...'));

      const { apiKey, datasetId, organizationId } = ensureTrieveConfig();

      const trieveClient: TrieveSDK = new TrieveSDK({
        apiKey,
        datasetId,
        organizationId,
      });

      const updatePayload: UpdateDatasetReqPayload = {
        dataset_id: datasetId,
        server_configuration: {
          SYSTEM_PROMPT: options.systemPrompt,
          TOOL_CONFIGURATION: {
            query_tool_options: {
              tool_description: options.toolDescription,
              query_parameter_description: options.queryDescription,
              price_filter_description:
                'The page range filter to use for the search',

              max_price_option_description: 'The maximum page to filter by',

              min_price_option_description: 'The minimum page to filter by',
            },
          },
        },
      };

      await trieveClient.updateDataset(updatePayload);

      console.log(chalk.green('✅ Tool configuration updated successfully!'));
    } catch (error) {
      console.error(
        chalk.red('❌ Failed to update tool configuration:'),
        error instanceof Error ? error.message : error,
      );
    }
  });

program
  .command('configure')
  .description('Set up or update your Trieve CLI configuration')
  .action(async () => {
    const { apiKey, datasetId, organizationId } = ensureTrieveConfig();

    const trieveClient: TrieveSDK = new TrieveSDK({
      apiKey,
      datasetId,
      organizationId,
    });

    const answers = await inquirer.prompt([
      {
        type: 'input',
        name: 'TRIEVE_ORGANIZATION_ID',
        message: 'Enter your TRIEVE_ORGANIZATION_ID:',
        default: (config.get('TRIEVE_ORGANIZATION_ID') as string) || '',
      },
      {
        type: 'input',
        name: 'TRIEVE_DATASET_ID',
        message: 'Enter your TRIEVE_DATASET_ID:',
        default: (config.get('TRIEVE_DATASET_ID') as string) || '',
      },
      {
        type: 'input',
        name: 'TRIEVE_API_KEY',
        message: 'Enter your TRIEVE_API_KEY:',
        default: (config.get('TRIEVE_API_KEY') as string) || '',
      },
      {
        type: 'input',
        name: 'userId',
        message: 'Enter your user ID (for topic ownership):',
        default: (config.get('userId') as string) || '',
      },
    ]);
    config.set('TRIEVE_API_KEY', answers.TRIEVE_API_KEY);
    config.set('TRIEVE_DATASET_ID', answers.TRIEVE_DATASET_ID);
    config.set('TRIEVE_ORGANIZATION_ID', answers.TRIEVE_ORGANIZATION_ID);
    config.set('userId', answers.userId);

    const updatePayload: UpdateDatasetReqPayload = {
      dataset_id: datasetId,
      server_configuration: {
        SYSTEM_PROMPT:
          "You are an AI assistant that helps people find information in a set of documents. You have access to a search tool that can retrieve relevant information from the documents based on a query. YOU MUST ALWAYS CALL AND USE THE SEARCH TOOL FOR EVERY USER QUESTION WITHOUT EXCEPTION. Do not rely on your own knowledge - it may be outdated or incorrect. For each user question: 1) Use the search tool with a well-crafted query 2) If the first search doesn't yield satisfactory results, try additional searches with different terms 3) Only after searching should you formulate your response, citing the information found. Always inform the user that your answer is based on search results from their documents. If you don't find relevant information after multiple searches, be honest about this limitation.",
        TOOL_CONFIGURATION: {
          query_tool_options: {
            tool_description:
              'ALWAYS call this search tool for EVERY user question, even if you think you already know the answer. Your knowledge is limited and potentially outdated - you must rely on the provided search tool to get the most accurate and up-to-date information.',
            query_parameter_description:
              'Write a specific query with critical keywords from the user question. Use multiple search queries with different terms if needed to get comprehensive results.',
            price_filter_description:
              'The page range filter to use for the search',
            max_price_option_description: 'The maximum page to filter by',
            min_price_option_description: 'The minimum page to filter by',
          },
        },
      },
    };

    await trieveClient.updateDataset(updatePayload);
    console.log(chalk.green('✅ Configuration saved!'));
  });

program
  .command('check-upload-status')
  .description('Check the status of uploaded files')
  .option(
    '-t, --tracking-id <trackingId>',
    'Check specific file by tracking ID',
  )
  .action(async (options) => {
    if (options.trackingId) {
      await checkSpecificFile(options.trackingId);
    } else if (options.all) {
      await listUploadedFiles();
    } else {
      await interactiveCheckStatus();
    }
  });

program
  .command('ask')
  .description(
    'Ask a question and get a streamed response (interactive mode if no question provided)',
  )
  .argument('[question]', 'The question to ask')
  .action(async (question) => {
    if (question) {
      await askQuestion(question);
    } else {
      // Interactive mode when no question is provided
      const answers = await inquirer.prompt([
        {
          type: 'input',
          name: 'question',
          message: 'What would you like to ask?',
          validate: (input) => {
            if (!input) return 'Question is required';
            return true;
          },
        },
      ]);

      await askQuestion(answers.question);
    }
  });

program.parse();
