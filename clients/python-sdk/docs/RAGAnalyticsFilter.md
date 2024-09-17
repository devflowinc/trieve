# RAGAnalyticsFilter


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 
**rag_type** | [**RagTypes**](RagTypes.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.rag_analytics_filter import RAGAnalyticsFilter

# TODO update the JSON string below
json = "{}"
# create an instance of RAGAnalyticsFilter from a JSON string
rag_analytics_filter_instance = RAGAnalyticsFilter.from_json(json)
# print the JSON string representation of the object
print(RAGAnalyticsFilter.to_json())

# convert the object into a dict
rag_analytics_filter_dict = rag_analytics_filter_instance.to_dict()
# create an instance of RAGAnalyticsFilter from a dict
rag_analytics_filter_form_dict = rag_analytics_filter.from_dict(rag_analytics_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


