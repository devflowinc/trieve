# SuggestedQueriesReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**context** | **str** | Context is the context of the query. This can be any string under 15 words and 200 characters. The context will be used to generate the suggested queries. Defaults to None. | [optional] 
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**query** | **str** | The query to base the generated suggested queries off of using RAG. A hybrid search for 10 chunks from your dataset using this query will be performed and the context of the chunks will be used to generate the suggested queries. | [optional] 
**search_type** | [**SearchMethod**](SearchMethod.md) |  | [optional] 
**suggestion_type** | [**SuggestType**](SuggestType.md) |  | [optional] 
**suggestions_to_create** | **int** | The number of suggested queries to create, defaults to 10 | [optional] 

## Example

```python
from trieve_py_client.models.suggested_queries_req_payload import SuggestedQueriesReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of SuggestedQueriesReqPayload from a JSON string
suggested_queries_req_payload_instance = SuggestedQueriesReqPayload.from_json(json)
# print the JSON string representation of the object
print(SuggestedQueriesReqPayload.to_json())

# convert the object into a dict
suggested_queries_req_payload_dict = suggested_queries_req_payload_instance.to_dict()
# create an instance of SuggestedQueriesReqPayload from a dict
suggested_queries_req_payload_form_dict = suggested_queries_req_payload.from_dict(suggested_queries_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


