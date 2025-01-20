# ApiKeyRequestParams

The default parameters which will be forcibly used when the api key is given on a request. If not provided, the api key will not have default parameters.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**highlight_options** | [**HighlightOptions**](HighlightOptions.md) |  | [optional] 
**no_result_message** | **str** | Options for handling the response for the llm to return when no results are found | [optional] 
**page_size** | **int** | Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time. | [optional] 
**remove_stop_words** | **bool** | If true, stop words will be removed. Queries that are entirely stop words will be preserved. | [optional] 
**score_threshold** | **float** | Set score_threshold to a float to filter out chunks with a score below the threshold. | [optional] 
**search_type** | [**SearchMethod**](SearchMethod.md) |  | [optional] 
**slim_chunks** | **bool** | Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. | [optional] 
**typo_options** | [**TypoOptions**](TypoOptions.md) |  | [optional] 
**use_quote_negated_terms** | **bool** | If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. | [optional] 

## Example

```python
from trieve_py_client.models.api_key_request_params import ApiKeyRequestParams

# TODO update the JSON string below
json = "{}"
# create an instance of ApiKeyRequestParams from a JSON string
api_key_request_params_instance = ApiKeyRequestParams.from_json(json)
# print the JSON string representation of the object
print(ApiKeyRequestParams.to_json())

# convert the object into a dict
api_key_request_params_dict = api_key_request_params_instance.to_dict()
# create an instance of ApiKeyRequestParams from a dict
api_key_request_params_form_dict = api_key_request_params.from_dict(api_key_request_params_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


