import {
	IDataObject,
	IExecuteFunctions,
	INodeExecutionData,
	INodeProperties,
	INodeType,
	INodeTypeDescription,
	IRequestOptions,
	NodeConnectionType,
	NodeOperationError,
} from 'n8n-workflow';

interface Chunk {
	chunk_html: string;
	tracking_id: string;
	tag_set: string[];
	time_stamp: string;
	metadata: object;
}

const createChunkProperties: INodeProperties[] = [
	{
		displayName: 'Chunk Html',
		name: 'chunk_html',
		type: 'string',
		default: '',
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['create_chunk'],
			},
			hide: {
				resource: ['tool_call'],
			},
		},
	},
	{
		displayName: 'Additional Fields',
		name: 'additionalFields',
		type: 'collection',
		placeholder: 'Add Field',
		default: {},
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['create_chunk'],
			},
			hide: {
				resource: ['tool_call'],
			},
		},
		options: [
			{
				displayName: 'Tag Set',
				name: 'tag_set',
				type: 'string',
				default: '',
				description: 'Comma-separated list of tags (e.g., "important,project-x,documentation")',
			},
			{
				displayName: 'Tracking ID',
				name: 'tracking_id',
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
				displayName: 'Metadata',
				name: 'metadata',
				type: 'json',
				default: '{}',
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
				resource: ['tool_call'],
				operation: ['get_tool_params'],
			},
			hide: {
				resource: ['chunk'],
			},
		},
	},
	{
		displayName: 'Function Description',
		name: 'function_description',
		type: 'string',
		default: 'A function to determine if the input is important',
		displayOptions: {
			show: {
				resource: ['tool_call'],
				operation: ['get_tool_params'],
			},
			hide: {
				resource: ['chunk'],
			},
		},
	},
	{
		displayName: 'Function Name',
		name: 'function_name',
		type: 'string',
		default: 'is_important',
		displayOptions: {
			show: {
				resource: ['tool_call'],
				operation: ['get_tool_params'],
			},
			hide: {
				resource: ['chunk'],
			},
		},
	},
	{
		displayName: 'Parameters',
		name: 'parameters',
		type: 'fixedCollection',
		typeOptions: {
			multipleValues: true,
			minValue: 1,
			minRequiredFields: 1
		},
		default: {},
		required: true,
		displayOptions: {
			show: {
				resource: ['tool_call'],
				operation: ['get_tool_params'],
			},
			hide: {
				resource: ['chunk'],
			},
		},
		options: [
			{
				displayName: 'Parameter',
				name: 'parameter',
				values: [
					{
						displayName: 'Name',
						name: 'name',
						type: 'string',
						default: '',
						description: 'The name of the parameter',
						required: true,
					},
					{
						displayName: 'Parameter Type',
						name: 'parameter_type',
						type: 'options',
						options: [
							{
								name: 'Boolean',
								value: 'boolean',
							},
							{
								name: 'Number',
								value: 'number',
							},
						],
						default: 'boolean',
						description: 'The type of the parameter',
						required: true,
					},
					{
						displayName: 'Description',
						name: 'description',
						type: 'string',
						default: '',
						description: 'Description of what this parameter does',
						required: true,
					},
				],
			},
		],
	},
]

const searchProperties: INodeProperties[] = [
	{
		displayName: 'Query',
		name: 'query',
		type: 'string',
		default: '',
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['search_chunks'],
			},
			hide: {
				resource: ['tool_call'],
			},
		},
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
		],
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['search_chunks'],
			},
			hide: {
				resource: ['tool_call'],
			},
		},
	},
	{
		displayName: 'Page Size',
		name: 'page_size',
		type: 'number',
		default: 10,
		description: 'Number of results to return per page',
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['search_chunks'],
			},
			hide: {
				resource: ['tool_call'],
			},
		},
	},
	{
		displayName: 'Filter',
		name: 'filter',
		type: 'json',
		default: '{}',
		displayOptions: {
			show: {
				resource: ['chunk'],
				operation: ['search_chunks'],
			},
			hide: {
				resource: ['tool_call'],
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
				displayOptions: {
					show: {
						resource: [
							'chunk',
						],
					},
					hide: {
						resource: [
							'tool_call'
						]
					}
				},
			},
			{
				name: 'Search',
				value: 'search_chunks',
				description: 'Search your chunks',
				action: 'Search chunks',
				displayOptions: {
					show: {
						resource: [
							'chunk',
						],
					},
					hide: {
						resource: [
							'tool_call'
						]
					}
				},
			},
			{
				name: 'Get Tool Call Function Parameters',
				value: 'get_tool_params',
				action: 'Tool call',
				displayOptions: {
					show: {
						resource: [
							'tool_call',
						],
					},
					hide: {
						resource: [
							'chunk'
						]
					}
				},
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
		subtitle: '={{$parameter["operation"] + ": " + $parameter["resource"]}}',
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

		if (operation === 'get_tool_params') {
			const parameters = this.getNodeParameter('parameters.parameter', 0) as IDataObject[];
			if (!parameters || parameters.length === 0) {
				throw new NodeOperationError(this.getNode(), 'At least one parameter is required for tool call operation');
			}
			const functionName = this.getNodeParameter('function_name', 0) as string;
			if (functionName.includes(' ')) {
				throw new NodeOperationError(this.getNode(), 'Function name cannot contain spaces');
			}
		}

		if (operation === 'create_chunk') {
			const chunk_html = this.getNodeParameter('chunk_html', 0) as string;
			if (!chunk_html) {
				throw new NodeOperationError(this.getNode(), 'Chunk HTML is required for create chunk operation');
			}
		}

		if (operation === 'search_chunks') {
			const query = this.getNodeParameter('query', 0) as string;
			if (!query) {
				throw new NodeOperationError(this.getNode(), 'Query is required for search operation');
			}
		}

		// Iterates over all input items and add the key "myString" with the
		// value the parameter "myString" resolves to.
		// (This could be a different value for each item in case it contains an expression)
		for (let itemIndex = 0; itemIndex < items.length; itemIndex++) {
			if (operation === 'create_chunk') {
				const chunk_html = this.getNodeParameter('chunk_html', itemIndex) as string;
				const additionalFields = this.getNodeParameter('additionalFields', itemIndex) as IDataObject;
				const metadata = additionalFields.metadata as string | undefined;
				let tracking_id = additionalFields.tracking_id as string | undefined;
				const time_stamp_string = additionalFields.time_stamp as string | null;
				const tag_set_string = additionalFields.tag_set as string | undefined;

				let time_stamp: string | undefined;
				if (time_stamp_string == "" || time_stamp_string == null) {
					time_stamp = undefined;
				} else {
					time_stamp = new Date(time_stamp_string).toISOString();
				}

				let tag_set: string[] | undefined;
				if (tag_set_string) {
					tag_set = tag_set_string.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0);
				}

				if (tracking_id == "" ) {
					tracking_id = undefined;
				}

				let parsedMetadata: object | undefined;
				if (metadata === '' || metadata === undefined) {
					parsedMetadata = undefined;
				} else {
					parsedMetadata = JSON.parse(metadata);
				}

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						chunk_html: chunk_html,
						tracking_id: tracking_id,
						tag_set: tag_set,
						upsert_by_tracking_id: true,
						metadata: parsedMetadata,
						time_stamp: time_stamp,
					},
					uri: `https://api.trieve.ai/api/chunk`,
					json: true,
				};

				const response = await this.helpers.requestWithAuthentication.call(this, 'trieveApi', options)

				// Extract only the editable fields from chunk_metadata
				const editableFields: Chunk = {
					chunk_html: response.chunk_metadata.chunk_html,
					metadata: response.chunk_metadata.metadata,
					tag_set: response.chunk_metadata.tag_set,
					time_stamp: response.chunk_metadata.time_stamp,
					tracking_id: response.chunk_metadata.tracking_id,
				};

				returnData.push(editableFields as any);
			} else if (operation === 'search_chunks') {
				console.log("search_chunks");
				const query = this.getNodeParameter('query', itemIndex) as string;
				const search_type = this.getNodeParameter('search_type', itemIndex) as string;
				const filterSerialized = this.getNodeParameter('filter', itemIndex) as IDataObject;
				const page_size = this.getNodeParameter('page_size', itemIndex) as number;
				let filters = JSON.parse(filterSerialized as unknown as string);

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						query: query,
						search_type: search_type,
						filters: filters,
						page_size: page_size
					},
					uri: `https://api.trieve.ai/api/chunk/search`,
					json: true,
				};

				const response = await this.helpers.requestWithAuthentication.call(this, 'trieveApi', options)

				// Process each chunk into a separate item
				for (const item of response.chunks) {
					const chunk = {
						chunk_html: item.chunk.chunk_html,
						metadata: item.chunk.metadata,
						tag_set: item.chunk.tag_set,
						time_stamp: item.chunk.time_stamp,
						tracking_id: item.chunk.tracking_id,
					} as Chunk;

					returnData.push({
						json: {
							chunk,
							score: item.score,
							highlights: item.highlights,
						}
					});
				}
			} else if (operation === 'get_tool_params') {
				const functionInput = this.getNodeParameter('function_input', itemIndex) as string;
				const functionName = this.getNodeParameter('function_name', itemIndex) as string;
				const functionDescription = this.getNodeParameter('function_description', itemIndex) as string;
				const parameters = this.getNodeParameter('parameters.parameter', itemIndex) as IDataObject[];
				const parameterArray = parameters.map(param => ({
					name: param.name,
					parameter_type: param.parameter_type,
					description: param.description
				}));

				const options: IRequestOptions = {
					headers: {
						'Accept': 'application/json',
					},
					method: 'POST',
					body: {
						user_message_text: functionInput,
						tool_function: {
							name: functionName,
							description: functionDescription,
							parameters: parameterArray
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
