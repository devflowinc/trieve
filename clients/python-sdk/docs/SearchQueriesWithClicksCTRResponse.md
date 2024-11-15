# SearchQueriesWithClicksCTRResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clicked_chunk** | [**ChunkMetadataWithPosition**](ChunkMetadataWithPosition.md) |  | 
**created_at** | **str** |  | 
**query** | **str** |  | 
**request_id** | **str** |  | 
**results** | **List[object]** |  | 

## Example

```python
from trieve_py_client.models.search_queries_with_clicks_ctr_response import SearchQueriesWithClicksCTRResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SearchQueriesWithClicksCTRResponse from a JSON string
search_queries_with_clicks_ctr_response_instance = SearchQueriesWithClicksCTRResponse.from_json(json)
# print the JSON string representation of the object
print(SearchQueriesWithClicksCTRResponse.to_json())

# convert the object into a dict
search_queries_with_clicks_ctr_response_dict = search_queries_with_clicks_ctr_response_instance.to_dict()
# create an instance of SearchQueriesWithClicksCTRResponse from a dict
search_queries_with_clicks_ctr_response_form_dict = search_queries_with_clicks_ctr_response.from_dict(search_queries_with_clicks_ctr_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


