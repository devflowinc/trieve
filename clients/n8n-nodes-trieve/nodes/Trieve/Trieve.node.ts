import {
	IExecuteFunctions,
	INodeExecutionData,
	INodeType,
	INodeTypeDescription,
	NodeConnectionType,
} from 'n8n-workflow';

import {
	TrieveSDK
} from "trieve-ts-sdk";

export class Trieve implements INodeType {
	description: INodeTypeDescription = {
		// Basic node details will go here
		properties: [
			// Resources and operations will go here
			{
				displayName: 'Create Chunk',
				name: 'operation',
				type: 'options',
				options: [
					{
						name: 'Create',
						value: 'create',
						description: 'Create a chunk',
						action: 'Create a chunk',
					},
				],
				default: 'create',
				noDataExpression: true,
			},
			{
				displayName: 'Chunk',
				name: 'chunk',
				type: 'string',
				default: '',
				required: true
			},
		],
		displayName: 'Trieve',
		name: 'trieve',
		icon: 'file:left-right-black-arrow.svg',
		group: ['transform'],
		version: 1,
		subtitle: '={{ $parameter["operation"] + ": " + $parameter["resource"] }}',
		description: 'Consume Trieve API',
		defaults: {
			name: 'Trieve',
		},
		inputs: [NodeConnectionType.Main],
		outputs: [],
		credentials: [
			{
				name: 'trieveApi',
				required: true,
			},
		],

	};
	// The execute method will go here
	async execute(this: IExecuteFunctions): Promise<INodeExecutionData[][]> {
		const items = this.getInputData();

		// const resource = this.getNodeParameter('resource', 0) as string;
		// const operation = this.getNodeParameter('operation', 0) as string;

		const credentials = await this.getCredentials('trieveApi');
		const trieve = new TrieveSDK({
			apiKey: credentials.apiKey as string,
			datasetId: credentials.datasetId as string
		});

		// Iterates over all input items and add the key "myString" with the
		// value the parameter "myString" resolves to.
		// (This could be a different value for each item in case it contains an expression)
		for (let itemIndex = 0; itemIndex < items.length; itemIndex++) {
			let chunk = this.getNodeParameter('chunk', itemIndex, '') as string;
			trieve.createChunk({
				chunk_html: chunk
			});
		}
		return [];
	}
}
