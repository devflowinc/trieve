# SearchMetrics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_metrics import SearchMetrics

# TODO update the JSON string below
json = "{}"
# create an instance of SearchMetrics from a JSON string
search_metrics_instance = SearchMetrics.from_json(json)
# print the JSON string representation of the object
print(SearchMetrics.to_json())

# convert the object into a dict
search_metrics_dict = search_metrics_instance.to_dict()
# create an instance of SearchMetrics from a dict
search_metrics_form_dict = search_metrics.from_dict(search_metrics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


