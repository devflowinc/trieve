# ScrollChunksReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**offset_chunk_id** | **str** | Offset chunk id is the id of the chunk to start the page from. If not specified, this defaults to the first chunk in the dataset sorted by id ascending. | [optional] 
**page_size** | **int** | Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time. | [optional] 
**sort_by** | [**SortByField**](SortByField.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.scroll_chunks_req_payload import ScrollChunksReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of ScrollChunksReqPayload from a JSON string
scroll_chunks_req_payload_instance = ScrollChunksReqPayload.from_json(json)
# print the JSON string representation of the object
print(ScrollChunksReqPayload.to_json())

# convert the object into a dict
scroll_chunks_req_payload_dict = scroll_chunks_req_payload_instance.to_dict()
# create an instance of ScrollChunksReqPayload from a dict
scroll_chunks_req_payload_form_dict = scroll_chunks_req_payload.from_dict(scroll_chunks_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


