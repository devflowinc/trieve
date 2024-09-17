# RAGUsage


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**RAGAnalyticsFilter**](RAGAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.rag_usage import RAGUsage

# TODO update the JSON string below
json = "{}"
# create an instance of RAGUsage from a JSON string
rag_usage_instance = RAGUsage.from_json(json)
# print the JSON string representation of the object
print(RAGUsage.to_json())

# convert the object into a dict
rag_usage_dict = rag_usage_instance.to_dict()
# create an instance of RAGUsage from a dict
rag_usage_form_dict = rag_usage.from_dict(rag_usage_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


