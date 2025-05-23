import {
	IDataObject,
	IExecuteFunctions,
	INodeExecutionData,
	INodeProperties,
	INodeType,
	INodeTypeDescription,
	IRequestOptions,
	NodeConnectionType,
} from 'n8n-workflow';

const createChunkProperties: INodeProperties[] = [
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
				requiresDataPath: 'multiple',
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
	},
]

const toolCallProperties: INodeProperties[] = [
	{
		displayName: 'Function Input',
		name: 'function_input',
		type: 'string',
		default: '',
		displayOptions: {
			show: {
				operation: [
					'tool_call',
				],
			},
		},
	},
	{
		displayName: 'Tool Call Parameters',
		name: 'tool_function',
		type: 'collection',
		noDataExpression: false,
		default: {
			function_description: 'A function to determine if the input is important',
			function_name: 'is_important',
			parameters: `
[
	{
		"name": "is_important",
		"parameter_type": "boolean",
		"description": "Is this input important?"
	}
]`,

		},
		displayOptions: {
			show: {
				resource: [
					'tool_call',
				],
				operation: [
					'tool_call',
				],
			},
		},
		options: [
			{
				displayName: 'Function Description',
				name: 'description',
				type: 'string',
				default: 'A function to determine if the input is important',
			},
			{
				displayName: 'Function Name',
				name: 'name',
				type: 'string',
				default: 'is_important',
			},
			{
				displayName: 'Parameters',
				name: 'parameters',
				type: 'json',
				default: `
[
	{
		"name": "",
		"parameter_type": "boolean",
		"description": ""
	}
]`,
			},
		],
	},
]

const searchProperties: INodeProperties[] = [
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
]

const baseProperties: INodeProperties[] = [
	{
		displayName: 'Resource',
		name: 'resource',
		type: 'options',
		default: 'chunk',
		options: [
			{
				name: 'Chunk',
				value: 'chunk',
			},
			{
				name: 'Tool Call',
				value: 'tool_call',
			}
		],
		noDataExpression: true,
	},
	{
		displayName: 'Operation',
		name: 'operation',
		type: 'options',
		options: [
			{
				name: 'Create Chunk',
				value: 'create_chunk',
				description: 'Create a chunk',
				action: 'Create chunk',
			},
			{
				name: 'Search',
				value: 'search_chunks',
				description: 'Search your chunks',
				action: 'Search chunks',
			},
			{
				name: 'Tool Call',
				value: 'tool_call',
				action: 'Tool call',
			}
		],
		default: 'create_chunk',
		noDataExpression: true,
	},
]

export class Trieve implements INodeType {
	description: INodeTypeDescription = {
		// Basic node details will go here
		properties: [
			// Resources and operations will go here
			...baseProperties,
			...createChunkProperties,
			...searchProperties,
			...toolCallProperties
		],
		displayName: 'Trieve',
		name: 'trieve',
		// icon: 'file:shoes-1.svg',
		icon: 'file:TrieveLogo.svg',
		group: ['transform'],
		version: 1,
		subtitle: '={{ $parameter["operation"] }}',
		description: 'Consume The Trieve API',
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

		const returnData: any[] = [];

		const operation = this.getNodeParameter('operation', 0) as string;

		// Iterates over all input items and add the key "myString" with the
		// value the parameter "myString" resolves to.
		// (This could be a different value for each item in case it contains an expression)
		for (let itemIndex = 0; itemIndex < items.length; itemIndex++) {
			console.log("hi");
			if (operation === 'create_chunk') {
				const chunk = this.getNodeParameter('chunk', itemIndex) as IDataObject;
				const data: IDataObject = {
				};
				Object.assign(data, chunk);
				const chunk_html = data.chunk_html as string;
				const group_tracking_id = data.group_tracking_id as string;
				const metadata = data.metadata as object;
				let tracking_id = data.tracking_id as string | undefined;
				const time_stamp_string = data.time_stamp as string | null;

				let time_stamp: string | undefined;
				if (time_stamp_string == "" || time_stamp_string == null) {
					time_stamp = undefined;
				} else {
					time_stamp = new Date(time_stamp_string).toISOString();
				}

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

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						chunk_html: chunk_html,
						tracking_id: tracking_id,
						tag_set: tag_set ?? undefined,
						upsert_by_tracking_id: true,
						group_tracking_ids: group_tracking_id ? [group_tracking_id] : undefined,
						metadata: metadata,
						time_stamp: time_stamp,
					},
					uri: `https://api.trieve.ai/api/chunk`,
					json: true,
				};

				const response = await this.helpers.requestWithAuthentication.call(this, 'trieveApi', options)

				returnData.push(response.chunk_metadata as any);
			} else if (operation === 'search_chunks') {
				console.log("search_chunks");
				const query = this.getNodeParameter('query', itemIndex) as IDataObject;
				const filterSerialized = this.getNodeParameter('filter', itemIndex) as IDataObject;
				let filters = JSON.parse(filterSerialized as unknown as string);

				const search_type = query.search_type ?? "fulltext";

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						query: query.query as string,
						search_type: search_type,
						filters: filters
					},
					uri: `https://api.trieve.ai/api/chunk/search`,
					json: true,
				};

				const response = await this.helpers.requestWithAuthentication.call(this, 'trieveApi', options)

				returnData.push(response.chunks as any);
			} else if (operation === 'tool_call') {

				const toolFunction = this.getNodeParameter('tool_function', itemIndex) as IDataObject;
				const functionInput = this.getNodeParameter('function_input', itemIndex) as string;

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						user_message_text: functionInput,
						tool_function: {
							name: toolFunction.name as string,
							description: toolFunction.description as string,
							parameters: JSON.parse(toolFunction.parameters as unknown as string)
						}
					},
					uri: "https://api.trieve.ai/api/message/get_tool_function_params",
					json: true,
				}
				const response = await this.helpers.requestWithAuthentication.call(this, 'trieveApi', options)

				returnData.push(response as any);

			}
		}
		return [this.helpers.returnJsonArray(returnData)];
	}
}
