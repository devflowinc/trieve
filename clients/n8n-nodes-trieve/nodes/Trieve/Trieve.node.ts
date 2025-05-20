import {
	IDataObject,
	IExecuteFunctions,
	INodeExecutionData,
	INodeProperties,
	INodeType,
	INodeTypeDescription,
	NodeConnectionType,
} from 'n8n-workflow';

import {
	ChunkFilter,
	SearchMethod,
	TrieveSDK
} from "trieve-ts-sdk";

const operations: INodeProperties[] = [
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		options: [
			{
				name: 'Create Chunk',
				value: 'create_chunk',
				description: 'Create a chunk',
				action: 'Create a chunk',
			},
			{
				name: 'Search Chunks',
				value: 'search_chunks',
				action: 'Search chunks',
			},
		],
		default: 'create_chunk',
		noDataExpression: true,
	},
]

const resources: INodeProperties[] = [
	{
		displayName: 'Resource',
		name: 'resource',
		type: 'options',
		noDataExpression: true,
		default: 'chunk',
		displayOptions: {
			show: {
				operation: [
					'create_chunk',
				],
			},
		},
		options: [
			{
				name: 'Chunk',
				value: 'chunk',
			},
		],
	},
	{
		displayName: 'Query',
		name: 'query',
		type: 'collection',
		default: {
			query: '',
			search_type: 'fulltext'
		},
		options: [
			{
				displayName: 'Query',
				name: 'query',
				type: 'string',
				default: '',
			},
			{
				displayName: 'Search Type',
				name: 'search_type',
				type: 'options',
				default: 'fulltext',
				options: [
					{
						name: 'Fulltext',
						value: 'fulltext',
					},
					{
						name: 'Hybrid',
						value: 'hybrid',
					},
					{
						name: 'Semantic',
						value: 'semantic',
					}
				]
			},
		],
		displayOptions: {
			show: {
				operation: [
					'search_chunks',
				],
			},
		},
		noDataExpression: false,
	},
	{
		displayName: 'Filter',
		name: 'filter',
		type: 'json',
		default: '{}',
		displayOptions: {
			show: {
				operation: [
					'search_chunks',
				],
			},
		}
	},
	{
		displayName: 'Chunk',
		name: 'chunk',
		type: 'collection',
		default: {
			chunk_html: '',
			tracking_id: '',
		},
		displayOptions: {
			show: {
				resource: [
					'chunk',
				],
				operation: [
					'create_chunk',
				],
			},
		},
		noDataExpression: false,
		options: [
			{
				displayName: 'Chunk Html',
				name: 'chunk_html',
				type: 'string',
				default: '',
			},
			{
				displayName: 'Group Tracking ID',
				name: 'group_tracking_id',
				type: 'string',
				default: '',
			},
			{
				displayName: 'Metadata',
				name: 'metadata',
				type: 'json',
				default: '',
			},
			{
				displayName: 'Tag Set',
				name: 'tag_set',
				type: 'string',
				default: '',
			},
			{
				displayName: 'Time Stamp',
				name: 'time_stamp',
				type: 'dateTime',
				default: '',
			},
			{
				displayName: 'Tracking ID',
				name: 'tracking_id',
				type: 'string',
				default: '',
			},
		],
	}
]

export class Trieve implements INodeType {
	description: INodeTypeDescription = {
		// Basic node details will go here
		properties: [
			// Resources and operations will go here
			...operations,
			...resources,
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
		outputs: [NodeConnectionType.Main],
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

		const credentials = await this.getCredentials('trieveApi');
		const trieve = new TrieveSDK({
			apiKey: credentials.apiKey as string,
			datasetId: credentials.datasetId as string
		});

		const returnData: any[] = [];

		const operation = this.getNodeParameter('operation', 0) as string;

		// Iterates over all input items and add the key "myString" with the
		// value the parameter "myString" resolves to.
		// (This could be a different value for each item in case it contains an expression)
		for (let itemIndex = 0; itemIndex < items.length; itemIndex++) {
			if (operation === 'create_chunk') {
				const chunk = this.getNodeParameter('chunk', itemIndex) as IDataObject;
				const data: IDataObject = {
				};
				Object.assign(data, chunk);
				const chunk_html = data.chunk_html as string;
				const group_tracking_id = data.group_tracking_id as string;
				const metadata = data.metadata as object;
				let tracking_id = data.tracking_id as string | undefined;
				let time_stamp = data.time_stamp as string;

				const tag_set_input = data.tag_set as string | undefined;
				let tag_set: string[] | undefined;

				if (tag_set_input == "" || tag_set_input == undefined) {
					tag_set = undefined;
				} else {
					tag_set = tag_set_input.split(",");
				}

				if (tracking_id == "" || group_tracking_id == "") {
					tracking_id = undefined;
				}

				const response = await trieve.createChunk({
					chunk_html: chunk_html,
					tracking_id: tracking_id,
					tag_set: tag_set ?? undefined,
					upsert_by_tracking_id: true,
					group_tracking_ids: group_tracking_id ? [group_tracking_id] : undefined,
					metadata: metadata,
					time_stamp: time_stamp,
				});

				returnData.push(response.chunk_metadata as any);
			} else if (operation === 'search_chunks') {
				const query = this.getNodeParameter('query', itemIndex) as IDataObject;
				const filterSerialized = this.getNodeParameter('filter', itemIndex) as IDataObject;
				let filters: ChunkFilter = JSON.parse(filterSerialized as unknown as string);

				const search_type = query.search_type as SearchMethod ?? "fulltext";

				const response = await trieve.search({
					query: query.query as string,
					search_type: search_type,
					filters: filters
				})

				returnData.push(response as any);
			} else if (operation === 'tool_call') {

			}
		}
		return [this.helpers.returnJsonArray(returnData)];
	}
}
