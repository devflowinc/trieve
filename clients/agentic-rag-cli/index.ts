#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { TrieveSDK, Topic, ChunkMetadata } from 'trieve-ts-sdk';
import { program } from 'commander';
import chalk from 'chalk';
import inquirer from 'inquirer';
import Conf from 'conf';
import os from 'os';
import readline from 'readline';

interface UploadedFile {
  fileName: string;
  filePath: string;
  trackingId: string;
  uploadedAt: string;
  status: 'pending' | 'completed';
}

const configDir = process.env.XDG_CONFIG_HOME || `${os.homedir()}/.config`;
const config = new Conf({
  cwd: `${configDir}/trieve-cli`,
  configName: 'config',
});

// Path for storing uploaded files tracking data
const uploadedFilesPath = path.join(
  `${configDir}/trieve-cli`,
  'uploaded_files.json',
);

// Path for storing topics data
const topicsPath = path.join(`${configDir}/trieve-cli`, 'topics.json');

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

  if (!apiKey) {
    console.error(
      chalk.red('Error: TRIEVE_API_KEY is not set in env or config.'),
    );
    console.log(chalk.yellow('Run the following command to set it:'));
    console.log(chalk.cyan('trieve-cli configure'));
    process.exit(1);
  }
  if (!datasetId) {
    console.error(
      chalk.red('Error: TRIEVE_DATASET_ID is not set in env or config.'),
    );
    console.log(chalk.yellow('Run the following command to set it:'));
    console.log(chalk.cyan('trieve-cli configure'));
    process.exit(1);
  }
  return { apiKey, datasetId };
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
        `trieve-cli check-upload-status --tracking-id "${options.trackingId ?? `tracking-${filePath.split('/').pop() ?? filePath}`}"`,
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
    const { apiKey, datasetId } = ensureTrieveConfig();
    const trieveClient: TrieveSDK = new TrieveSDK({
      apiKey,
      datasetId,
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

    // Create a topic
    console.log(chalk.blue('📝 Creating a new topic...'));
    const ownerId =
      (config.get('userId') as string) ||
      'default-user-' + Math.random().toString(36).substring(2, 15);

    const topicData = await trieveClient.createTopic({
      first_user_message: question,
      name: topicName,
      owner_id: ownerId,
    });

    // Save the topic for future reference
    manageTopics('add', topicData as Topic);

    console.log(chalk.green(`✅ Topic created: ${topicName}`));
    console.log(chalk.blue('🔍 Fetching answer...'));

    // Create a message and stream the response
    const { reader } = await trieveClient.createMessageReaderWithQueryId({
      topic_id: topicData.id,
      new_message_content: question,
      use_agentic_search: true,
    });

    // Stream the response
    const decoder = new TextDecoder();
    let fullResponse = '';
    let parsedChunks: ChunkMetadata[] = [];
    let isCollapsed = true;
    let isChunkSection = true; // Initially assume we're receiving chunks
    let actualAnswer = '';

    // Set up keyboard interaction for collapsible chunks
    readline.emitKeypressEvents(process.stdin);
    if (process.stdin.isTTY) {
      process.stdin.setRawMode(true);
    }

    const keyPressHandler = (
      str: string,
      key: { name: string; ctrl?: boolean; sequence?: string },
    ) => {
      // Handle both key.name and raw sequence for better compatibility
      if (key.name === 'j' || key.sequence === 'j') {
        isCollapsed = !isCollapsed;
        // Clear console and redisplay with updated collapse state
        console.clear();

        if (parsedChunks.length > 0) {
          if (isCollapsed) {
            console.log(
              chalk.cyan(
                `📚 Found ${parsedChunks.length} reference chunks (press 'j' to expand)`,
              ),
            );
          } else {
            console.log(formatChunksCollapsible(parsedChunks));
          }

          // Add a separator between chunks and answer
          console.log(chalk.dim('─'.repeat(40) + ' Answer ' + '─'.repeat(40)));
        }

        if (actualAnswer) {
          console.log(actualAnswer);
        }
      } else if (key.name === 'c' && key.ctrl) {
        // Allow Ctrl+C to exit
        process.exit();
      }
    };

    process.stdin.on('keypress', keyPressHandler);

    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        fullResponse += chunk;

        // Check if we've reached the separator between chunks and answer
        if (isChunkSection && fullResponse.includes('||')) {
          isChunkSection = false;
          const parts = fullResponse.split('||');

          try {
            // The first part should contain the JSON array of chunks
            const chunksJson = parts[0].trim();
            if (chunksJson) {
              parsedChunks = JSON.parse(chunksJson);

              // Only show a collapsed summary initially
              if (isCollapsed) {
                console.log(
                  chalk.cyan(
                    `📚 Found ${parsedChunks.length} reference chunks (press 'j' to expand)`,
                  ),
                );
              } else {
                console.log(formatChunksCollapsible(parsedChunks));
              }

              // Add a separator between chunks and answer
              console.log(
                chalk.dim('─'.repeat(40) + ' Answer ' + '─'.repeat(40)),
              );
            }
          } catch (e) {
            console.error(chalk.red('❌ Error parsing chunks:'), e);
          }

          // Start displaying the actual answer from the second part
          actualAnswer = parts[1] || '';
          process.stdout.write(actualAnswer);
        } else if (!isChunkSection) {
          // We're in the answer section, just display the chunk
          actualAnswer += chunk;
          process.stdout.write(chunk);
        }
      }
    } catch (e) {
      console.error(chalk.red('❌ Error streaming response:'), e);
    } finally {
      reader.releaseLock();
      // Clean up the keypress listener
      if (process.stdin.isTTY) {
        process.stdin.setRawMode(false);
      }
      process.stdin.removeListener('keypress', keyPressHandler);
    }

    console.log('\n' + '─'.repeat(80));
    console.log(chalk.green('✅ Response complete'));

    if (parsedChunks.length > 0) {
      console.log(
        chalk.blue(
          `📚 ${parsedChunks.length} reference chunks used (press 'j' to ${isCollapsed ? 'expand' : 'collapse'})`,
        ),
      );
    }

    console.log(
      chalk.blue(`� Topic ID: ${topicData.id} (saved for future reference)`),
    );
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

  const summary = chalk.cyan(`📚 Found ${chunks.length} reference chunks`);
  const collapsedMessage = chalk.dim(`(Use 'j' to expand/collapse references)`);

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
          `\n  "${plainText.substring(0, 150)}${plainText.length > 150 ? '...' : ''}"`,
        );
      }

      return `${header}\n  ${details}${contentPreview}`;
    })
    .join('\n');

  return `${summary} ${collapsedMessage}\n${formattedChunks}`;
}

program
  .name('trieve-cli')
  .description('A CLI tool for using Trieve')
  .version('1.0.0');

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
  .command('configure')
  .description('Set up or update your Trieve CLI configuration')
  .action(async () => {
    const answers = await inquirer.prompt([
      {
        type: 'input',
        name: 'TRIEVE_API_KEY',
        message: 'Enter your TRIEVE_API_KEY:',
        default: (config.get('TRIEVE_API_KEY') as string) || '',
      },
      {
        type: 'input',
        name: 'TRIEVE_DATASET_ID',
        message: 'Enter your TRIEVE_DATASET_ID:',
        default: (config.get('TRIEVE_DATASET_ID') as string) || '',
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
    config.set('userId', answers.userId);
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

program.addHelpText(
  'after',
  `
${chalk.yellow('Examples:')}
  $ ${chalk.green('trieve-cli configure')}
  $ ${chalk.green('trieve-cli upload path/to/file.txt -t my-tracking-id')}
  $ ${chalk.green('trieve-cli check-upload-status')}
  $ ${chalk.green('trieve-cli check-upload-status --tracking-id <tracking-id>')}
  $ ${chalk.green('trieve-cli ask "What is the capital of France?"')}
  $ ${chalk.green('trieve-cli ask')}
`,
);

program.parse();
