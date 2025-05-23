import {
	IAuthenticateGeneric,
	ICredentialTestRequest,
	ICredentialType,
	INodeProperties,
} from 'n8n-workflow';

export class TrieveApi implements ICredentialType {
	name = 'trieveApi';
	displayName = 'Trieve API';
	documentationUrl = 'https://github.com/devflowinc/trieve/blob/main/clients/n8n-nodes-trieve/README.md';
	properties: INodeProperties[] = [
		{
			displayName: 'API Key',
			name: 'apiKey',
			type: 'string',
			typeOptions: { password: true },
			default: '',
		},
		{
			displayName: 'Dataset ID',
			name: 'datasetId',
			type: 'string',
			default: '',
		}
	];

	authenticate: IAuthenticateGeneric = {
		type: 'generic',
		properties: {
			headers: {
				Authorization: '=Bearer {{$credentials.apiKey}}',
				'Content-Type': 'application/json',
				'TR-Dataset': "={{ $credentials.datasetId }}",
				'X-API-Version': 'v2'
			},
		},
	};

	test: ICredentialTestRequest = {
		request: {
			baseURL: 'https://api.trieve.ai',
			url: '=/api/dataset/{{ $credentials.datasetId }}',
			headers: {
				Authorization: '=Bearer {{$credentials.apiKey}}',
				'Content-Type': 'application/json',
				'TR-Dataset': "={{ $credentials.datasetId }}",
			}
		},
	};
}
