# RAGUsageResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**total_queries** | **int** |  | 

## Example

```python
from trieve_py_client.models.rag_usage_response import RAGUsageResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RAGUsageResponse from a JSON string
rag_usage_response_instance = RAGUsageResponse.from_json(json)
# print the JSON string representation of the object
print(RAGUsageResponse.to_json())

# convert the object into a dict
rag_usage_response_dict = rag_usage_response_instance.to_dict()
# create an instance of RAGUsageResponse from a dict
rag_usage_response_form_dict = rag_usage_response.from_dict(rag_usage_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


