# EditMessageReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**audio_input** | **str** | The base64 encoded audio input of the user message to attach to the topic and then generate an assistant message in response to. | [optional] 
**concat_user_messages_query** | **bool** | If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false. | [optional] 
**context_options** | [**ContextOptions**](ContextOptions.md) |  | [optional] 
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**highlight_options** | [**HighlightOptions**](HighlightOptions.md) |  | [optional] 
**image_urls** | **List[str]** | The URL of the image(s) to attach to the message. | [optional] 
**llm_options** | [**LLMOptions**](LLMOptions.md) |  | [optional] 
**message_sort_order** | **int** | The sort order of the message to edit. | 
**new_message_content** | **str** | The new content of the message to replace the old content with. | [optional] 
**no_result_message** | **str** | No result message for when there are no chunks found above the score threshold. | [optional] 
**only_include_docs_used** | **bool** | Only include docs used in the completion. If not specified, this defaults to false. | [optional] 
**page_size** | **int** | Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None. | [optional] 
**score_threshold** | **float** | Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0. | [optional] 
**search_query** | **str** | Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None. | [optional] 
**search_type** | [**SearchMethod**](SearchMethod.md) |  | [optional] 
**sort_options** | [**SortOptions**](SortOptions.md) |  | [optional] 
**topic_id** | **str** | The id of the topic to edit the message at the given sort order for. | 
**use_group_search** | **bool** |  | [optional] 
**user_id** | **str** | The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results. | [optional] 

## Example

```python
from trieve_py_client.models.edit_message_req_payload import EditMessageReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of EditMessageReqPayload from a JSON string
edit_message_req_payload_instance = EditMessageReqPayload.from_json(json)
# print the JSON string representation of the object
print(EditMessageReqPayload.to_json())

# convert the object into a dict
edit_message_req_payload_dict = edit_message_req_payload_instance.to_dict()
# create an instance of EditMessageReqPayload from a dict
edit_message_req_payload_form_dict = edit_message_req_payload.from_dict(edit_message_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


