# SearchChunkQueryResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**corrected_query** | **str** |  | [optional] 
**score_chunks** | [**List[ScoreChunkDTO]**](ScoreChunkDTO.md) |  | 
**total_chunk_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_chunk_query_response_body import SearchChunkQueryResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of SearchChunkQueryResponseBody from a JSON string
search_chunk_query_response_body_instance = SearchChunkQueryResponseBody.from_json(json)
# print the JSON string representation of the object
print(SearchChunkQueryResponseBody.to_json())

# convert the object into a dict
search_chunk_query_response_body_dict = search_chunk_query_response_body_instance.to_dict()
# create an instance of SearchChunkQueryResponseBody from a dict
search_chunk_query_response_body_form_dict = search_chunk_query_response_body.from_dict(search_chunk_query_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


