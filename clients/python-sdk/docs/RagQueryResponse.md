# RagQueryResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[RagQueryEvent]**](RagQueryEvent.md) |  | 

## Example

```python
from trieve_py_client.models.rag_query_response import RagQueryResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RagQueryResponse from a JSON string
rag_query_response_instance = RagQueryResponse.from_json(json)
# print the JSON string representation of the object
print(RagQueryResponse.to_json())

# convert the object into a dict
rag_query_response_dict = rag_query_response_instance.to_dict()
# create an instance of RagQueryResponse from a dict
rag_query_response_form_dict = rag_query_response.from_dict(rag_query_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


